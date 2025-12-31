//! Production extension.
//!
//! This module implements the 3MF Production Extension specification which adds
//! support for uniquely identifying build components and managing multi-file 3MF
//! packages for high-volume manufacturing environments.

use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use super::error::{Error, Result};

/// Production extension data for item elements.
#[derive(Debug, Clone)]
pub struct ProductionItem {
    /// Unique identifier for the item
    pub uuid: Uuid,
    /// Optional path to the model file containing the object
    pub path: Option<String>,
}

impl ProductionItem {
    /// Create a new production item with a UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            path: None,
        })
    }

    /// Create a production item with a generated UUID.
    pub fn with_uuid() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            path: None,
        }
    }

    /// Create a production item with a UUID and path.
    pub fn with_path(uuid: &str, path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            path: Some(path.into()),
        })
    }

    /// Set the UUID.
    pub fn set_uuid(&mut self, uuid: &str) -> Result<()> {
        self.uuid = Uuid::parse_str(uuid)?;
        Ok(())
    }

    /// Set the path.
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }
}

/// Production extension data for component elements.
#[derive(Debug, Clone)]
pub struct ProductionComponent {
    /// Unique identifier for the component
    pub uuid: Uuid,
    /// Optional path to the model file containing the object
    pub path: Option<String>,
}

impl ProductionComponent {
    /// Create a new production component with a UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            path: None,
        })
    }

    /// Create a production component with a generated UUID.
    pub fn with_uuid() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            path: None,
        }
    }

    /// Create a production component with a UUID and path.
    pub fn with_path(uuid: &str, path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            path: Some(path.into()),
        })
    }

    /// Set the UUID.
    pub fn set_uuid(&mut self, uuid: &str) -> Result<()> {
        self.uuid = Uuid::parse_str(uuid)?;
        Ok(())
    }

    /// Set the path.
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }
}

/// Production extension data for object elements.
#[derive(Debug, Clone)]
pub struct ProductionObject {
    /// Unique identifier for the object
    pub uuid: Uuid,
    /// Model resolution for alternative representations
    pub model_resolution: ModelResolution,
}

impl ProductionObject {
    /// Create a new production object with a UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            model_resolution: ModelResolution::default(),
        })
    }

    /// Create a production object with a generated UUID.
    pub fn with_uuid() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            model_resolution: ModelResolution::default(),
        }
    }

    /// Create a production object with a UUID and model resolution.
    pub fn with_resolution(uuid: &str, resolution: ModelResolution) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
            model_resolution: resolution,
        })
    }

    /// Set the model resolution.
    pub fn set_resolution(&mut self, resolution: ModelResolution) {
        self.model_resolution = resolution;
    }
}

/// Model resolution type for alternative representations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ModelResolution {
    /// Full resolution model intended for printing
    #[default]
    FullRes,
    /// Low resolution model for visualization
    LowRes,
    /// Obfuscated model with sensitive areas hidden
    Obfuscated,
}

impl fmt::Display for ModelResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelResolution::FullRes => write!(f, "fullres"),
            ModelResolution::LowRes => write!(f, "lowres"),
            ModelResolution::Obfuscated => write!(f, "obfuscated"),
        }
    }
}

impl FromStr for ModelResolution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "fullres" => Ok(ModelResolution::FullRes),
            "lowres" => Ok(ModelResolution::LowRes),
            "obfuscated" => Ok(ModelResolution::Obfuscated),
            _ => Err(Error::InvalidAttribute {
                name: "modelresolution".to_string(),
                message: format!("unknown model resolution: {}", s),
            }),
        }
    }
}

/// Collection of alternative representations for an object.
#[derive(Debug, Clone, Default)]
pub struct Alternatives {
    /// List of alternative representations
    pub alternatives: Vec<Alternative>,
}

impl Alternatives {
    /// Create a new empty alternatives collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an alternative representation.
    pub fn add(&mut self, alternative: Alternative) {
        self.alternatives.push(alternative);
    }

    /// Get the number of alternatives.
    pub fn len(&self) -> usize {
        self.alternatives.len()
    }

    /// Check if there are any alternatives.
    pub fn is_empty(&self) -> bool {
        self.alternatives.is_empty()
    }
}

/// An alternative representation of an object.
#[derive(Debug, Clone)]
pub struct Alternative {
    /// Object ID in the referenced model file
    pub object_id: u32,
    /// Unique identifier for the alternative
    pub uuid: Uuid,
    /// Optional path to the model file containing the alternative
    pub path: Option<String>,
    /// Model resolution for this alternative
    pub model_resolution: ModelResolution,
}

impl Alternative {
    /// Create a new alternative representation.
    pub fn new(object_id: u32, uuid: &str) -> Result<Self> {
        Ok(Self {
            object_id,
            uuid: Uuid::parse_str(uuid)?,
            path: None,
            model_resolution: ModelResolution::default(),
        })
    }

    /// Create an alternative with path.
    pub fn with_path(object_id: u32, uuid: &str, path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            object_id,
            uuid: Uuid::parse_str(uuid)?,
            path: Some(path.into()),
            model_resolution: ModelResolution::default(),
        })
    }

    /// Create an alternative with resolution.
    pub fn with_resolution(object_id: u32, uuid: &str, resolution: ModelResolution) -> Result<Self> {
        Ok(Self {
            object_id,
            uuid: Uuid::parse_str(uuid)?,
            path: None,
            model_resolution: resolution,
        })
    }

    /// Set the path.
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }

    /// Set the model resolution.
    pub fn set_resolution(&mut self, resolution: ModelResolution) {
        self.model_resolution = resolution;
    }
}

/// Production extension data for build elements.
#[derive(Debug, Clone)]
pub struct ProductionBuild {
    /// Unique identifier for the build
    pub uuid: Uuid,
}

impl ProductionBuild {
    /// Create a new production build with a UUID.
    pub fn new(uuid: &str) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::parse_str(uuid)?,
        })
    }

    /// Create a production build with a generated UUID.
    pub fn with_uuid() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}