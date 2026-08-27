use bevy::prelude::{
    StandardMaterial, Transform,
    Color, Vec3,
};
use bevy::mesh::{Mesh, PrimitiveTopology};
use thiserror::Error;

pub use thdfiler::gcode;

mod geometry;
mod layer;
mod infill;
mod indicate;

use geometry::*;
use layer::Layer;
use indicate::{GCode, Indicate, GCodeInd, StripInd};
use infill::{DenseInfill, SparseInfill};

/// Result type alias for slicer operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur when slicing a mesh.
#[derive(Error, Debug)]
pub enum Error {
    /// Mesh does not have position attribute.
    #[error("Mesh does not have position attribute")]
    MeshNoPosition,
    /// Position attribute is not float3.
    #[error("Position attribute is not float3")]
    MeshNoPosFloat,
    /// Mesh does not have vertices.
    #[error("Mesh does not have vertices")]
    MeshNoVertices,
    /// Mesh does not have indices.
    #[error("Mesh does not have indices")]
    MeshNoIndices,
    /// Mesh is not triangular.
    #[error("Mesh is not triangular")]
    MeshNotTriangular,
    /// Unexpected error.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Slicer configuration parameters
#[derive(Clone, Debug)]
pub struct Config {
    /// Layer height in millimeters
    pub layer_height: f32,
    /// First layer height in millimeters
    pub first_height: f32,
    /// Line width (nozzle diameter) in millimeters
    pub line_width: f32,
    /// Filament diameter in millimeters
    pub filament_width: f32,
    /// Print speed in mm/min
    pub print_speed: f32,
    /// Travel speed in mm/min
    pub travel_speed: f32,
    /// First layer speed in mm/min
    pub first_speed: f32,
    /// Extrusion multiplier
    pub extrusion_multiplier: f32,
    /// Enable retraction
    pub enable_retraction: bool,
    /// Number of perimeter shells
    pub perimeters: u32,
    /// Dense infill pattern
    pub dense_infill: DenseInfill,
    /// Sparse infill pattern
    pub sparse_infill: SparseInfill,
    /// Infill density (0.0 - 1.0)
    pub infill_density: f32,
    /// Retraction distance in millimeters
    pub retraction_distance: f32,
    /// Retraction speed in mm/min
    pub retraction_speed: f32,
    /// Bed temperature in Celsius
    pub bed_temperature: f32,
    /// Nozzle temperature in Celsius
    pub nozzle_temperature: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layer_height: 0.2,
            first_height: 0.2,
            line_width: 0.4,
            filament_width: 1.75,
            print_speed: 1800.0,
            travel_speed: 3000.0,
            first_speed: 900.0,
            extrusion_multiplier: 1.0,
            perimeters: 2,
            dense_infill: DenseInfill::Line,
            sparse_infill: SparseInfill::Grid,
            infill_density: 0.2,
            enable_retraction: true,
            retraction_distance: 2.0,
            retraction_speed: 1200.0,
            bed_temperature: 60.0,
            nozzle_temperature: 200.0,
        }
    }
}

impl Config {
    /// Calculate the height of a layer at index i
    fn height(&self, i: usize) -> f32 {
        // Min i always 0 for base plane
        if i > 1 {
            self.first_height + self.layer_height * ((i - 1) as f32)
        } else {
            self.first_height * (i as f32)
        }
    }

    /// Calculate the layer range for a given height range
    fn range(&self, min: f32, max: f32) -> (usize, usize) {
        let miv = (min - self.first_height) / self.layer_height;
        let mav = (max - self.first_height) / self.layer_height;
        ((miv.floor() + 1.0) as usize, (mav.ceil() + 1.0) as usize)
    }
}


/// Main slicer containing all layers
pub struct Slicer<'a> {
    pub layers: Vec<Layer<'a>>,
    pub config: &'a Config,
}

impl<'a> Slicer<'a> {
    /// Create a new slicer with the given configuration
    pub fn new(config: &'a Config) -> Self {
        Self {
            layers: Vec::new(),
            config,
        }
    }

    /// Slice a 3D mesh into layers
    pub fn slice(&mut self, mesh: &Mesh, transform: &Transform) -> Result<()> {
        // Extract vertices from the mesh
        let vertices = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .ok_or(Error::MeshNoPosition)?
            .as_float3()
            .ok_or(Error::MeshNoPosFloat)?;
        // Extract indices from the mesh
        let indices = mesh
            .indices()
            .ok_or(Error::MeshNoIndices)?;

        // Store each layer's segments that the triangle intersects
        let mut pileups = Vec::new();
        for index in indices.iter().collect::<Vec<_>>().chunks(3) {
            if index.len() == 3 {
                let v0 = Vec3::from_array(vertices[index[0]]);
                let v1 = Vec3::from_array(vertices[index[1]]);
                let v2 = Vec3::from_array(vertices[index[2]]);
                let v0 = transform.transform_point(v0);
                let v1 = transform.transform_point(v1);
                let v2 = transform.transform_point(v2);
                let min_y = v0[1].min(v1[1]).min(v2[1]);
                let max_y = v0[1].max(v1[1]).max(v2[1]);

                if max_y > min_y {
                    // Triangle each edge
                    let edges = [(v0, v1), (v1, v2), (v2, v0)];
    
                    // Calculate the layer index range that this triangle intersects
                    let (sta_i, end_i) = self.config.range(min_y, max_y);

                    if pileups.len() <= end_i {
                        pileups.resize(end_i + 1, vec![]);
                    }

                    for (idx, pileup) in pileups.iter_mut()
                        .enumerate()
                        .skip(sta_i)
                        .take(end_i + 1 - sta_i)
                    {
                        // Find intersection points
                        let mut points = Vec::new();
                        let y_hig = self.config.height(idx);
            
                        for (sta, end) in edges {
                            let y_sta = sta[1];
                            let y_end = end[1];
            
                            if Cov(y_sta) == Cov(y_hig) {
                                // Start point is on the plane
                                points.push(Point2D::new(sta[0], sta[2]));
                            } else if Cov(y_end) == Cov(y_hig) {
                                // End point is on the plane
                                points.push(Point2D::new(end[0], end[2]));
                            } else if (y_sta - y_hig) * (y_end - y_hig) < 0.0 {
                                // Edge crosses the plane - interpolate
                                let t = (y_hig - y_sta) / (y_end - y_sta);
                                let x = sta[0] + t * (end[0] - sta[0]);
                                let z = sta[2] + t * (end[2] - sta[2]);
                                points.push(Point2D::new(x, z));
                            }
                        }

                        if points.len() >= 2 {
                            let n = points.len() - 1;
                            let first = points[0];
                            let mut last = points[n];
                            if first == last {
                                last = points[n - 1];
                            }
                            if first != last {
                                pileup.push(Segment2D::new(first, last));
                            }
                        }
                    }
                }
            }
        }
        
        if pileups.is_empty() {
            Err(Error::MeshNotTriangular)
        } else {
            for (index, segments) in pileups.into_iter().enumerate() {
                // The direction of the segments on each layer(pileup) maybe inconsistent
                let mut layer = Layer::new(index, self.config);
                layer.adjacence(segments);
                layer.delineate();
                self.layers.push(layer);
            }
            Ok(())
        }
    }

    /// Create a Bevy [`Mesh`] and [`StandardMaterial`] for display
    pub fn strip(&self) -> (Mesh, StandardMaterial) {
        let config = &self.config;
        let mut indic = StripInd::new(config);
        self.impel(&mut indic);
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, Default::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indic.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, indic.colors);

        let material = StandardMaterial {
            emissive: Color::srgb(0.0, 0.0, 0.0).to_linear(),
            unlit: true,
            ..Default::default()
        };

        (mesh, material)
    }

    /// Convert slice result to G-code
    pub fn gcode(&self) -> GCode {
        let config = &self.config;
        let mut indic = GCodeInd::new(config);
        self.impel(&mut indic);
        indic.gcode
    }

    fn impel<T: Indicate>(&self, indic: &mut T) {
        indic.initiation();

        let laylen = self.layers.len();
        
        // Process each layer
        for (index, layer) in self.layers.iter().enumerate() {
            indic.laybegin(index, laylen);

            // Print perimeters
            for segments in layer.limbics.iter() {
                let n = segments.len();
                for (i, segment) in segments.iter().enumerate() {
                    if i == 0 {
                        // Move to start (travel move) - map Point2D (x, z) to printer (X, Y)
                        indic.idleing(segment, i, n);
                    }

                    indic.drawing(segment, i, n);
                }
            }

            let n = layer.infills.len();
            // Print infill
            for (i, (solided, segment)) in layer.infills.iter().enumerate() {
                if *solided {
                    // Enable extrusion for solid segments
                    indic.extruing();
                    indic.drawing(segment, i, n);
                } else {
                    // Travel move - map Point2D (x, z) to printer (X, Y)
                    indic.retracting();
                    indic.idleing(segment, i, n);
                }
            }

            indic.layfinish();
        }

        indic.completion();
    }
}