//! Triangle Set extension.
//!
//! This module implements the 3MF Triangle Set Extension specification which provides
//! logical grouping and organization capabilities for triangle meshes in 3D models.

/// A collection of triangle sets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TriangleSets {
    pub triangle_sets: Vec<TriangleSet>,
}

/// A triangle set for grouping triangles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TriangleSet {
    /// Human-readable name.
    pub name: String,
    /// Unique identifier.
    pub identifier: String,
    pub r#ref: Vec<TriangleRef>,
    pub refrange: Vec<TriangleRefRange>,
}

/// A reference to a triangle in a triangle set.
impl TriangleSet {
    /// Create a new triangle set.
    pub fn new(name: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            identifier: identifier.into(),
            r#ref: Vec::new(),
            refrange: Vec::new(),
        }
    }

    /// Add a triangle index.
    pub fn add_index(&mut self, index: usize) {
        self.r#ref.push(TriangleRef { index });
    }
    /// Add a range of triangle indices.
    pub fn add_range(&mut self, start: usize, end: usize) {
        for i in start..=end {
            self.r#ref.push(TriangleRef { index: i });
        }
        self.refrange.push(TriangleRefRange {
            start_index: start,
            end_index: end,
        });
    }
}

/// A reference to a triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TriangleRef {
    /// Resource index of the triangle.
    pub index: usize,
}

/// A range of triangle indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TriangleRefRange {
    /// Resource index of the first triangle in the range.
    pub start_index: usize,
    /// Resource index of the last triangle in the range.
    pub end_index: usize,
}