//! Displacement extension.
//!
//! This module implements the 3MF Displacement Extension specification which enables
//! surface displacement mapping for adding detailed geometric features to 3D models
//! through texture-based displacement techniques.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};

/// Displacement 2D texture resource
#[derive(Debug, Clone)]
pub struct Displacement2D {
    /// Displacement 2D resource ID
    pub id: u32,
    /// Texture image file path
    pub path: String,
    /// Color channel used
    pub channel: ChannelName,
    /// U-axis tiling style
    pub tile_style_u: TileStyle,
    /// V-axis tiling style
    pub tile_style_v: TileStyle,
    /// Texture filter
    pub filter: Filter,
}

impl Displacement2D {
    /// Create new displacement 2D resource
    pub fn new(id: u32, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            channel: ChannelName::default(),
            tile_style_u: TileStyle::default(),
            tile_style_v: TileStyle::default(),
            filter: Filter::default(),
        }
    }

    /// Set color channel
    pub fn set_channel(&mut self, channel: ChannelName) -> &mut Self {
        self.channel = channel;
        self
    }

    /// Set U-axis tiling style
    pub fn set_tile_style_u(&mut self, tile_style: TileStyle) -> &mut Self {
        self.tile_style_u = tile_style;
        self
    }

    /// Set V-axis tiling style
    pub fn set_tile_style_v(&mut self, tile_style: TileStyle) -> &mut Self {
        self.tile_style_v = tile_style;
        self
    }

    /// Set texture filter
    pub fn set_filter(&mut self, filter: Filter) -> &mut Self {
        self.filter = filter;
        self
    }
}

/// Displacement texture channel name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelName {
    /// Red channel
    R,
    /// Green channel
    G,
    /// Blue channel
    B,
    /// Alpha channel
    A,
}

impl Default for ChannelName {
    fn default() -> Self {
        ChannelName::G
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelName::R => write!(f, "R"),
            ChannelName::G => write!(f, "G"),
            ChannelName::B => write!(f, "B"),
            ChannelName::A => write!(f, "A"),
        }
    }
}

impl FromStr for ChannelName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "R" => Ok(ChannelName::R),
            "G" => Ok(ChannelName::G),
            "B" => Ok(ChannelName::B),
            "A" => Ok(ChannelName::A),
            _ => Err(Error::InvalidAttribute {
                name: "channel".to_string(),
                message: format!("unknown channel name: {}", s),
            }),
        }
    }
}

/// Texture tiling style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStyle {
    /// Repeat tiling
    Wrap,
    /// Mirror tiling
    Mirror,
    /// Clamp
    Clamp,
    /// No tiling
    None,
}

impl Default for TileStyle {
    fn default() -> Self {
        TileStyle::Wrap
    }
}

impl fmt::Display for TileStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileStyle::Wrap => write!(f, "wrap"),
            TileStyle::Mirror => write!(f, "mirror"),
            TileStyle::Clamp => write!(f, "clamp"),
            TileStyle::None => write!(f, "none"),
        }
    }
}

impl FromStr for TileStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "wrap" => Ok(TileStyle::Wrap),
            "mirror" => Ok(TileStyle::Mirror),
            "clamp" => Ok(TileStyle::Clamp),
            "none" => Ok(TileStyle::None),
            _ => Err(Error::InvalidAttribute {
                name: "tilestyleu/tilestylev".to_string(),
                message: format!("unknown tile style: {}", s),
            }),
        }
    }
}

/// Texture filter type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Auto select best filter
    Auto,
    /// Linear interpolation
    Linear,
    /// Nearest neighbor interpolation
    Nearest,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::Auto
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Filter::Auto => write!(f, "auto"),
            Filter::Linear => write!(f, "linear"),
            Filter::Nearest => write!(f, "nearest"),
        }
    }
}

impl FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Filter::Auto),
            "linear" => Ok(Filter::Linear),
            "nearest" => Ok(Filter::Nearest),
            _ => Err(Error::InvalidAttribute {
                name: "filter".to_string(),
                message: format!("unknown filter type: {}", s),
            }),
        }
    }
}

/// Normalized vector group
#[derive(Debug, Clone)]
pub struct NormVectorGroup {
    /// Normalized vector group ID
    pub id: u32,
    /// Normalized vector list
    pub norm_vectors: Vec<NormVector>,
}

impl NormVectorGroup {
    /// Create new normalized vector group
    pub fn new(id: u32) -> Self {
        Self {
            id,
            norm_vectors: Vec::new(),
        }
    }

    /// Add normalized vector and return its index
    pub fn add_vector(&mut self, vector: NormVector) -> usize {
        let index = self.norm_vectors.len();
        self.norm_vectors.push(vector);
        index
    }

    /// Add vector (by coordinates)
    pub fn add_vector_coords(&mut self, x: f64, y: f64, z: f64) -> usize {
        self.add_vector(NormVector::new(x, y, z))
    }

    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.norm_vectors.len()
    }
}

/// Normalized displacement vector
#[derive(Debug, Clone, Copy)]
pub struct NormVector {
    /// X component
    pub x: f64,
    /// Y component
    pub y: f64,
    /// Z component
    pub z: f64,
}

impl NormVector {
    /// Create new normalized vector
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Calculate vector length
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize vector
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
}

/// 2D displacement group
#[derive(Debug, Clone)]
pub struct Disp2DGroup {
    /// 唯一资源ID
    pub id: u32,
    /// Displacement texture resource ID
    pub disp_id: u32,
    /// Normalized vector group resource ID
    pub n_id: u32,
    /// Displacement amplitude
    pub height: f64,
    /// Displacement offset
    pub offset: f64,
    /// Displacement coordinate list
    pub disp_2d_coords: Vec<Disp2DCoord>,
}

impl Disp2DGroup {
    /// Create new 2D displacement group
    pub fn new(id: u32, disp_id: u32, n_id: u32, height: f64) -> Self {
        Self {
            id,
            disp_id,
            n_id,
            height,
            offset: 0.0,
            disp_2d_coords: Vec::new(),
        }
    }

    /// Set displacement offset
    pub fn set_offset(&mut self, offset: f64) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Add displacement coordinate
    pub fn add_coord(&mut self, coord: Disp2DCoord) -> usize {
        let index = self.disp_2d_coords.len();
        self.disp_2d_coords.push(coord);
        index
    }

    /// Get coordinate count
    pub fn coord_count(&self) -> usize {
        self.disp_2d_coords.len()
    }
}

/// 2D displacement coordinate
#[derive(Debug, Clone, Copy)]
pub struct Disp2DCoord {
    /// U coordinate
    pub u: f64,
    /// V coordinate
    pub v: f64,
    /// Normalized vector index
    pub n: u32,
    /// Displacement factor
    pub f: f64,
}

impl Disp2DCoord {
    /// Create new 2D displacement coordinate
    pub fn new(u: f64, v: f64, n: u32) -> Self {
        Self {
            u,
            v,
            n,
            f: 1.0,
        }
    }

    /// Create 2D displacement coordinate with factor
    pub fn with_factor(u: f64, v: f64, n: u32, f: f64) -> Self {
        Self { u, v, n, f }
    }
}

/// Displacement mesh
#[derive(Debug, Clone)]
pub struct DisplacementMesh {
    /// Vertex collection
    pub vertices: DispVertices,
    /// Triangle collection
    pub triangles: DispTriangles,
}

impl DisplacementMesh {
    /// Create new displacement mesh
    pub fn new() -> Self {
        Self {
            vertices: DispVertices::new(),
            triangles: DispTriangles::new(),
        }
    }

    /// Set vertex collection
    pub fn set_vertices(&mut self, vertices: Vec<DispVertex>) -> &mut Self {
        self.vertices.vertices = vertices;
        self
    }

    /// Set triangle collection
    pub fn set_triangles(&mut self, triangles: Vec<DispTriangle>) -> &mut Self {
        self.triangles.triangles = triangles;
        self
    }

    /// Add vertex by coordinates and return its index
    pub fn add_vertex(&mut self, x: f64, y: f64, z: f64) -> u32 {
        self.vertices.add_vertex_coords(x, y, z)
    }

    /// Add triangle by indices and return its index
    pub fn add_triangle(&mut self, v1: u32, v2: u32, v3: u32) -> u32 {
        self.triangles.add_triangle_indices(v1, v2, v3)
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.vertex_count()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.triangles.triangle_count()
    }
}

impl Default for DisplacementMesh {
    fn default() -> Self {
        Self::new()
    }
}


/// Displacement mesh vertex collection, extended from Vertices structure
#[derive(Debug, Clone)]
pub struct DispVertices {
    pub vertices: Vec<DispVertex>,
}

impl DispVertices {
    /// Create new vertex collection
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    /// Add vertex by coordinates and return its index
    pub fn add_vertex(&mut self, vertex: DispVertex) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vertex);
        index
    }

    /// Add vertex by coordinates
    pub fn add_vertex_coords(&mut self, x: f64, y: f64, z: f64) -> u32 {
        self.add_vertex(DispVertex::new(x, y, z))
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

impl Default for DispVertices {
    fn default() -> Self {
        Self::new()
    }
}

/// Displacement mesh vertex, extended from Vertex structure
#[derive(Debug, Clone, Copy)]
pub struct DispVertex {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate
    pub z: f64,
}

impl DispVertex {
    /// Create new displacement mesh vertex
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Displacement mesh triangle collection, extended from Triangles structure
#[derive(Debug, Clone)]
pub struct DispTriangles {
    pub triangles: Vec<DispTriangle>,
    /// Optional default displacement group ID for triangle collection
    pub did: Option<u32>,
}

impl DispTriangles {
    /// Create new triangle collection
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
            did: None,
        }
    }

    /// Create triangle collection with default displacement group
    pub fn with_did(did: u32) -> Self {
        Self {
            triangles: Vec::new(),
            did: Some(did),
        }
    }

    /// Set default displacement group ID
    pub fn set_did(&mut self, did: u32) -> &mut Self {
        self.did = Some(did);
        self
    }

    /// Get default displacement group ID
    pub fn get_did(&self) -> Option<u32> {
        self.did
    }

    /// Add triangle and return its index
    pub fn add_triangle(&mut self, triangle: DispTriangle) -> u32 {
        let index = self.triangles.len() as u32;
        self.triangles.push(triangle);
        index
    }

    /// Add triangle by vertex indices
    pub fn add_triangle_indices(&mut self, v1: u32, v2: u32, v3: u32) -> u32 {
        self.add_triangle(DispTriangle::new(v1, v2, v3))
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}

impl Default for DispTriangles {
    fn default() -> Self {
        Self::new()
    }
}

/// Displacement mesh triangle, extended from Triangle structure
#[derive(Debug, Clone, Copy)]
pub struct DispTriangle {
    /// First vertex index
    pub v1: u32,
    /// Second vertex index
    pub v2: u32,
    /// Third vertex index
    pub v3: u32,
    /// Optional first vertex displacement coordinate index
    pub d1: Option<u32>,
    /// Optional second vertex displacement coordinate index
    pub d2: Option<u32>,
    /// Optional third vertex displacement coordinate index
    pub d3: Option<u32>,
    /// Displacement group ID
    pub did: Option<u32>,
}

impl DispTriangle {
    /// Create new displacement mesh triangle
    pub fn new(v1: u32, v2: u32, v3: u32) -> Self {
        Self {
            v1,
            v2,
            v3,
            d1: None,
            d2: None,
            d3: None,
            did: None,
        }
    }

    /// Create triangle with displacement
    pub fn with_did(v1: u32, v2: u32, v3: u32, did: u32) -> Self {
        Self {
            v1,
            v2,
            v3,
            d1: None,
            d2: None,
            d3: None,
            did: Some(did),
        }
    }

    /// Set displacement group ID
    pub fn set_did(&mut self, did: u32) -> &mut Self {
        self.did = Some(did);
        self
    }

    /// Set displacement coordinate indices
    pub fn set_indices(&mut self, d1: u32, d2: u32, d3: u32) -> &mut Self {
        self.d1 = Some(d1);
        self.d2 = Some(d2);
        self.d3 = Some(d3);
        self
    }

    /// Get vertex indices
    pub fn vertices(&self) -> (u32, u32, u32) {
        (self.v1, self.v2, self.v3)
    }
}

/// Collection of all displacement extension resources.
#[derive(Debug, Clone, Default)]
pub struct DisplacementResources {
    /// Optional displacement 2D texture groups.
    pub displacement_2ds: Vec<Displacement2D>,
    /// Optional displacement normal vector groups.
    pub norm_vector_groups: Vec<NormVectorGroup>,
    /// Optional displacement groups.
    pub disp_2d_groups: Vec<Disp2DGroup>,
}

impl DisplacementResources {
    /// Create a new empty displacement resources collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a displacement 2D texture.
    pub fn add_displacement_2d(&mut self, disp_2d: Displacement2D) {
        self.displacement_2ds.push(disp_2d);
    }

    /// Add a normal vector group.
    pub fn add_norm_vector_group(&mut self, norm_vector_group: NormVectorGroup) {
        self.norm_vector_groups.push(norm_vector_group);
    }

    /// Add a displacement 2D group.
    pub fn add_disp_2d_group(&mut self, disp_2d_group: Disp2DGroup) {
        self.disp_2d_groups.push(disp_2d_group);
    }
}