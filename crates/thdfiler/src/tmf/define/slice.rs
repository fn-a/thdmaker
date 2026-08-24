//! Slice extension.
//!
//! This module implements the 3MF Slice Extension specification which provides
//! support for pre-sliced 3D models and layer-by-layer manufacturing instructions.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};

/// A stack of slices defining an object.
#[derive(Debug, Clone)]
pub struct SliceStack {
    /// Unique resource ID.
    pub id: u32,
    /// Starting Z level relative to build platform.
    pub z_bottom: Option<f64>,
    /// Content of the slice stack (either slices or references).
    pub content: SliceContent,
}

impl SliceStack {
    /// Create a new slice stack.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            z_bottom: None,
            content: SliceContent::Slices(Vec::new()),
        }
    }
}

/// Content of a slice stack: either explicit slices or references to another file.
#[derive(Debug, Clone)]
pub enum SliceContent {
    /// List of explicit slices.
    Slices(Vec<Slice>),
    /// List of references to slices in other files.
    SliceRefs(Vec<SliceRef>),
}

/// A single 2D slice.
#[derive(Debug, Clone)]
pub struct Slice {
    /// Z position of the top of the slice.
    pub z_top: f64,
    /// Vertices used in this slice.
    pub vertices: Vertices2D,
    /// Polygons defining the slice geometry.
    pub polygons: Vec<Polygon>,
}

impl Slice {
    /// Create a new slice.
    pub fn new(z_top: f64) -> Self {
        Self {
            z_top,
            vertices: Vertices2D::default(),
            polygons: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vertices2D {
    pub vertices: Vec<Vertex2D>,
}

/// A 2D vertex.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex2D {
    pub x: f64,
    pub y: f64,
}

impl Vertex2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A closed polygon defined by segments.
#[derive(Debug, Clone)]
pub struct Polygon {
    /// Index of the starting vertex.
    pub start_v: u32,
    /// Segments forming the polygon.
    pub segments: Vec<Segment>,
}

impl Polygon {
    /// Create a new polygon.
    pub fn new(start_v: u32) -> Self {
        Self {
            start_v,
            segments: Vec::new(),
        }
    }
}

/// A segment of a polygon.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Index of the end vertex.
    pub v2: u32,
    /// The XSD schema define p1,p2,pid use is required, it should be a bug.
    /// Optional property index for start vertex.
    pub p1: Option<u32>,
    /// Optional property index for end vertex.
    pub p2: Option<u32>,
    /// Optional property group ID.
    pub pid: Option<u32>,
}

impl Segment {
    /// Create a new segment.
    pub fn new(v2: u32) -> Self {
        Self {
            v2,
            p1: None,
            p2: None,
            pid: None,
        }
    }
}

/// A reference to a slice stack in another file.
#[derive(Debug, Clone)]
pub struct SliceRef {
    /// ID of the slice stack in the referenced file.
    pub slice_stack_id: u32,
    /// Path to the file containing the slice stack.
    pub slice_path: String,
}

/// Slice extension for resources.
#[derive(Debug, Clone, Default)]
pub struct SliceResources {
    /// Collection of slice stacks.
    pub slice_stacks: Vec<SliceStack>,
}

/// Slice extension data for Object.
#[derive(Debug, Clone, Default)]
pub struct SliceObject {
    /// ID of the slice stack.
    pub slice_stack_id: Option<u32>,
    /// Mesh resolution hint.
    pub mesh_resolution: Option<MeshResolution>,
}

/// Mesh resolution hint for objects with slice data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshResolution {
    /// Full resolution mesh (can be used to regenerate slices).
    #[default]
    FullRes,
    /// Low resolution mesh (visualization only).
    LowRes,
}

impl fmt::Display for MeshResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshResolution::FullRes => write!(f, "fullres"),
            MeshResolution::LowRes => write!(f, "lowres"),
        }
    }
}

impl FromStr for MeshResolution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "fullres" => Ok(MeshResolution::FullRes),
            "lowres" => Ok(MeshResolution::LowRes),
            _ => Err(Error::InvalidAttribute {
                name: "meshresolution".to_string(),
                message: format!("unknown mesh resolution: {}", s),
            }),
        }
    }
}