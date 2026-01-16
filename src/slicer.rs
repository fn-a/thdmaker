use std::f32::consts;
use std::cmp::Ordering;
use bevy::prelude::{
    StandardMaterial, Material,
    Color, ColorToComponents,
};
use bevy::mesh::{Mesh, PrimitiveTopology};
use thiserror::Error;
use crate::gcode::define::*;

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
    /// Height offset in millimeters
    pub height_offset: f32,
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
            height_offset: 0.0,
            bed_temperature: 60.0,
            nozzle_temperature: 200.0,
        }
    }
}

impl Config {
    /// Calculate the height of a layer at index i
    fn height(&self, i: usize) -> f32 {
        (self.first_height + self.layer_height * (i as f32)) + self.height_offset
    }

    /// Calculate the layer range for a given height range
    fn range(&self, min: f32, max: f32) -> (usize, usize) {
        (
            ((min - self.first_height) / self.layer_height).ceil() as usize,
            ((max - self.first_height) / self.layer_height).floor() as usize
        )
    }
}

/// Dense infill pattern types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DenseInfill {
    Grid,
    Line,
}

impl DenseInfill {
    /// Generate grid infill bounding box
    fn grid((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width;
        let mut infills = Vec::new();

        // Lines along Z direction
        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        // Lines along X direction
        let mut x = min_x;
        while x <= max_x {
            let start = Point2D::new(x, min_z);
            let end = Point2D::new(x, max_z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            x += spacing;
        }

        infills
    }

    /// Generate line infill bounding box
    fn line((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width;
        let mut infills = Vec::new();

        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        infills
    }
}

/// Sparse infill pattern types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SparseInfill {
    Grid,
    Line,
    Concentric,
    Honeycomb,
}

impl SparseInfill {
    /// Generate grid infill bounding box
    fn grid((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let mut infills = Vec::new();

        // Lines along Z direction
        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        // Lines along X direction
        let mut x = min_x;
        while x <= max_x {
            let start = Point2D::new(x, min_z);
            let end = Point2D::new(x, max_z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            x += spacing;
        }

        infills
    }

    /// Generate line infill bounding box
    fn line((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let mut infills = Vec::new();

        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        infills
    }

    /// Generate honeycomb infill bounding box
    fn honeycomb((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let hex_size = spacing * 2.0;
        let mut infills = Vec::new();

        let mut row = 0;
        let mut z = min_z;
        while z <= max_z {
            let offset = if row % 2 == 0 { 0.0 } else { hex_size / 2.0 };
            let mut x = min_x + offset;

            while x <= max_x {
                // Create hexagon pattern
                let cx = x;
                let cz = z;

                for i in 0..6 {
                    let angle1 = consts::PI / 3.0 * i as f32;
                    let angle2 = consts::PI / 3.0 * (i + 1) as f32;

                    let start = Point2D::new(
                        cx + hex_size / 2.0 * angle1.cos(),
                        cz + hex_size / 2.0 * angle1.sin(),
                    );
                    let end = Point2D::new(
                        cx + hex_size / 2.0 * angle2.cos(),
                        cz + hex_size / 2.0 * angle2.sin(),
                    );
                    let line = Segment2D::new(start, end);

                    infills.push(line);
                }

                x += hex_size * 1.5;
            }

            z += hex_size * consts::SQRT_2 * 0.75;
            row += 1;
        }

        infills
    }

    /// Generate concentric infill bounding box
    fn concentric((min, max): (Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        // Maximum radius of the concentric circle
        let max_r = (max_x - min_x).min(max_z - min_z) / 2.0;
        let offset = config.line_width / config.infill_density;
        // Center of the bounding box
        let center = Point2D {
            x: (min_x + max_x) / 2.0,
            z: (min_z + max_z) / 2.0,
        };
        let circles = (max_r / offset).floor() as usize;
        let mut infills = Vec::new();
        // Number of segments per circle
        let resolution: usize = 64;
        
        for i in 1..=circles {
            let radius = i as f32 * offset;
            let mut previous = None;
            for j in 0..resolution {
                let angle = 2.0 * consts::PI * (j as f32) / (resolution as f32);
                let x = center.x + radius * angle.cos();
                let z = center.z + radius * angle.sin();
                if let Some(start) = previous {
                    let end = Point2D { x, z };
                    infills.push(Segment2D { start, end });
                    previous = Some(end);
                } else {
                    previous = Some(Point2D { x, z });
                }
            }
        }

        infills
    }
}

/// A coordinate wrapper value use for floating point comparisons
#[derive(Debug, Clone, Copy, PartialEq)]
struct Cov(f32);

impl Cov {
    /// Epsilon value for floating value comparisons
    const TOLERANCE: f32 = 1e-6;
    const TOLERHALF: f32 = 1e-3;
    /// Calculate the difference between two value
    fn diff(&self, other: &Self) -> f32 {
        (self.0 - other.0).abs()
    }
}

impl From<f32> for Cov {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for Cov {}

impl Ord for Cov {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Cov {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.diff(other) < Self::TOLERANCE {
            Some(Ordering::Equal)
        } else {
            self.0.partial_cmp(&other.0)
        }
    }
}

/// Represents a 2D point in the XZ slice plane (Bevy Y-axis up)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub z: f32,
}

impl Point2D {
    /// Create a new 2D point in XZ plane
    pub fn new(x: f32, z: f32) -> Self {
        Self { x, z }
    }

    /// Check if a point is inside a polygon using ray casting
    pub fn inside(self, polygon: &Polygon2D) -> bool {
        let mut inside = false;
        let n = polygon.points.len();
        if n >= 3 {
            for i in 0..n {
                let j = (i + 1) % n;
                let pi = polygon.points[i];
                let pj = polygon.points[j];
    
                if ((pi.z > self.z) != (pj.z > self.z))
                    && (self.x < (pj.x - pi.x) * (self.z - pi.z) / (pj.z - pi.z) + pi.x)
                {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

/// Represents a 2D line segment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment2D {
    pub start: Point2D,
    pub end: Point2D,
}

impl Segment2D {
    /// Create a new line segment from two points
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    /// Calculate the length of the line segment
    pub fn length(&self) -> f32 {
        let dx = self.end.x - self.start.x;
        let dz = self.end.z - self.start.z;
        (dx * dx + dz * dz).sqrt()
    }

    /// Check if a line segment is inside any of the contours
    pub fn inside(&self, contours: &[Polygon2D]) -> bool {
        let midpoint = Point2D::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.z + self.end.z) / 2.0,
        );

        for contour in contours {
            if midpoint.inside(contour) {
                return true;
            }
        }

        false
    }

    /// Calculate the intersection point of two line segments
    /// Returns None if segments are parallel or do not intersect
    pub fn intersect(&self, other: &Segment2D) -> Option<Point2D> {
        let x1 = self.start.x;
        let z1 = self.start.z;
        let x2 = self.end.x;
        let z2 = self.end.z;
        let x3 = other.start.x;
        let z3 = other.start.z;
        let x4 = other.end.x;
        let z4 = other.end.z;

        let denom = (x1 - x2) * (z3 - z4) - (z1 - z2) * (x3 - x4);

        // Check if lines are parallel
        if denom.abs() < Cov::TOLERANCE {
            return None;
        }

        let t = ((x1 - x3) * (z3 - z4) - (z1 - z3) * (x3 - x4)) / denom;
        let u = -((x1 - x2) * (z1 - z3) - (z1 - z2) * (x1 - x3)) / denom;

        // Check if intersection point is within both segments
        if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
            let x = x1 + t * (x2 - x1);
            let z = z1 + t * (z2 - z1);
            Some(Point2D::new(x, z))
        } else {
            None
        }
    }
}

/// Represents a 2D polygon (contour)
#[derive(Debug, Clone)]
pub struct Polygon2D {
    pub points: Vec<Point2D>,
}

impl Polygon2D {
    /// Create a new empty polygon
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a point to the polygon
    pub fn push(&mut self, point: Point2D) {
        self.points.push(point);
    }

    /// Check if the polygon is closed (first and last points are the same or very close)
    pub fn closed(&self) -> bool {
        if self.points.is_empty() {
            false
        } else {
            let i = self.points.len() - 1;
            let x = Cov::from(self.points[0].x) == Cov::from(self.points[i].x);
            let z = Cov::from(self.points[0].z) == Cov::from(self.points[i].z);
            x && z
        }
    }

    /// Calculate the area of the polygon
    pub fn area(&self) -> f32 {
        if self.points.len() < 3 {
            0.0
        } else {
            let mut area = 0.0;
            let n = self.points.len();
            for i in 0..n {
                let j = (i + 1) % n;
                area += self.points[i].x * self.points[j].z;
                area -= self.points[j].x * self.points[i].z;
            }
            area.abs() / 2.0
        }

    }

    /// Calculate the centroid (center of mass) of the polygon
    pub fn centroid(&self) -> Option<Point2D> {
        if self.points.len() < 3 {
            return None;
        }

        let mut cx = 0.0;
        let mut cz = 0.0;
        let n = self.points.len();

        for point in &self.points {
            cx += point.x;
            cz += point.z;
        }

        Some(Point2D::new(cx / n as f32, cz / n as f32))
    }

    /// Calculate the bounding box of the polygon
    pub fn bounding(&self) -> Option<(Point2D, Point2D)> {
        if !self.points.is_empty() {
            let mut min_x = self.points[0].x;
            let mut max_x = self.points[0].x;
            let mut min_z = self.points[0].z;
            let mut max_z = self.points[0].z;
    
            for point in &self.points {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_z = min_z.min(point.z);
                max_z = max_z.max(point.z);
            }
    
            Some((
                Point2D::new(min_x, min_z),
                Point2D::new(max_x, max_z),
            ))
        } else {
            None
        }

    }

    /// Offset a polygon inward by a given distance
    pub fn offset(self, offset: f32) -> Polygon2D {
        let mut polygon = Polygon2D::new();

        for i in 0..self.points.len() {
            let prev = self.points[(i + self.points.len() - 1) % self.points.len()];
            let curr = self.points[i];
            let next = self.points[(i + 1) % self.points.len()];

            // Calculate inward normal vectors
            let v1 = [curr.x - prev.x, curr.z - prev.z];
            let v2 = [next.x - curr.x, next.z - curr.z];

            let len1 = (v1[0] * v1[0] + v1[1] * v1[1]).sqrt();
            let len2 = (v2[0] * v2[0] + v2[1] * v2[1]).sqrt();

            if len1 >= Cov::TOLERANCE && len2 >= Cov::TOLERANCE {
                let n1 = [-v1[1] / len1, v1[0] / len1];
                let n2 = [-v2[1] / len2, v2[0] / len2];
    
                // Average normal
                let nx = (n1[0] + n2[0]) / 2.0;
                let nz = (n1[1] + n2[1]) / 2.0;
    
                let nlen = (nx * nx + nz * nz).sqrt();
                if nlen >= Cov::TOLERANCE {
                    polygon.push(Point2D::new(
                        curr.x + offset * nx / nlen,
                        curr.z + offset * nz / nlen,
                    ));
                }
            }
        }

        polygon
    }
}

/// Represents a single layer of the slice
#[derive(Debug, Clone)]
pub struct Layer<'a> {
    pub index: usize,
    pub contours: Vec<Polygon2D>,
    pub limbics: Vec<Segment2D>,
    pub infills: Vec<Segment2D>,
    config: &'a Config,
}

impl <'a> Layer<'a> {
    /// Create a new layer with the given height and configuration
    pub fn new(index: usize, config: &'a Config) -> Self {
        Self {
            index,
            contours: Vec::new(),
            limbics: Vec::new(),
            infills: Vec::new(),
            config,
        }
    }

    /// Calculate the height of this layer
    fn height(&self) -> f32 {
        self.config.height(self.index)
    }

    /// Generate layer from contours
    fn delineate(&mut self) {
        if !self.contours.is_empty() {
            // Sort contours by area in descending order
            // The largest contour is the outer wall, smaller ones are inner walls (holes)
            self.contours.sort_by(|a, b| b.area().partial_cmp(&a.area()).unwrap_or(Ordering::Equal));

            // Calculate bounding box of all contours
            let mut pt_min = Point2D::new(f32::MAX, f32::MAX);
            let mut pt_max = Point2D::new(f32::MIN, f32::MIN);

            for (index, contour) in self.contours.iter_mut().enumerate() {
                if let Some((bb_min, bb_max)) = contour.bounding() {
                    pt_min.x = pt_min.x.min(bb_min.x);
                    pt_max.x = pt_max.x.max(bb_max.x);
                    pt_min.z = pt_min.z.min(bb_min.z);
                    pt_max.z = pt_max.z.max(bb_max.z);
                }

                // Determine if this is outer wall (index 0) or inner wall (index > 0)
                // Outer wall: offset inward by line_width
                // Inner wall: offset outward by line_width
                let offset_distance = if index == 0 {
                    -self.config.line_width // inward for outer wall
                } else {
                    self.config.line_width // outward for inner wall
                };

                // Generate perimeters with offset
                let mut current = contour.clone();
                for _ in 0..self.config.perimeters {
                    let n = current.points.len();
                    for i in 0..n {
                        let j = (i + 1) % n;
                        let start = current.points[i];
                        let end = current.points[j];
                        self.limbics.push(Segment2D::new(start, end));
                    }
                    // Offset contour for next perimeter iteration
                    current = current.offset(offset_distance);
                }
            }

            let bounding = (pt_min, pt_max);

            // Generate infill
            let infills = if self.index == 0 {
                 match self.config.dense_infill {
                    DenseInfill::Grid => {
                        DenseInfill::grid(bounding, &self.config)
                    }
                    DenseInfill::Line => {
                        DenseInfill::line(bounding, &self.config)
                    }
                }
            } else {
                match self.config.sparse_infill {
                    SparseInfill::Grid => {
                        SparseInfill::grid(bounding, &self.config)
                    }
                    SparseInfill::Line => {
                        SparseInfill::line(bounding, &self.config)
                    }
                    SparseInfill::Honeycomb => {
                        SparseInfill::honeycomb(bounding, &self.config)
                    }
                    SparseInfill::Concentric => {
                        SparseInfill::concentric(bounding, &self.config)
                    }
                }
            };

            // Clip infill segments to be inside the outer contour but outside inner contours
            self.infills = self.clipedges(&infills);
        }
    }

    /// Clip infill segments to be inside outer contour and outside inner contours
    fn clipedges(&self, infills: &[Segment2D]) -> Vec<Segment2D> {
        let mut segments = Vec::new();

        if !self.contours.is_empty() {
            // Outer contour is the first one (largest area)
            let outer = &self.contours[0];
            // Inner contours are the rest
            let inners = &self.contours[1..];
    
            for segment in infills {
                // Find all intersection points with all contours
                let mut intersections = Vec::new();
    
                // Check intersection with outer contour
                for i in 0..outer.points.len() {
                    let j = (i + 1) % outer.points.len();
                    let contour = Segment2D::new(outer.points[i], outer.points[j]);
    
                    if let Some(point) = segment.intersect(&contour) {
                        intersections.push((point, false)); // false = outer contour
                    }
                }
    
                // Check intersection with inner contours
                for inner_contour in inners {
                    for i in 0..inner_contour.points.len() {
                        let j = (i + 1) % inner_contour.points.len();
                        let contour = Segment2D::new(inner_contour.points[i], inner_contour.points[j]);
    
                        if let Some(point) = segment.intersect(&contour) {
                            intersections.push((point, true)); // true = inner contour
                        }
                    }
                }
    
                // Add start and end points
                intersections.push((segment.start, false));
                intersections.push((segment.end, false));
    
                // Sort intersections along the segment
                intersections.sort_by(|a, b| {
                    let dist_a = (a.0.x - segment.start.x).powi(2) + (a.0.z - segment.start.z).powi(2);
                    let dist_b = (b.0.x - segment.start.x).powi(2) + (b.0.z - segment.start.z).powi(2);
                    dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                });
    
                // Create subsegments and check if they are inside the valid region
                for i in 0..intersections.len() - 1 {
                    let subsegment = Segment2D::new(intersections[i].0, intersections[i + 1].0);
    
                    // Check if midpoint is inside outer contour and outside all inner contours
                    let midpoint = Point2D::new(
                        (subsegment.start.x + subsegment.end.x) / 2.0,
                        (subsegment.start.z + subsegment.end.z) / 2.0,
                    );
    
                    let inouter = midpoint.inside(outer);
                    let outinner = !inners.iter().any(|c| midpoint.inside(c));
    
                    if inouter && outinner {
                        segments.push(subsegment);
                    }
                }
            }
        }

        segments
    }

    /// Connect line segments into closed contours
    fn adjacence(&mut self, segments: Vec<Segment2D>) {
        let mut contours = Vec::new();
        let mut consumes = vec![false; segments.len()];

        for i in 0..segments.len() {
            if consumes[i] {
                continue;
            }

            let mut contour = Polygon2D::new();
            let mut current = segments[i];
            consumes[i] = true;

            contour.push(current.start);

            loop {
                contour.push(current.end);

                // Try to find next segment
                let mut found = false;
                for j in 0..segments.len() {
                    if consumes[j] {
                        continue;
                    }

                    let next = segments[j];
                    let equalx = Cov::from(next.start.x) == Cov::from(current.end.x);
                    let equalz = Cov::from(next.start.z) == Cov::from(current.end.z);

                    // Check if next segment starts where current ends
                    if equalx && equalz {
                        current = next;
                        consumes[j] = true;
                        found = true;
                        break;
                    }
                }

                if !found {
                    break;
                }
            }

            // Close the contour if possible
            let n = contour.points.len();
            if n > 2 {
                let first = contour.points[0];
                let last = contour.points[n - 1];
                let equalx = Cov::from(first.x) == Cov::from(last.x);
                let equalz = Cov::from(first.z) == Cov::from(last.z);

                if equalx && equalz {
                    contour.points.pop();
                }
            }

            if contour.closed() {
                contours.push(contour);
            }
        }
        self.contours = contours;
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
    pub fn slice(&mut self, mesh: &Mesh) -> Result<()> {
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
                let v0 = vertices[index[0]];
                let v1 = vertices[index[1]];
                let v2 = vertices[index[2]];

                let y0 = v0[1]; let y1 = v1[1]; let y2 = v2[1];

                // Triangle each edge
                let edges = [(v0, v1), (v1, v2), (v2, v0)];

                // Calculate the layer index range that this triangle intersects
                let (sta_i, end_i) = self.config.range(y0.min(y1).min(y2), y0.max(y1).max(y2));
                
                if end_i >= sta_i {
                    if pileups.len() <= end_i {
                        pileups.resize(end_i + 1, vec![]);
                    }
                    for idx in sta_i..=end_i {
                        // Find intersection points
                        let mut points = Vec::new();
                        let y_hig = self.config.height(idx);
            
                        for (sta, end) in edges {
                            let y_sta = sta[1];
                            let y_end = end[1];
            
                            if Cov::from(y_sta) == Cov::from(y_hig) {
                                // Start point is on the plane
                                points.push(Point2D::new(sta[0], sta[2]));
                            } else if Cov::from(y_end) == Cov::from(y_hig) {
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
            
                        if points.len() == 2 {
                            pileups[idx].push(Segment2D::new(points[0], points[1]));
                        }
                    }
                }
            }
        }
        
        if pileups.is_empty() {
            Err(Error::MeshNotTriangular)
        } else {
            for (index, segments) in pileups.into_iter().enumerate() {
                let mut layer = Layer::new(index, &self.config);
                layer.adjacence(segments);
                layer.delineate();
                self.layers.push(layer);
            }
            Ok(())
        }
    }

    /// Create a Bevy [`Mesh`] and [`Material`] for display
    pub fn mesh(&self) -> (Mesh, impl Material) {
        let mut positions = Vec::new();
        let mut colors = Vec::new();

        // Colors for different line types
        let solid = Color::srgb(0.0, 1.0, 0.0).to_srgba().to_vec4();
        let empty = Color::srgb(0.4, 0.5, 0.6).to_srgba().to_vec4();

        let mut current_x = 0.0;
        let mut current_z = 0.0;

        for layer in &self.layers {
            let y = layer.height();
            for segment in layer.limbics.iter().chain(&layer.infills) {
                let dx = segment.start.x - current_x;
                let dz = segment.start.z - current_z;
                let ds = (dx * dx + dz * dz).sqrt();

                if ds > Cov::TOLERANCE {
                    // Add travel line
                    positions.push([current_x, y, current_z]);
                    colors.push(empty);
                }

                // Add extrusion line
                positions.push([segment.start.x, y, segment.start.z]);
                colors.push(solid);

                current_x = segment.end.x;
                current_z = segment.end.z;
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::LineList, Default::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

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
        let mut current = Indicate::new(config);

        current.initiation();

        // Process each layer
        for (index, layer) in self.layers.iter().enumerate() {
            current.index = index;
            current.laybegin();

            // Print perimeters
            for (i, segment) in layer.limbics.iter().enumerate() {
                if i == 0 {
                    // Move to start (travel move) - map Point2D (x, z) to printer (X, Y)
                    current.idleride(segment);
                }

                current.drawline(segment);
            }

            // Print infill
            for (i, segment) in layer.infills.iter().enumerate() {
                let greatx = Cov::from(segment.start.x) > Cov::from(current.x);
                let greatz = Cov::from(segment.start.z) > Cov::from(current.z);
                if i == 0 || greatx || greatz {
                    // Travel move - map Point2D (x, z) to printer (X, Y)
                    current.retraction();

                    current.idleride(segment);

                    current.extrusion();
                }

                current.drawline(segment);
            }

            current.layfinish();
        }

        current.completion();

        current.gcode
    }
}

/// Current position and extruder state
#[derive(Debug)]
struct Indicate<'a> {
    x: f32,
    z: f32,
    e: f32,
    gcode: GCode,
    index: usize,
    config: &'a Config,
}

impl <'a> Indicate<'a> {
    /// Create a new [`Indicate`]
    fn new(config: &'a Config) -> Self {
        Self {
            x: 0.0,
            z: 0.0,
            e: 0.0,
            gcode: GCode::default(),
            index: 0,
            config,
        }
    }

    /// G-code generator start
    fn initiation(&mut self) {
        // G-code header
        self.gcode.push(Line::from(&Void).label("Generated by ThdMaker Slicer"));
        self.gcode.push(Line::from(&Void).label(&format!("Layer height: {}mm", self.config.layer_height)));
        self.gcode.push(Line::from(&Void).label(&format!("Line width: {}mm", self.config.line_width)));
        self.gcode.push(Line::from(&Void).label(&format!("Infill: {:.0}%", self.config.infill_density * 100.0)));
        self.gcode.push(Line::from(&Void));

        // Start G-code
        self.gcode.push(Line::from(&G21));
        self.gcode.push(Line::from(&G90));
        self.gcode.push(Line::from(&G28::default()));
        self.gcode.push(Line::from(&*M190::default().set_s(self.config.bed_temperature)));
        self.gcode.push(Line::from(&*M104::default().set_s(self.config.nozzle_temperature)));
        self.gcode.push(Line::from(&*G1::default().set_y(2.0).set_f(3000.0)).label("Move to safe height"));
        self.gcode.push(Line::from(&*G92::default().set_e(0.0)).label("Reset extruder"));
        self.gcode.push(Line::from(&Void));
    }

    /// Layer begin, move to layer height
    fn laybegin(&mut self) {
        self.gcode.push(Line::from(&Void).label(&format!("Layer {}\n", self.index + 1)));
        self.gcode.push(Line::from(&*G1::default().set_z(self.config.height(self.index)).set_f(self.config.travel_speed)));
    }

    /// Draw a line segment
    fn drawline(&mut self, segment: &Segment2D) {
        // Print segment - map Point2D (x, z) to printer (X, Y)
        let dx = segment.end.x - self.x;
        let dz = segment.end.z - self.z;
        let length = (dx * dx + dz * dz).sqrt();

        if length > Cov::TOLERHALF {
            // Calculate extrusion amount
            let filament = consts::PI * self.config.filament_width * self.config.filament_width / 4.0; // 1.75mm filament
            let (layer_h, speed) = if self.index == 0 {
                (self.config.first_height, self.config.first_speed)
            } else {
                (self.config.layer_height, self.config.print_speed)
            };
            let path_area = self.config.line_width * layer_h;
            let extrusion = length * path_area / filament * self.config.extrusion_multiplier;

            self.e += extrusion;

            self.x = segment.end.x;
            self.z = segment.end.z;
            self.gcode.push(Line::from(&*G1::default().set_x(segment.end.x).set_z(segment.end.z).set_e(self.e).set_f(speed)));
        }
    }

    /// Idle travel move
    fn idleride(&mut self, segment: &Segment2D) {
        self.x = segment.start.x;
        self.z = segment.start.z;
        self.gcode.push(Line::from(&*G0::default().set_x(segment.start.x).set_z(segment.start.z).set_f(self.config.travel_speed)));
    }

    /// Extrusion
    fn extrusion(&mut self) {
        if self.config.enable_retraction {
            self.e += self.config.retraction_distance;
            self.gcode.push(Line::from(&*G1::default().set_e(self.e).set_f(self.config.retraction_speed)));
        }
    }

    /// Retraction
    fn retraction(&mut self) {
        if self.config.enable_retraction {
            self.e -= self.config.retraction_distance;
            self.gcode.push(Line::from(&*G1::default().set_e(self.e).set_f(self.config.retraction_speed)));
        }
    }

    /// Layer finish, retract and move to next layer height
    fn layfinish(&mut self) {
        self.gcode.push(Line::from(&Void));
    }

    /// Completion, move to safe height and home
    fn completion(&mut self) {
        // End G-code
        self.gcode.push(Line::from(&Void).label("End of print"));
        self.gcode.push(Line::from(&*G1::default().set_y(10.0).set_f(3000.0)).label("Move to safe height"));
        self.gcode.push(Line::from(&*G28::default().set_x(0.0).set_y(0.0)));
        self.gcode.push(Line::from(&*M104::default().set_s(0.0)));
        self.gcode.push(Line::from(&*M140::default().set_s(0.0)));
        self.gcode.push(Line::from(&M84));
    }
}