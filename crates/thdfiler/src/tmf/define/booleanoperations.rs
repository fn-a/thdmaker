//! Boolean Operations extension.
//!
//! This module implements the 3MF Boolean Operations Extension specification which
//! enables geometric combination operations (union, difference, intersection) for
//! creating new 3D shapes from existing objects.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};
use super::primitive::Matrix3D;

/// Boolean operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BooleanOperation {
    /// Union operation
    #[default]
    Union,
    /// Difference operation
    Difference,
    /// Intersection operation
    Intersection,
}

impl fmt::Display for BooleanOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BooleanOperation::Union => write!(f, "union"),
            BooleanOperation::Difference => write!(f, "difference"),
            BooleanOperation::Intersection => write!(f, "intersection"),
        }
    }
}

impl FromStr for BooleanOperation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "union" => Ok(BooleanOperation::Union),
            "difference" => Ok(BooleanOperation::Difference),
            "intersection" => Ok(BooleanOperation::Intersection),
            _ => Err(Error::InvalidAttribute {
                name: "operation".to_string(),
                message: format!("unknown boolean operation: {}", s),
            }),
        }
    }
}

/// Single boolean operation definition
#[derive(Debug, Clone)]
pub struct Boolean {
    /// Object ID reference for boolean operation
    pub object_id: u32,
    /// Optional 3D transformation matrix applied to object before boolean operation
    pub transform: Option<Matrix3D>,
    /// Optional path referencing object in non-root model file (only valid in root model file)
    pub path: Option<String>,
}

impl Boolean {
    /// Create new boolean operation
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            transform: None,
            path: None,
        }
    }

    /// Create boolean operation with transform
    pub fn with_transform(object_id: u32, transform: Matrix3D) -> Self {
        Self {
            object_id,
            transform: Some(transform),
            path: None,
        }
    }

    /// Create boolean operation with path
    pub fn with_path(object_id: u32, path: impl Into<String>) -> Self {
        Self {
            object_id,
            transform: None,
            path: Some(path.into()),
        }
    }

    /// Set transformation matrix
    pub fn set_transform(&mut self, transform: Matrix3D) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set path
    pub fn set_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.path = Some(path.into());
        self
    }
}

/// Boolean shape definition
#[derive(Debug, Clone)]
pub struct BooleanShape {
    /// Base object ID reference for applying boolean operations
    pub object_id: u32,
    /// Boolean operation type, defaults to union
    pub operation: BooleanOperation,
    /// Optional 3D transformation matrix applied to base object
    pub transform: Option<Matrix3D>,
    /// Optional path referencing base object in non-root model file (only valid in root model file)
    pub path: Option<String>,
    /// Boolean operation sequence
    pub booleans: Vec<Boolean>,
}

impl BooleanShape {
    /// Create new boolean shape
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            operation: BooleanOperation::Union,
            transform: None,
            path: None,
            booleans: Vec::new(),
        }
    }

    /// Create boolean shape with operation type
    pub fn with_operation(object_id: u32, operation: BooleanOperation) -> Self {
        Self {
            object_id,
            operation,
            transform: None,
            path: None,
            booleans: Vec::new(),
        }
    }

    /// Create boolean shape with transform
    pub fn with_transform(object_id: u32, transform: Matrix3D) -> Self {
        Self {
            object_id,
            operation: BooleanOperation::Union,
            transform: Some(transform),
            path: None,
            booleans: Vec::new(),
        }
    }

    /// Create boolean shape with path
    pub fn with_path(object_id: u32, path: impl Into<String>) -> Self {
        Self {
            object_id,
            operation: BooleanOperation::Union,
            transform: None,
            path: Some(path.into()),
            booleans: Vec::new(),
        }
    }

    /// Add boolean operation
    pub fn add_boolean(&mut self, boolean: Boolean) -> &mut Self {
        self.booleans.push(boolean);
        self
    }

    /// Set operation type
    pub fn set_operation(&mut self, operation: BooleanOperation) -> &mut Self {
        self.operation = operation;
        self
    }

    /// Set transformation matrix
    pub fn set_transform(&mut self, transform: Matrix3D) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set path
    pub fn set_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    /// Get boolean operation count
    pub fn boolean_count(&self) -> usize {
        self.booleans.len()
    }

    /// Validate boolean shape integrity
    pub fn validate(&self) -> Result<()> {
        // Validate base object ID is not zero
        if self.object_id == 0 {
            return Err(Error::InvalidAttribute {
                name: "objectid".to_string(),
                message: "base object ID cannot be zero".to_string(),
            });
        }

        // Validate object IDs of all boolean operations
        for boolean in self.booleans.iter() {
            if boolean.object_id == 0 {
                return Err(Error::InvalidAttribute {
                    name: "objectid".to_string(),
                    message: "boolean operation object ID cannot be zero".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for BooleanShape {
    fn default() -> Self {
        Self::new(0)
    }
}