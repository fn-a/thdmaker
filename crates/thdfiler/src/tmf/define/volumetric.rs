//! Volumetric extension.
//!
//! This module implements the 3MF Volumetric Extension specification which enables
//! field-based geometric representations using volumetric data. This provides an
//! alternative to traditional mesh-based geometry for representing complex 3D objects.

use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;
use super::error::{Error, Result};
use super::primitive::*;

/// Volume data resource defining volumetric properties.
/// Volumedata MUST only be referenced by an object type "mesh" or "levelset"
#[derive(Debug, Clone)]
pub struct VolumeData {
    /// Unique resource ID.
    pub id: u32,
    /// Optional color data.
    pub color: Option<Colour>,
    /// Optional composite material data.
    pub composite: Option<Composite>,
    /// Optional property data.
    pub properties: Vec<Property>,
}

impl VolumeData {
    /// Create a new volume data.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            color: None,
            composite: None,
            properties: Vec::new(),
        }
    }

    /// Set the color data.
    pub fn set_color(&mut self, color: Colour) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the composite data.
    pub fn set_composite(&mut self, composite: Composite) -> &mut Self {
        self.composite = Some(composite);
        self
    }

    /// Add a property.
    pub fn add_property(&mut self, property: Property) -> &mut Self {
        self.properties.push(property);
        self
    }
}

/// Color data defining color throughout the volume.
#[derive(Debug, Clone)]
pub struct Colour {
    /// Reference to the function providing color.
    pub function_id: u32,
    /// Name of the function output to use for color.
    pub channel: String,
    /// Transformation from object to function coordinates.
    pub transform: Option<Matrix3D>,
    /// Minimum feature size hint.
    pub min_feature_size: f64,
    /// Fallback value if function output is undefined.
    pub fallback_value: f64,
}

impl Colour {
    /// Create a new color data.
    pub fn new(function_id: u32, channel: impl Into<String>) -> Self {
        Self {
            function_id,
            channel: channel.into(),
            transform: None,
            min_feature_size: 0.0,
            fallback_value: 0.0,
        }
    }
}

/// Composite material data defining material mixing ratios.
#[derive(Debug, Clone)]
pub struct Composite {
    /// Reference to base materials resource.
    pub base_material_id: u32,
    /// Material mappings for each base material.
    pub material_mappings: Vec<MaterialMapping>,
}

impl Composite {
    /// Create a new composite.
    pub fn new(base_material_id: u32) -> Self {
        Self {
            base_material_id,
            material_mappings: Vec::new(),
        }
    }

    /// Add a material mapping.
    pub fn add_mapping(&mut self, mapping: MaterialMapping) -> &mut Self {
        self.material_mappings.push(mapping);
        self
    }
}

/// Material mapping for a single base material.
#[derive(Debug, Clone)]
pub struct MaterialMapping {
    /// Reference to the function providing mixing contribution.
    pub function_id: u32,
    /// Name of the function output to use.
    pub channel: String,
    /// Transformation from object to function coordinates.
    pub transform: Option<Matrix3D>,
    /// Minimum feature size hint.
    pub min_feature_size: f64,
    /// Fallback value if function output is undefined.
    pub fallback_value: f64,
}

impl MaterialMapping {
    /// Create a new material mapping.
    pub fn new(function_id: u32, channel: impl Into<String>) -> Self {
        Self {
            function_id,
            transform: None,
            channel: channel.into(),
            min_feature_size: 0.0,
            fallback_value: 0.0,
        }
    }
}

/// Property data defining arbitrary properties throughout the volume.
#[derive(Debug, Clone)]
pub struct Property {
    /// Reference to the function providing the property value.
    pub function_id: u32,
    /// Name of the function output to use.
    pub channel: String,
    /// Transformation from object to function coordinates.
    pub transform: Option<Matrix3D>,
    /// Property name (with namespace).
    pub name: String,
    /// Whether this property is required.
    pub required: bool,
    /// Minimum feature size hint.
    /// The specification content not stated but declared in the XSD.
    pub min_feature_size: f64,
    /// Fallback value if function output is undefined.
    pub fallback_value: f64,
}

impl Property {
    /// Create a new property.
    pub fn new(function_id: u32, channel: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            function_id,
            transform: None,
            channel: channel.into(),
            name: name.into(),
            required: false,
            min_feature_size: 0.0,
            fallback_value: 0.0,
        }
    }
}

/// A 3D image resource containing voxel data.
#[derive(Debug, Clone)]
pub struct Image3D {
    /// Unique resource ID.
    pub id: u32,
    /// Name for annotation purposes.
    pub name: Option<String>,
    /// The 3D image content.
    pub content: Image3DContent,
}

impl Image3D {
    /// Create a new 3D image.
    pub fn new(id: u32, content: Image3DContent) -> Self {
        Self {
            id,
            name: None,
            content,
        }
    }

    /// Set the name.
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }
}

/// 3D image content types.
#[derive(Debug, Clone)]
pub enum Image3DContent {
    /// Stack of 2D image sheets.
    ImageStack(ImageStack),
}

/// A stack of 2D images representing a 3D volume.
#[derive(Debug, Clone)]
pub struct ImageStack {
    /// Number of pixel rows.
    pub row_count: u32,
    /// Number of pixel columns.
    pub column_count: u32,
    /// Number of image sheets.
    pub sheet_count: u32,
    /// The image sheets (paths to PNG files).
    pub image_sheets: Vec<ImageSheet>,
}

impl ImageStack {
    /// Create a new image stack.
    pub fn new(row_count: u32, column_count: u32, sheet_count: u32) -> Self {
        Self {
            row_count,
            column_count,
            sheet_count,
            image_sheets: Vec::new(),
        }
    }

    /// Add an image sheet.
    pub fn add_sheet(&mut self, path: impl Into<String>) -> &mut Self {
        self.image_sheets.push(ImageSheet { path: path.into() });
        self
    }

    /// Validate the image stack.
    pub fn validate(&self) -> Result<()> {
        // Check limits
        if self.row_count > 1024 {
            return Err(Error::InvalidStructure(format!(
                "rowcount {} exceeds maximum of 1024",
                self.row_count
            )));
        }
        if self.column_count > 1024 {
            return Err(Error::InvalidStructure(format!(
                "columncount {} exceeds maximum of 1024",
                self.column_count
            )));
        }
        if self.sheet_count > 1024 {
            return Err(Error::InvalidStructure(format!(
                "sheetcount {} exceeds maximum of 1024",
                self.sheet_count
            )));
        }

        // Check total voxels
        let total_voxels = self.row_count as u64 * self.column_count as u64 * self.sheet_count as u64;
        if total_voxels > 1024u64.pow(5) {
            return Err(Error::InvalidStructure(format!(
                "total voxels {} exceeds maximum of 1024^5",
                total_voxels
            )));
        }

        // Check sheet count matches
        if self.image_sheets.len() != self.sheet_count as usize {
            return Err(Error::InvalidStructure(format!(
                "expected {} sheets, got {}",
                self.sheet_count,
                self.image_sheets.len()
            )));
        }

        Ok(())
    }
}

/// A single 2D image sheet in the stack.
#[derive(Debug, Clone)]
pub struct ImageSheet {
    /// Path to the image file in the package.
    pub path: String,
}

/// A function resource that can be evaluated at any point in space.
#[derive(Debug, Clone)]
pub struct Function {
    /// Unique resource ID.
    pub id: u32,
    /// Display name for annotation purposes.
    pub display_name: Option<String>,
    /// The function type.
    pub r#type: FunctionType,
}

impl Function {
    /// Create a new function.
    pub fn new(id: u32, r#type: FunctionType) -> Self {
        Self {
            id,
            display_name: None,
            r#type,
        }
    }

    /// Set the display name.
    pub fn set_display_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.display_name = Some(name.into());
        self
    }
}

/// Function types supported by the volumetric extension.
#[derive(Debug, Clone)]
pub enum FunctionType {
    /// Function from 3D image.
    FunctionFromImage3D(FunctionFromImage3D),
    /// Private extension function.
    PrivateExtensionFunction(PrivateExtensionFunction),
}

/// A function that samples values from a 3D image.
#[derive(Debug, Clone)]
pub struct FunctionFromImage3D {
    /// Reference to the image3d resource.
    pub image_3d_id: u32,
    /// Filter method for interpolation.
    pub filter: Filter,
    /// Numerical offset for the sampled values.
    pub value_offset: f64,
    /// Numerical scaling of the sampled values.
    pub value_scale: f64,
    /// Tile style for U direction.
    pub tile_style_u: TileStyle,
    /// Tile style for V direction.
    pub tile_style_v: TileStyle,
    /// Tile style for W direction.
    pub tile_style_w: TileStyle,
}

impl FunctionFromImage3D {
    /// Create a new function from image3d.
    pub fn new(image3d_id: u32) -> Self {
        Self {
            image_3d_id: image3d_id,
            value_offset: 0.0,
            value_scale: 1.0,
            filter: Filter::Linear,
            tile_style_u: TileStyle::Wrap,
            tile_style_v: TileStyle::Wrap,
            tile_style_w: TileStyle::Wrap,
        }
    }
}

/// Filter method for interpolation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Filter {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Nearest neighbor interpolation.
    Nearest,
}


impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Filter::Linear => write!(f, "linear"),
            Filter::Nearest => write!(f, "nearest"),
        }
    }
}

impl FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "linear" => Ok(Filter::Linear),
            "nearest" => Ok(Filter::Nearest),
            _ => Err(Error::InvalidAttribute {
                name: "filter".to_string(),
                message: format!("unknown filter type: {}", s),
            }),
        }
    }
}

/// Tile style for texture coordinates outside [0,1].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TileStyle {
    /// Wrap (periodic).
    #[default]
    Wrap,
    /// Mirror (reflect).
    Mirror,
    /// Clamp to [0,1].
    Clamp,
}

impl fmt::Display for TileStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileStyle::Wrap => write!(f, "wrap"),
            TileStyle::Mirror => write!(f, "mirror"),
            TileStyle::Clamp => write!(f, "clamp"),
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
            _ => Err(Error::InvalidAttribute {
                name: "tilestyleu/tilestylev/tilestylew".to_string(),
                message: format!("unknown tile style: {}", s),
            }),
        }
    }
}

/// Private extension function.
/// Owned custom namespace(xmlns attribute)
#[derive(Debug, Clone, Default)]
pub struct PrivateExtensionFunction {
    pub attributes: HashMap<String, String>,
    pub content: Option<String>,
}

/// LevelSet element defining shape using a level set function.
#[derive(Debug, Clone)]
pub struct LevelSet {
    /// Reference to the function providing the level set.
    pub function_id: u32,
    /// Name of the function output to use as level set.
    pub channel: String,
    /// Transformation from object to function coordinates.
    pub transform: Option<Matrix3D>,
    /// Minimum feature size hint.
    pub min_feature_size: f64,
    /// Whether to use only the mesh bounding box.
    pub mesh_bbox_only: bool,
    /// Fallback value if function output is undefined.
    pub fallback_value: f64,
    /// Reference to mesh defining evaluation domain.
    pub mesh_id: u32,
    /// Optional reference to volume data, apply volume data to mesh.
    pub volume_id: Option<u32>,
}

impl LevelSet {
    /// Create a new level set.
    pub fn new(function_id: u32, channel: impl Into<String>, mesh_id: u32) -> Self {
        Self {
            function_id,
            channel: channel.into(),
            transform: None,
            min_feature_size: 0.0,
            mesh_id,
            mesh_bbox_only: false,
            fallback_value: 0.0,
            volume_id: None,
        }
    }
}

/// Volumetric resources containing functions, 3D images, and volume data.
#[derive(Debug, Clone, Default)]
pub struct VolumetricResources {
    /// Volume data resources.
    pub volume_datas: Vec<VolumeData>,
    /// 3D image resources.
    pub image_3ds: Vec<Image3D>,
    /// Function resources.
    pub functions: Vec<Function>,
}

impl VolumetricResources {
    /// Create a new volumetric resources collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function and return its ID.
    pub fn add_function(&mut self, function: Function) -> u32 {
        let id = function.id;
        self.functions.push(function);
        id
    }

    /// Add an image3d and return its ID.
    pub fn add_image(&mut self, image3d: Image3D) -> u32 {
        let id = image3d.id;
        self.image_3ds.push(image3d);
        id
    }

    /// Add a volumedata and return its ID.
    pub fn add_volume(&mut self, volumedata: VolumeData) -> u32 {
        let id = volumedata.id;
        self.volume_datas.push(volumedata);
        id
    }

    /// Get a function by ID.
    pub fn get_function(&self, id: u32) -> Option<&Function> {
        self.functions.iter().find(|f| f.id == id)
    }

    /// Get an image3d by ID.
    pub fn get_image(&self, id: u32) -> Option<&Image3D> {
        self.image_3ds.iter().find(|i| i.id == id)
    }

    /// Get a volumedata by ID.
    pub fn get_volume(&self, id: u32) -> Option<&VolumeData> {
        self.volume_datas.iter().find(|v| v.id == id)
    }
}