use std::iter;
use std::fmt::{Display, Formatter, Result};

/// Triangle mesh.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// STL file format.
    pub format: Format,
    /// Name of the mesh, ascii format only.
    pub name: Option<String>,
    /// Start index of the mesh in the STL file.
    pub start: usize,
    /// Mesh all triangle vertices for indices.
    pub vertices: Vec<Vertex>,
    /// Mesh triangle vertex indices.
    pub indices: Vec<[usize; 3]>,
    /// Mesh triangle normals.
    pub normals: Vec<Vertex>,
    /// Header of the mesh, binary format only.
    pub header: Option<[u8; 80]>,
    /// Color of the mesh, binary format only.
    pub color: Option<Color>,
    /// Mesh triangle colors, binary format only.
    pub colors: Option<Vec<Color>>,
    /// Mesh triangle attributes, binary format only.
    pub attrs: Option<Vec<u16>>,
}

impl Mesh {
    pub fn triangles(&self) -> Vec<Triangle> {
        self.indices.iter().enumerate().map(|(i, v)| Triangle {
            normal: self.normals[i],
            vertices: [self.vertices[v[0]], self.vertices[v[1]], self.vertices[v[2]]],
        }).collect()
    }

    pub fn get_positions(&self) -> Vec<[f32; 3]> {
        self.vertices.iter().map(|v| {
            // Convert Z-up coordinates to Y-up coordinates
            // Z-up: (x, y, z) -> Y-up: (x, z, -y)
            [v.x, v.z, -v.y]
        }).collect()
    }

    pub fn get_normals(&self) -> Vec<[f32; 3]> {
        self.normals.iter().flat_map(|v| {
            // Convert Z-up coordinates to Y-up coordinates
            // Z-up: (x, y, z) -> Y-up: (x, z, -y)
            iter::repeat_n([v.x, v.z, -v.y], 3)
        }).collect()
    }

    pub fn get_indices(&self) -> Vec<u32> {
        self.indices.iter().flat_map(|v| [v[0] as u32, v[1] as u32, v[2] as u32]).collect()
    }
}

/// STL file format.
#[derive(Debug, Clone, Default)]
pub enum Format {
    #[default]
    Ascii,
    Binary,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Format::Ascii => write!(f, "ascii"),
            Format::Binary => write!(f, "binary"),
        }
    }
}

/// A triangle in a 3D mesh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    /// Normal vector of the Triangle.
    pub normal: Vertex,
    /// The three vertices of the Triangle.
    pub vertices: [Vertex; 3],
}

/// A vertex in a 3D mesh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    /// X coordinate of the Vertex.
    pub x: f32,
    /// Y coordinate of the Vertex.
    pub y: f32,
    /// Z coordinate of the Vertex.
    pub z: f32,
}

impl From<&[f32; 3]> for Vertex {
    fn from(xyz: &[f32; 3]) -> Self {
        Self {
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
        }
    }
}

impl From<&Vertex> for [f32; 3] {
    fn from(v: &Vertex) -> Self {
        [v.x, v.y, v.z]
    }
}

/// Color in RGBA format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red (0-255)
    pub r: u8,
    /// Green (0-255)
    pub g: u8,
    /// Blue (0-255)
    pub b: u8,
    /// Alpha (0-255, 255 = fully opaque)
    pub a: u8,
}

/// Convert a 4-byte array to a Color.
impl From<&[u8; 4]> for Color {
    fn from(rgba: &[u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }
}