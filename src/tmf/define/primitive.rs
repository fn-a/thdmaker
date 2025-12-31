//! Basic types used throughout the 3MF library.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};
use super::trianglesets::TriangleSets;
use super::beamlattice::BeamLattice;

/// Triangle mesh.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Mesh vertices.
    pub vertices: Vertices,
    /// Mesh triangles.
    pub triangles: Triangles,
    /// Optional triangle sets extension structure.
    pub triangle_sets: TriangleSets,
    /// Optional beam lattice extension structure.
    pub beam_lattice: Option<BeamLattice>,
    /// Optional volumetric extension volume id, reference to volume data.
    pub volume_id: Option<u32>
}

impl Mesh {
    /// Create a new empty mesh.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mesh with given vertices and triangles.
    pub fn with_data(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Self {
        Self {
            vertices: Vertices { vertices },
            triangles: Triangles { triangles },
            triangle_sets: TriangleSets::default(),
            beam_lattice: None,
            volume_id: None,
        }
    }

    /// Add a vertex and return its index.
    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        let index = self.vertices.vertices.len() as u32;
        self.vertices.vertices.push(vertex);
        index
    }

    /// Add a vertex by coordinates.
    pub fn add_vertex_coords(&mut self, x: f64, y: f64, z: f64) -> u32 {
        self.add_vertex(Vertex::new(x, y, z))
    }

    /// Add a triangle and return its index.
    pub fn add_triangle(&mut self, triangle: Triangle) -> u32 {
        let index = self.triangles.triangles.len() as u32;
        self.triangles.triangles.push(triangle);
        index
    }

    /// Add a triangle by vertex indices.
    pub fn add_triangle_indices(&mut self, v1: u32, v2: u32, v3: u32) -> u32 {
        self.add_triangle(Triangle::new(v1, v2, v3))
    }

    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.vertices.len()
    }

    /// Get the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.triangles.triangles.len()
    }

    /// Calculate the bounding box of the mesh.
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        if self.vertices.vertices.is_empty() {
            return None;
        }

        let first = &self.vertices.vertices[0];
        let mut min = Vertex::new(first.x, first.y, first.z);
        let mut max = Vertex::new(first.x, first.y, first.z);

        for v in &self.vertices.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        Some(BoundingBox { min, max })
    }

    /// Calculate the normal of a triangle.
    pub fn triangle_normal(&self, triangle: &Triangle) -> Option<Vertex> {
        let v1 = self.vertices.vertices.get(triangle.v1 as usize)?;
        let v2 = self.vertices.vertices.get(triangle.v2 as usize)?;
        let v3 = self.vertices.vertices.get(triangle.v3 as usize)?;

        let edge1 = v2.sub(v1);
        let edge2 = v3.sub(v1);
        let normal = edge1.cross(&edge2);

        Some(normal.normalize())
    }

    /// Create a simple cube mesh.
    pub fn cube(size: f64) -> Self {
        let half = size / 2.0;
        let mut mesh = Mesh::new();

        // Add 8 vertices
        let v0 = mesh.add_vertex_coords(-half, -half, -half);
        let v1 = mesh.add_vertex_coords(half, -half, -half);
        let v2 = mesh.add_vertex_coords(half, half, -half);
        let v3 = mesh.add_vertex_coords(-half, half, -half);
        let v4 = mesh.add_vertex_coords(-half, -half, half);
        let v5 = mesh.add_vertex_coords(half, -half, half);
        let v6 = mesh.add_vertex_coords(half, half, half);
        let v7 = mesh.add_vertex_coords(-half, half, half);

        // Bottom face
        mesh.add_triangle_indices(v0, v2, v1);
        mesh.add_triangle_indices(v0, v3, v2);
        // Top face
        mesh.add_triangle_indices(v4, v5, v6);
        mesh.add_triangle_indices(v4, v6, v7);
        // Front face
        mesh.add_triangle_indices(v0, v1, v5);
        mesh.add_triangle_indices(v0, v5, v4);
        // Back face
        mesh.add_triangle_indices(v2, v3, v7);
        mesh.add_triangle_indices(v2, v7, v6);
        // Left face
        mesh.add_triangle_indices(v0, v4, v7);
        mesh.add_triangle_indices(v0, v7, v3);
        // Right face
        mesh.add_triangle_indices(v1, v2, v6);
        mesh.add_triangle_indices(v1, v6, v5);

        mesh
    }

    /// Create a simple tetrahedron mesh.
    pub fn tetrahedron(size: f64) -> Self {
        let mut mesh = Mesh::new();
        let s = size / 2.0;

        // Vertices of a regular tetrahedron
        let v0 = mesh.add_vertex_coords(s, s, s);
        let v1 = mesh.add_vertex_coords(s, -s, -s);
        let v2 = mesh.add_vertex_coords(-s, s, -s);
        let v3 = mesh.add_vertex_coords(-s, -s, s);

        // 4 faces
        mesh.add_triangle_indices(v0, v1, v2);
        mesh.add_triangle_indices(v0, v2, v3);
        mesh.add_triangle_indices(v0, v3, v1);
        mesh.add_triangle_indices(v1, v3, v2);

        mesh
    }

    /// Create a rectangular box (cuboid) mesh.
    pub fn box_shape(width: f64, height: f64, depth: f64) -> Self {
        let hw = width / 2.0;
        let hh = height / 2.0;
        let hd = depth / 2.0;

        let mut mesh = Mesh::new();

        // Add 8 vertices
        let v0 = mesh.add_vertex_coords(-hw, -hh, -hd);
        let v1 = mesh.add_vertex_coords(hw, -hh, -hd);
        let v2 = mesh.add_vertex_coords(hw, hh, -hd);
        let v3 = mesh.add_vertex_coords(-hw, hh, -hd);
        let v4 = mesh.add_vertex_coords(-hw, -hh, hd);
        let v5 = mesh.add_vertex_coords(hw, -hh, hd);
        let v6 = mesh.add_vertex_coords(hw, hh, hd);
        let v7 = mesh.add_vertex_coords(-hw, hh, hd);

        // Bottom face (z = -hd)
        mesh.add_triangle_indices(v0, v2, v1);
        mesh.add_triangle_indices(v0, v3, v2);
        // Top face (z = hd)
        mesh.add_triangle_indices(v4, v5, v6);
        mesh.add_triangle_indices(v4, v6, v7);
        // Front face (y = -hh)
        mesh.add_triangle_indices(v0, v1, v5);
        mesh.add_triangle_indices(v0, v5, v4);
        // Back face (y = hh)
        mesh.add_triangle_indices(v2, v3, v7);
        mesh.add_triangle_indices(v2, v7, v6);
        // Left face (x = -hw)
        mesh.add_triangle_indices(v0, v4, v7);
        mesh.add_triangle_indices(v0, v7, v3);
        // Right face (x = hw)
        mesh.add_triangle_indices(v1, v2, v6);
        mesh.add_triangle_indices(v1, v6, v5);

        mesh
    }

    /// Create a cylinder mesh.
    pub fn cylinder(radius: f64, height: f64, segments: usize) -> Self {
        let segments = segments.max(3);
        let half_height = height / 2.0;
        let mut mesh = Mesh::new();

        // Create vertices for top and bottom circles
        let mut bottom_vertices = Vec::with_capacity(segments);
        let mut top_vertices = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (segments as f64);
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            bottom_vertices.push(mesh.add_vertex_coords(x, y, -half_height));
            top_vertices.push(mesh.add_vertex_coords(x, y, half_height));
        }

        // Center vertices for caps
        let bottom_center = mesh.add_vertex_coords(0.0, 0.0, -half_height);
        let top_center = mesh.add_vertex_coords(0.0, 0.0, half_height);

        // Create triangles
        for i in 0..segments {
            let next = (i + 1) % segments;

            // Bottom cap (reversed winding for outward normals)
            mesh.add_triangle_indices(
                bottom_center,
                bottom_vertices[next],
                bottom_vertices[i],
            );

            // Top cap
            mesh.add_triangle_indices(top_center, top_vertices[i], top_vertices[next]);

            // Side triangles
            mesh.add_triangle_indices(
                bottom_vertices[i],
                bottom_vertices[next],
                top_vertices[next],
            );
            mesh.add_triangle_indices(bottom_vertices[i], top_vertices[next], top_vertices[i]);
        }

        mesh
    }

    /// Create a sphere mesh using UV sphere algorithm.
    pub fn sphere(radius: f64, rings: usize, segments: usize) -> Self {
        let rings = rings.max(2);
        let segments = segments.max(3);
        let mut mesh = Mesh::new();

        // Add vertices
        // Top pole
        let top_pole = mesh.add_vertex_coords(0.0, 0.0, radius);

        // Ring vertices
        let mut ring_vertices: Vec<Vec<u32>> = Vec::with_capacity(rings - 1);
        for r in 1..rings {
            let phi = std::f64::consts::PI * (r as f64) / (rings as f64);
            let z = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            let mut ring = Vec::with_capacity(segments);
            for s in 0..segments {
                let theta = 2.0 * std::f64::consts::PI * (s as f64) / (segments as f64);
                let x = ring_radius * theta.cos();
                let y = ring_radius * theta.sin();
                ring.push(mesh.add_vertex_coords(x, y, z));
            }
            ring_vertices.push(ring);
        }

        // Bottom pole
        let bottom_pole = mesh.add_vertex_coords(0.0, 0.0, -radius);

        // Create triangles
        // Top cap
        for s in 0..segments {
            let next = (s + 1) % segments;
            mesh.add_triangle_indices(top_pole, ring_vertices[0][s], ring_vertices[0][next]);
        }

        // Middle rings
        for r in 0..(rings - 2) {
            for s in 0..segments {
                let next = (s + 1) % segments;
                mesh.add_triangle_indices(
                    ring_vertices[r][s],
                    ring_vertices[r + 1][s],
                    ring_vertices[r + 1][next],
                );
                mesh.add_triangle_indices(
                    ring_vertices[r][s],
                    ring_vertices[r + 1][next],
                    ring_vertices[r][next],
                );
            }
        }

        // Bottom cap
        let last_ring = rings - 2;
        for s in 0..segments {
            let next = (s + 1) % segments;
            mesh.add_triangle_indices(
                ring_vertices[last_ring][s],
                bottom_pole,
                ring_vertices[last_ring][next],
            );
        }

        mesh
    }

    /// Set the beam lattice.
    pub fn set_beam_lattice(&mut self, beam_lattice: BeamLattice) {
        self.beam_lattice = Some(beam_lattice);
    }

    /// Get the beam lattice.
    pub fn get_beam_lattice(&self) -> Option<&BeamLattice> {
        self.beam_lattice.as_ref()
    }

    /// Get the number of beams if beam lattice exists.
    pub fn beam_count(&self) -> usize {
        self.beam_lattice.as_ref().map_or(0, |l| l.beams.beams.len())
    }

    /// Get the number of balls if beam lattice exists.
    pub fn ball_count(&self) -> usize {
        self.beam_lattice.as_ref().map_or(0, |l| l.balls.balls.len())
    }
}

/// Bounding box defined by min and max corners.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vertex,
    pub max: Vertex,
}

impl BoundingBox {
    /// Get the size of the bounding box.
    pub fn size(&self) -> (f64, f64, f64) {
        (
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Get the center of the bounding box.
    pub fn center(&self) -> Vertex {
        Vertex::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }
}

/// Unit of measurement for the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    /// Micrometers (μm)
    Micron,
    /// Millimeters (mm) - default
    #[default]
    Millimeter,
    /// Centimeters (cm)
    Centimeter,
    /// Inches
    Inch,
    /// Feet
    Foot,
    /// Meters
    Meter,
}

impl Unit {
    /// Convert a value from this unit to millimeters.
    pub fn millimeter_into(&self, value: f64) -> f64 {
        match self {
            Unit::Micron => value * 0.001,
            Unit::Millimeter => value,
            Unit::Centimeter => value * 10.0,
            Unit::Inch => value * 25.4,
            Unit::Foot => value * 304.8,
            Unit::Meter => value * 1000.0,
        }
    }

    /// Convert a value from millimeters to this unit.
    pub fn millimeter_from(&self, value: f64) -> f64 {
        match self {
            Unit::Micron => value * 1000.0,
            Unit::Millimeter => value,
            Unit::Centimeter => value * 0.1,
            Unit::Inch => value / 25.4,
            Unit::Foot => value / 304.8,
            Unit::Meter => value * 0.001,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Micron => write!(f, "micron"),
            Unit::Millimeter => write!(f, "millimeter"),
            Unit::Centimeter => write!(f, "centimeter"),
            Unit::Inch => write!(f, "inch"),
            Unit::Foot => write!(f, "foot"),
            Unit::Meter => write!(f, "meter"),
        }
    }
}

impl FromStr for Unit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "micron" => Ok(Unit::Micron),
            "millimeter" => Ok(Unit::Millimeter),
            "centimeter" => Ok(Unit::Centimeter),
            "inch" => Ok(Unit::Inch),
            "foot" => Ok(Unit::Foot),
            "meter" => Ok(Unit::Meter),
            _ => Err(Error::InvalidAttribute {
                name: "unit".to_string(),
                message: format!("unknown unit: {}", s),
            }),
        }
    }
}

/// sRGB color with optional alpha channel.
/// 
/// Format: #RRGGBB or #RRGGBBAA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, 255 = fully opaque)
    pub a: u8,
}

impl Color {
    /// Create a new opaque color.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color with alpha.
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Common colors.
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl fmt::Display for Color {
    /// Convert to hex string (#RRGGBB or #RRGGBBAA).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            write!(f, "#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    /// Create a color from a hex string (#RRGGBB or #RRGGBBAA).
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if !s.starts_with('#') {
            return Err(Error::InvalidColor(format!(
                "color must start with '#': {}",
                s
            )));
        }
        let hex = &s[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| Error::InvalidColor(format!("red: {}", s)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| Error::InvalidColor(format!("green: {}", s)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| Error::InvalidColor(format!("blue: {}", s)))?;
                Ok(Color::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| Error::InvalidColor(format!("red: {}", s)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| Error::InvalidColor(format!("green: {}", s)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| Error::InvalidColor(format!("blue: {}", s)))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| Error::InvalidColor(format!("alpha: {}", s)))?;
                Ok(Color::rgba(r, g, b, a))
            }
            _ => Err(Error::InvalidColor(format!(
                "color must be 6 or 8 hex digits: {}",
                s
            ))),
        }
    }
}

/// 4x4 affine transformation matrix stored in row-major order.
/// 
/// The matrix format is:
/// ```text
/// | m00 m01 m02 0 |
/// | m10 m11 m12 0 |
/// | m20 m21 m22 0 |
/// | m30 m31 m32 1 |
/// ```
/// 
/// In 3MF, only 12 values are stored: m00 m01 m02 m10 m11 m12 m20 m21 m22 m30 m31 m32
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3D {
    /// First row: m00, m01, m02
    pub m0: [f64; 3],
    /// Second row: m10, m11, m12
    pub m1: [f64; 3],
    /// Third row: m20, m21, m22
    pub m2: [f64; 3],
    /// Fourth row (translation): m30, m31, m32
    pub m3: [f64; 3],
}

impl Matrix3D {
    /// Create a new identity matrix.
    pub fn identity() -> Self {
        Self {
            m0: [1.0, 0.0, 0.0],
            m1: [0.0, 1.0, 0.0],
            m2: [0.0, 0.0, 1.0],
            m3: [0.0, 0.0, 0.0],
        }
    }

    /// Create a translation matrix.
    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            m0: [1.0, 0.0, 0.0],
            m1: [0.0, 1.0, 0.0],
            m2: [0.0, 0.0, 1.0],
            m3: [x, y, z],
        }
    }

    /// Create a scaling matrix.
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            m0: [sx, 0.0, 0.0],
            m1: [0.0, sy, 0.0],
            m2: [0.0, 0.0, sz],
            m3: [0.0, 0.0, 0.0],
        }
    }

    /// Create a uniform scaling matrix.
    pub fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s, s)
    }

    /// Create a rotation matrix around the X axis.
    pub fn rotation_x(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            m0: [1.0, 0.0, 0.0],
            m1: [0.0, c, s],
            m2: [0.0, -s, c],
            m3: [0.0, 0.0, 0.0],
        }
    }

    /// Create a rotation matrix around the Y axis.
    pub fn rotation_y(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            m0: [c, 0.0, -s],
            m1: [0.0, 1.0, 0.0],
            m2: [s, 0.0, c],
            m3: [0.0, 0.0, 0.0],
        }
    }

    /// Create a rotation matrix around the Z axis.
    pub fn rotation_z(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            m0: [c, s, 0.0],
            m1: [-s, c, 0.0],
            m2: [0.0, 0.0, 1.0],
            m3: [0.0, 0.0, 0.0],
        }
    }

    /// Calculate the determinant of the 3x3 rotation/scale part.
    pub fn determinant_3x3(&self) -> f64 {
        self.m0[0] * (self.m1[1] * self.m2[2] - self.m1[2] * self.m2[1])
            - self.m0[1] * (self.m1[0] * self.m2[2] - self.m1[2] * self.m2[0])
            + self.m0[2] * (self.m1[0] * self.m2[1] - self.m1[1] * self.m2[0])
    }

    /// Check if this is an identity matrix.
    pub fn is_identity(&self) -> bool {
        const EPSILON: f64 = 1e-10;
        (self.m0[0] - 1.0).abs() < EPSILON
            && self.m0[1].abs() < EPSILON
            && self.m0[2].abs() < EPSILON
            && self.m1[0].abs() < EPSILON
            && (self.m1[1] - 1.0).abs() < EPSILON
            && self.m1[2].abs() < EPSILON
            && self.m2[0].abs() < EPSILON
            && self.m2[1].abs() < EPSILON
            && (self.m2[2] - 1.0).abs() < EPSILON
            && self.m3[0].abs() < EPSILON
            && self.m3[1].abs() < EPSILON
            && self.m3[2].abs() < EPSILON
    }

    /// Transform a point using this matrix.
    pub fn transform_point(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let nx = self.m0[0] * x + self.m1[0] * y + self.m2[0] * z + self.m3[0];
        let ny = self.m0[1] * x + self.m1[1] * y + self.m2[1] * z + self.m3[1];
        let nz = self.m0[2] * x + self.m1[2] * y + self.m2[2] * z + self.m3[2];
        (nx, ny, nz)
    }

    /// Multiply two matrices.
    pub fn multiply(&self, other: &Matrix3D) -> Matrix3D {
        Matrix3D {
            m0: [
                self.m0[0] * other.m0[0] + self.m0[1] * other.m1[0] + self.m0[2] * other.m2[0],
                self.m0[0] * other.m0[1] + self.m0[1] * other.m1[1] + self.m0[2] * other.m2[1],
                self.m0[0] * other.m0[2] + self.m0[1] * other.m1[2] + self.m0[2] * other.m2[2],
            ],
            m1: [
                self.m1[0] * other.m0[0] + self.m1[1] * other.m1[0] + self.m1[2] * other.m2[0],
                self.m1[0] * other.m0[1] + self.m1[1] * other.m1[1] + self.m1[2] * other.m2[1],
                self.m1[0] * other.m0[2] + self.m1[1] * other.m1[2] + self.m1[2] * other.m2[2],
            ],
            m2: [
                self.m2[0] * other.m0[0] + self.m2[1] * other.m1[0] + self.m2[2] * other.m2[0],
                self.m2[0] * other.m0[1] + self.m2[1] * other.m1[1] + self.m2[2] * other.m2[1],
                self.m2[0] * other.m0[2] + self.m2[1] * other.m1[2] + self.m2[2] * other.m2[2],
            ],
            m3: [
                self.m3[0] * other.m0[0] + self.m3[1] * other.m1[0] + self.m3[2] * other.m2[0] + other.m3[0],
                self.m3[0] * other.m0[1] + self.m3[1] * other.m1[1] + self.m3[2] * other.m2[1] + other.m3[1],
                self.m3[0] * other.m0[2] + self.m3[1] * other.m1[2] + self.m3[2] * other.m2[2] + other.m3[2],
            ],
        }
    }
}

impl Default for Matrix3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl FromStr for Matrix3D {
    type Err = Error;

    /// Parse from 3MF format string "m00 m01 m02 m10 m11 m12 m20 m21 m22 m30 m31 m32".
    fn from_str(s: &str) -> Result<Self> {
        let values: Vec<f64> = s
            .split_whitespace()
            .map(|v| v.parse::<f64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| Error::InvalidMatrix(format!("cannot parse matrix: {}", s)))?;

        if values.len() != 12 {
            return Err(Error::InvalidMatrix(format!(
                "matrix must have 12 values, got {}",
                values.len()
            )));
        }

        Ok(Self {
            m0: [values[0], values[1], values[2]],
            m1: [values[3], values[4], values[5]],
            m2: [values[6], values[7], values[8]],
            m3: [values[9], values[10], values[11]],
        })
    }
}

impl fmt::Display for Matrix3D {
    /// Convert to 3MF format string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "{} {} {} {} {} {} {} {} {} {} {} {}",
            self.m0[0], self.m0[1], self.m0[2],
            self.m1[0], self.m1[1], self.m1[2],
            self.m2[0], self.m2[1], self.m2[2],
            self.m3[0], self.m3[1], self.m3[2]
        )
    }
}

/// A collection of vertices.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vertices {
    pub vertices: Vec<Vertex>,
}

/// A 3D vertex (point).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vertex {
    /// Create a new vertex.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Calculate distance to another vertex.
    pub fn distance(&self, other: &Vertex) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate the length from origin.
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize to unit length.
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    /// Cross product.
    pub fn cross(&self, other: &Vertex) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Dot product.
    pub fn dot(&self, other: &Vertex) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Subtract two vertices.
    pub fn sub(&self, other: &Vertex) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl From<(f64, f64, f64)> for Vertex {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self { x, y, z }
    }
}

impl From<[f64; 3]> for Vertex {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self { x, y, z }
    }
}

/// A collection of triangles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Triangles {
    pub triangles: Vec<Triangle>,
}

/// A triangle face referencing three vertices by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Triangle {
    /// Index of first vertex.
    pub v1: u32,
    /// Index of second vertex.
    pub v2: u32,
    /// Index of third vertex.
    pub v3: u32,
    /// Optional property index for first vertex.
    pub p1: Option<u32>,
    /// Optional property index for second vertex.
    pub p2: Option<u32>,
    /// Optional property index for third vertex.
    pub p3: Option<u32>,
    /// Optional property group ID.
    pub pid: Option<u32>,
}

impl Triangle {
    /// Create a new triangle.
    pub fn new(v1: u32, v2: u32, v3: u32) -> Self {
        Self {
            v1,
            v2,
            v3,
            p1: None,
            p2: None,
            p3: None,
            pid: None,
        }
    }

    /// Create a new triangle with property.
    pub fn with_property(v1: u32, v2: u32, v3: u32, pid: u32, pindex: u32) -> Self {
        Self {
            v1,
            v2,
            v3,
            p1: Some(pindex),
            p2: None,
            p3: None,
            pid: Some(pid),
        }
    }

    /// Get vertex indices as a tuple.
    pub fn vertices(&self) -> (u32, u32, u32) {
        (self.v1, self.v2, self.v3)
    }
}