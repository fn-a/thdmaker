use std::mem;
use thiserror::Error;
use thdfiler::tris::{Mat4, Mesh, Topology};
use thdfiler::util::math::{identitied, transpoint};

pub use thdfiler::gcode;
pub use thdfiler::tris;

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
    /// Mesh does not have vertices.
    #[error("Mesh does not have vertices")]
    MeshNoVertices,
    /// Mesh does not have faces.
    #[error("Mesh does not have faces")]
    MeshNoFaces,
    /// Mesh is not triangular.
    #[error("Mesh is not triangular")]
    MeshNotTriangular,
    /// Unexpected error.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Slicer configuration parameters.
#[derive(Clone, Debug)]
pub struct Config {
    /// Layer height in millimeters.
    pub layer_height: f32,
    /// First layer height in millimeters.
    pub first_height: f32,
    /// Line width (nozzle diameter) in millimeters.
    pub line_width: f32,
    /// Filament diameter in millimeters.
    pub filament_width: f32,
    /// Print speed in mm/min.
    pub print_speed: f32,
    /// Travel speed in mm/min.
    pub travel_speed: f32,
    /// First layer speed in mm/min.
    pub first_speed: f32,
    /// Extrusion multiplier.
    pub extrusion_multiplier: f32,
    /// Enable retraction.
    pub enable_retraction: bool,
    /// Number of perimeter shells.
    pub perimeters: u32,
    /// Dense infill pattern.
    pub dense_infill: DenseInfill,
    /// Sparse infill pattern
    pub sparse_infill: SparseInfill,
    /// Infill density (0.0 - 1.0).
    pub infill_density: f32,
    /// Retraction distance in millimeters.
    pub retraction_distance: f32,
    /// Retraction speed in mm/min.
    pub retraction_speed: f32,
    /// Bed temperature in Celsius.
    pub bed_temperature: f32,
    /// Nozzle temperature in Celsius.
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
    /// Calculate the height of a layer at index i.
    fn height(&self, i: usize) -> f32 {
        // Min i always 0 for base plane.
        if i > 1 {
            self.first_height + self.layer_height * ((i - 1) as f32)
        } else {
            self.first_height * (i as f32)
        }
    }

    /// Calculate the layer range for a given height range.
    fn range(&self, min: f32, max: f32) -> (usize, usize) {
        let miv = (min - self.first_height) / self.layer_height;
        let mav = (max - self.first_height) / self.layer_height;
        ((miv.floor() + 1.0) as usize, (mav.ceil() + 1.0) as usize)
    }
}


/// Main slicer containing all layers.
pub struct Slicer<'a> {
    pub layers: Vec<Layer<'a>>,
    pub config: &'a Config,
}

impl<'a> Slicer<'a> {
    /// Create a new slicer with the given configuration.
    pub fn new(config: &'a Config) -> Self {
        Self {
            layers: Vec::new(),
            config,
        }
    }

    /// Slice a 3D mesh into layers.
    ///
    /// `transform` is the column-major matrix the object-space coordinates are
    /// mapped through (identity for a pre-baked mesh).
    pub fn slice(&mut self, mesh: &Mesh, transform: &Mat4) -> Result<()> {
        let mut pileups: Vec<Vec<Segment2D>> = Vec::new();
        self.pileup(mesh, transform, &mut pileups)?;
        self.windup(pileups)
    }

    /// Intersect every triangle with the layer planes.
    ///
    /// The segments of all meshes are accumulated into the same pileups, so a
    /// scene becomes one set of layers. The inner loop avoids allocations: each
    /// triangle needs at most three intersection points, which live on the
    /// stack, and the vertex transform is skipped when the matrix is the
    /// identity.
    fn pileup(&self, mesh: &Mesh, transform: &Mat4, pileups: &mut Vec<Vec<Segment2D>>) -> Result<()> {
        if mesh.positions.is_empty() {
            Err(Error::MeshNoVertices)?;
        }
        if !mesh.topology.is_triangle() {
            Err(Error::MeshNotTriangular)?;
        }

        let vertices = &mesh.positions;
        // A mesh without an index buffer is a triangle soup.
        let indexed = !mesh.indices.is_empty();
        let faces = if indexed {
            mesh.indices.len() / 3
        } else {
            vertices.len() / 3
        };
        if faces == 0 {
            Err(Error::MeshNoFaces)?;
        }

        let identity = identitied(transform);
        let map = |index: usize| -> [f32; 3] {
            let p = vertices[if indexed {
                mesh.indices[index] as usize
            } else {
                index
            }];
            if identity {
                p
            } else {
                transpoint(transform, &p)
            }
        };

        for face in 0..faces {
            let v0 = map(face * 3);
            let v1 = map(face * 3 + 1);
            let v2 = map(face * 3 + 2);

            let min_y = v0[1].min(v1[1]).min(v2[1]);
            let max_y = v0[1].max(v1[1]).max(v2[1]);

            // A degenerate (horizontal) triangle never crosses a plane.
            if max_y > min_y {
                // Triangle each edge.
                let edges = [(v0, v1), (v1, v2), (v2, v0)];
                // Calculate the layer index range that this triangle intersects.
                let (sta_i, end_i) = self.config.range(min_y, max_y);

                // The direction the cut runs in, taken from the triangle's own
                // normal: the plane's normal crossed with it, which in the (x, z)
                // plane is `(-n.z, n.x)`. The order the two ends come out of the
                // edge loop in is not enough - for an edge that lies in the plane
                // it is as likely to come out one way as the other, and then the
                // two faces that meet along that edge hand it over the same way
                // round instead of opposing each other and cancelling out.
                let (ax, az) = (v1[0] - v0[0], v1[2] - v0[2]);
                let (bx, bz) = (v2[0] - v0[0], v2[2] - v0[2]);
                let (ay, by) = (v1[1] - v0[1], v2[1] - v0[1]);
                let direct = [-(ax * by - ay * bx), ay * bz - az * by];
                let (dirx, dirz) = (direct[0], direct[1]);

                // Resizes `pileups` so the slots `start..=end` exist; it has to
                // happen before the loop below takes its mutable borrow.
                if pileups.len() <= end_i {
                    pileups.resize(end_i + 1, Vec::new());
                }
                for (index, pileup) in     pileups
                    .iter_mut()
                    .enumerate()
                    .skip(sta_i)
                    .take(end_i + 1 - sta_i)
                {
                    // At most one intersection point per edge, so at most three.
                    let mut points = [Point2D::new(0.0, 0.0); 3];
                    let mut count = 0usize;
                    let y_hig = self.config.height(index);

                    // A triangle that lies in the plane is a face of the cross
                    // section rather than part of its outline: cutting it would
                    // hand over an edge of its own triangulation instead. The
                    // faces around it share the edges of that face and carry the
                    // outline. The test has to allow the rounding that leaves a
                    // face meant to be flat a hair off the plane, otherwise the
                    // triangles of a face flush with the bed - one the object
                    // rests on - cut up the loop they should only bound.
                    if Cov(min_y) == Cov(y_hig) && Cov(max_y) == Cov(y_hig) { continue; }

                    for (sta, end) in edges {
                        let y_sta = sta[1];
                        let y_end = end[1];
    
                        if Cov(y_sta) == Cov(y_hig) {
                            // Start point is on the plane.
                            points[count] = Point2D::new(sta[0], sta[2]);
                            count += 1;
                        } else if Cov(y_end) == Cov(y_hig) {
                            // End point is on the plane.
                            points[count] = Point2D::new(end[0], end[2]);
                            count += 1;
                        } else if (y_sta - y_hig) * (y_end - y_hig) < 0.0 {
                            // Edge crosses the plane - interpolate.
                            let t = (y_hig - y_sta) / (y_end - y_sta);
                            points[count] = Point2D::new(
                                sta[0] + t * (end[0] - sta[0]),
                                sta[2] + t * (end[2] - sta[2]),
                            );
                            count += 1;
                        }
                    }
    
                    if count >= 2 {
                        let mut first = points[0];
                        let mut last = points[count - 1];
                        if first == last {
                            last = points[count - 2];
                        }
                        if first != last {
                            // Material on the left of the direction of travel.
                            if (last.x - first.x) * dirx + (last.z - first.z) * dirz < 0.0 {
                                mem::swap(&mut first, &mut last);
                            }
                            pileup.push(Segment2D::new(first, last));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Turns the accumulated segments into addressable layers.
    fn windup(&mut self, pileups: Vec<Vec<Segment2D>>) -> Result<()> {
        if pileups.is_empty() {
            Err(Error::MeshNotTriangular)
        } else {
            for (index, segments) in pileups.into_iter().enumerate() {
                // The direction of the segments on each layer(pileup) maybe inconsistent.
                let mut layer = Layer::new(index, self.config);
                layer.adjacence(segments);
                layer.delineate();
                self.layers.push(layer);
            }
            Ok(())
        }
    }

    /// Slice preview as a tris line strip.
    ///
    /// The positions carry the extrusion (green) and travel (blue grey) colours
    /// as linear RGBA vertex colours, so the bridge back into a line strip mesh
    /// needs no colour decision of its own.
    pub fn preview(&self) -> Mesh {
        let mut indic = StripInd::new(self.config);
        self.impel(&mut indic);
        Mesh {
            topology: Topology::LineStrip,
            positions: indic.positions,
            colors: Some(indic.colors),
            ..Default::default()
        }
    }

    /// Convert slice result to G-code.
    pub fn gcode(&self) -> GCode {
        let config = &self.config;
        let mut indic = GCodeInd::new(config);
        self.impel(&mut indic);
        indic.gcode
    }

    fn impel<T: Indicate>(&self, indic: &mut T) {
        indic.initiation();

        let laylen = self.layers.len();

        // Process each layer.
        for (index, layer) in self.layers.iter().enumerate() {
            indic.laybegin(index, laylen);

            // Print perimeters. A loop begins with the ride to its first point,
            // which printing that first segment takes care of.
            for segments in layer.limbics.iter() {
                for segment in segments.iter() {
                    indic.drawing(segment);
                }
            }

            // Print infill. A span that is not solid is a hole in the fill: the
            // head does not print it, and the ride across it belongs to the next
            // printed span.
            for (solided, segment) in layer.infills.iter() {
                if *solided {
                    indic.drawing(segment);
                }
            }

            // Pull the filament back before the head leaves the layer.
            indic.retracting();
            indic.layfinish();
        }

        indic.retracting();
        indic.completion();
    }
}