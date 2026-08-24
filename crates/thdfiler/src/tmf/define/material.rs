//! Materials and Properties Extension.
//! 
//! This module implements the 3MF Materials and Properties Extension specification
//! which adds support for full color and multi-material definitions.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};
use super::primitive::Color;

/// A 2D texture resource.
#[derive(Debug, Clone, Default)]
pub struct Texture2D {
    /// Unique resource ID.
    pub id: u32,
    /// Path to the texture data.
    pub path: String,
    /// Content type of the texture.
    pub content_type: ContentType,
    /// Tile style for U coordinate.
    pub tile_style_u: TileStyle,
    /// Tile style for V coordinate.
    pub tile_style_v: TileStyle,
    /// Texture filter type.
    pub filter: Filter,
}

impl Texture2D {
    /// Create a new texture.
    pub fn new(id: u32, path: impl Into<String>, content_type: ContentType) -> Self {
        Self {
            id,
            path: path.into(),
            content_type,
            tile_style_u: TileStyle::Wrap,
            tile_style_v: TileStyle::Wrap,
            filter: Filter::Auto,
        }
    }

    /// Set the tile style for U coordinate.
    pub fn set_tile_style_u(&mut self, tile_style: TileStyle) -> &mut Self {
        self.tile_style_u = tile_style;
        self
    }

    /// Set the tile style for V coordinate.
    pub fn set_tile_style_v(&mut self, tile_style: TileStyle) -> &mut Self {
        self.tile_style_v = tile_style;
        self
    }

    /// Set the content type of the texture.
    pub fn set_content_type(&mut self, content_type: ContentType) -> &mut Self {
        self.content_type = content_type;
        self
    }

    /// Set the filter type.
    pub fn set_filter(&mut self, filter: Filter) -> &mut Self {
        self.filter = filter;
        self
    }
}

/// Content type of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentType {
    /// JPEG image.
    #[default]
    ImageJpeg,
    /// PNG image.
    ImagePng,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::ImageJpeg => write!(f, "image/jpeg"),
            ContentType::ImagePng => write!(f, "image/png"),
        }
    }
}

impl FromStr for ContentType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "image/jpeg" => Ok(ContentType::ImageJpeg),
            "image/png" => Ok(ContentType::ImagePng),
            _ => Err(Error::InvalidAttribute {
                name: "contenttype".to_string(),
                message: format!("unknown content type: {}", s),
            }),
        }
    }
}

/// Tile style for texture coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileStyle {
    /// Repeat the texture in both directions.
    #[default]
    Wrap,
    /// Mirror the texture when repeating.
    Mirror,
    /// Clamp to edge pixels.
    Clamp,
    /// Clamp to edge pixels with transparent alpha.
    None,
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

/// Texture filter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Use highest quality filter available.
    #[default]
    Auto,
    /// Bilinear interpolation.
    Linear,
    /// Nearest neighbor interpolation.
    Nearest,
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
                message: format!("unknown filter: {}", s),
            }),
        }
    }
}

/// A group of colors.
#[derive(Debug, Clone, Default)]
pub struct ColorGroup {
    /// Unique resource ID.
    pub id: u32,
    /// Reference to display properties.
    pub display_properties_id: Option<u32>,
    /// List of colors.
    pub colors: Vec<ColorType>,
}

impl ColorGroup {
    /// Create a new color group.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            display_properties_id: None,
            colors: Vec::new(),
        }
    }

    /// Add a color to the group.
    pub fn add_color(&mut self, color: Color) -> usize {
        let index = self.colors.len();
        self.colors.push(ColorType { color });
        index
    }

    /// Set the display properties ID.
    pub fn set_display_properties(&mut self, display_properties_id: u32) -> &mut Self {
        self.display_properties_id = Some(display_properties_id);
        self
    }
}

/// A single color definition.
#[derive(Debug, Clone, Default)]
pub struct ColorType {
    /// The color value.
    pub color: Color,
}

impl ColorType {
    /// Create a new color definition.
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

/// A 2D texture coordinate group.
#[derive(Debug, Clone, Default)]
pub struct Texture2DGroup {
    /// Unique resource ID.
    pub id: u32,
    /// Reference to texture2d element.
    pub tex_id: u32,
    /// Reference to display properties.
    pub display_properties_id: Option<u32>,
    /// List of texture coordinates.
    pub tex_2_coords: Vec<Tex2Coord>,
}

impl Texture2DGroup {
    /// Create a new texture 2D group.
    pub fn new(id: u32, texture_id: u32) -> Self {
        Self {
            id,
            tex_id: texture_id,
            display_properties_id: None,
            tex_2_coords: Vec::new(),
        }
    }

    /// Add a texture coordinate.
    pub fn add_tex2_coord(&mut self, u: f64, v: f64) -> usize {
        let index = self.tex_2_coords.len();
        self.tex_2_coords.push(Tex2Coord { u, v });
        index
    }

    /// Set the display properties ID.
    pub fn set_display_properties(&mut self, display_properties_id: u32) -> &mut Self {
        self.display_properties_id = Some(display_properties_id);
        self
    }

}

/// A 2D texture coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tex2Coord {
    /// U coordinate.
    pub u: f64,
    /// V coordinate.
    pub v: f64,
}

impl Tex2Coord {
    /// Create a new texture coordinate.
    pub fn new(u: f64, v: f64) -> Self {
        Self { u, v }
    }
}

/// Composite materials group.
#[derive(Debug, Clone, Default)]
pub struct CompositeMaterials {
    /// Unique resource ID.
    pub id: u32,
    /// Reference to base materials group.
    pub mat_id: u32,
    /// Space-delimited list of material indices.
    pub mat_indices: Vec<u32>,
    /// Reference to display properties.
    pub display_properties_id: Option<u32>,
    /// List of composites.
    pub composites: Vec<Composite>,
}

impl CompositeMaterials {
    /// Create a new composite materials group.
    pub fn new(id: u32, material_id: u32) -> Self {
        Self {
            id,
            mat_id: material_id,
            mat_indices: Vec::new(),
            display_properties_id: None,
            composites: Vec::new(),
        }
    }

    /// Add a composite.
    pub fn add_composite(mut self, values: Vec<f64>) -> usize {
        let index = self.composites.len();
        self.composites.push(Composite { values });
        index
    }

    /// Set the material indices.
    pub fn set_material_indices(&mut self, indices: Vec<u32>) -> &mut Self {
        self.mat_indices = indices;
        self
    }

    /// Set the display properties ID.
    pub fn set_display_properties(&mut self, display_properties_id: u32) -> &mut Self {
        self.display_properties_id = Some(display_properties_id);
        self
    }

}

/// A composite material definition.
#[derive(Debug, Clone, Default)]
pub struct Composite {
    /// Space-delimited list of values between 0 and 1 representing fractions.
    pub values: Vec<f64>,
}

impl Composite {
    /// Create a new composite.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Multi-properties group.
#[derive(Debug, Clone, Default)]
pub struct MultiProperties {
    /// Unique resource ID.
    pub id: u32,
    /// Space-delimited list of property group IDs.
    pub p_ids: Vec<u32>,
    /// List of blend methods (optional).
    pub blend_methods: Vec<BlendMethod>,
    /// List of multi-property definitions.
    pub multis: Vec<Multi>,
}

impl MultiProperties {
    /// Create a new multi-properties group.
    pub fn new(id: u32, property_ids: Vec<u32>) -> Self {
        Self {
            id,
            p_ids: property_ids,
            blend_methods: Vec::new(),
            multis: Vec::new(),
        }
    }

    /// Add a multi-property definition.
    pub fn add_multi(mut self, property_indices: Vec<u32>) -> usize {
        let index = self.multis.len();
        self.multis.push(Multi { p_indices: property_indices });
        index
    }

    /// Set the blend methods.
    pub fn set_blend_methods(&mut self, blend_methods: Vec<BlendMethod>) -> &mut Self {
        self.blend_methods = blend_methods;
        self
    }
}

/// Blend method for multi-properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMethod {
    /// Linear mix interpolation.
    #[default]
    Mix,
    /// Multiply blend.
    Multiply,
}

impl fmt::Display for BlendMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlendMethod::Mix => write!(f, "mix"),
            BlendMethod::Multiply => write!(f, "multiply"),
        }
    }
}

impl FromStr for BlendMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mix" => Ok(BlendMethod::Mix),
            "multiply" => Ok(BlendMethod::Multiply),
            _ => Err(Error::InvalidAttribute {
                name: "blendmethods".to_string(),
                message: format!("unknown blend method: {}", s),
            }),
        }
    }
}

/// A list of blend methods.
pub struct BlendMethods(Vec<BlendMethod>);

impl FromStr for BlendMethods {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut methods = vec![];
        for s in s.split_whitespace() {
            methods.push(BlendMethod::from_str(s)?);
        }
        Ok(BlendMethods(methods))
    }
}

impl fmt::Display for BlendMethods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let methods: Vec<String> = self.0.iter()
            .map(|m| m.to_string()).collect();
        write!(f, "{}", methods.join(" "))
    }
}

impl From<BlendMethods> for Vec<BlendMethod> {
    fn from(blend_methods: BlendMethods) -> Self {
        blend_methods.0
    }
}

/// A multi-property definition.
#[derive(Debug, Clone, Default)]
pub struct Multi {
    /// Space-delimited list of property indices.
    pub p_indices: Vec<u32>,
}

impl Multi {
    /// Create a new multi-property.
    pub fn new(property_indices: Vec<u32>) -> Self {
        Self { p_indices: property_indices }
    }
}

/// PBR Specular display properties group.
#[derive(Debug, Clone, Default)]
pub struct PBSpecularDisplayProperties {
    /// Unique resource ID.
    pub id: u32,
    /// List of specular materials.
    pub pb_speculars: Vec<PBSpecular>,
}

impl PBSpecularDisplayProperties {
    /// Create a new PBR specular display properties group.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            pb_speculars: Vec::new(),
        }
    }

    /// Add a PBR specular material.
    pub fn add_pbspecular(mut self, pbspecular: PBSpecular) -> usize {
        let index = self.pb_speculars.len();
        self.pb_speculars.push(pbspecular);
        index
    }
}

/// PBR Specular material.
#[derive(Debug, Clone, Default)]
pub struct PBSpecular {
    /// Material name.
    pub name: String,
    /// Specular color (default: #383838).
    pub specular_color: Color,
    /// Surface glossiness (default: 0).
    pub glossiness: f64,
}

impl PBSpecular {
    /// Create a new PBR specular material.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            specular_color: Color::rgb(0x38, 0x38, 0x38),
            glossiness: 0.0,
        }
    }

    /// Set the specular color.
    pub fn set_specular_color(&mut self, color: Color) -> &mut Self {
        self.specular_color = color;
        self
    }

    /// Set the glossiness.
    pub fn set_glossiness(&mut self, glossiness: f64) -> &mut Self {
        self.glossiness = glossiness;
        self
    }
}

/// PBR Metallic display properties group.
#[derive(Debug, Clone, Default)]
pub struct PBMetallicDisplayProperties {
    /// Unique resource ID.
    pub id: u32,
    /// List of metallic materials.
    pub pb_metallics: Vec<PBMetallic>,
}

impl PBMetallicDisplayProperties {
    /// Create a new PBR metallic display properties group.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            pb_metallics: Vec::new(),
        }
    }

    /// Add a PBR metallic material.
    pub fn add_pbmetallic(mut self, pbmetallic: PBMetallic) -> usize {
        let index = self.pb_metallics.len();
        self.pb_metallics.push(pbmetallic);
        index
    }
}

/// PBR Metallic material.
#[derive(Debug, Clone, Default)]
pub struct PBMetallic {
    /// Material name.
    pub name: String,
    /// Surface metallicness (default: 0).
    pub metallicness: f64,
    /// Surface roughness (default: 1).
    pub roughness: f64,
}

impl PBMetallic {
    /// Create a new PBR metallic material.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metallicness: 0.0,
            roughness: 1.0,
        }
    }

    /// Set the metallicness.
    pub fn set_metallicness(&mut self, metallicness: f64) -> &mut Self {
        self.metallicness = metallicness;
        self
    }

    /// Set the roughness.
    pub fn set_roughness(&mut self, roughness: f64) -> &mut Self {
        self.roughness = roughness;
        self
    }
}

/// PBR Specular texture display properties.
#[derive(Debug, Clone, Default)]
pub struct PBSpecularTextureDisplayProperties {
    /// Unique resource ID.
    pub id: u32,
    /// Material name.
    pub name: String,
    /// Reference to specular texture.
    pub specular_texture_id: u32,
    /// Reference to glossiness texture.
    pub glossiness_texture_id: u32,
    /// Diffuse color multiplication factor (default: #FFFFFF).
    pub diffuse_factor: Color,
    /// Specular multiplication factor (default: #FFFFFF).
    pub specular_factor: Color,
    /// Glossiness multiplication factor (default: 1).
    pub glossiness_factor: f64,
}

impl PBSpecularTextureDisplayProperties {
    /// Create a new PBR specular texture display properties.
    pub fn new(id: u32, name: impl Into<String>, specular_texture_id: u32, glossiness_texture_id: u32) -> Self {
        Self {
            id,
            name: name.into(),
            specular_texture_id,
            glossiness_texture_id,
            diffuse_factor: Color::WHITE,
            specular_factor: Color::WHITE,
            glossiness_factor: 1.0,
        }
    }

    /// Set the diffuse factor.
    pub fn set_diffuse_factor(&mut self, color: Color) -> &mut Self {
        self.diffuse_factor = color;
        self
    }

    /// Set the specular factor.
    pub fn set_specular_factor(&mut self, color: Color) -> &mut Self {
        self.specular_factor = color;
        self
    }

    /// Set the glossiness factor.
    pub fn set_glossiness_factor(&mut self, factor: f64) -> &mut Self {
        self.glossiness_factor = factor;
        self
    }
}

/// PBR Metallic texture display properties.
#[derive(Debug, Clone, Default)]
pub struct PBMetallicTextureDisplayProperties {
    /// Unique resource ID.
    pub id: u32,
    /// Material name.
    pub name: String,
    /// Reference to metallic texture.
    pub metallic_texture_id: u32,
    /// Reference to roughness texture.
    pub roughness_texture_id: u32,
    /// Base color multiplication factor (default: #FFFFFF).
    pub base_color_factor: Color, // The xsd section undefined, but specification has explanation.
    /// Metallicness multiplication factor (default: 1).
    pub metallic_factor: f64,
    /// Roughness multiplication factor (default: 1).
    pub roughness_factor: f64,
}

impl PBMetallicTextureDisplayProperties {
    /// Create a new PBR metallic texture display properties.
    pub fn new(id: u32, name: impl Into<String>, metallic_texture_id: u32, roughness_texture_id: u32) -> Self {
        Self {
            id,
            name: name.into(),
            metallic_texture_id,
            roughness_texture_id,
            base_color_factor: Color::WHITE,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
        }
    }

    /// Set the metallic factor.
    pub fn set_metallic_factor(&mut self, factor: f64) -> &mut Self {
        self.metallic_factor = factor;
        self
    }

    /// Set the roughness factor.
    pub fn set_roughness_factor(&mut self, factor: f64) -> &mut Self {
        self.roughness_factor = factor;
        self
    }

    /// Set the base color factor.
    pub fn set_base_color_factor(&mut self, color: Color) -> &mut Self {
        self.base_color_factor = color;
        self
    }
}

/// Translucent display properties group.
#[derive(Debug, Clone, Default)]
pub struct TranslucentDisplayProperties {
    /// Unique resource ID.
    pub id: u32,
    /// List of translucent materials.
    pub translucents: Vec<Translucent>,
}

impl TranslucentDisplayProperties {
    /// Create a new translucent display properties group.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            translucents: Vec::new(),
        }
    }

    /// Add a translucent material.
    pub fn add_translucent(mut self, translucent: Translucent) -> usize {
        let index = self.translucents.len();
        self.translucents.push(translucent);
        index
    }
}

/// Translucent material.
#[derive(Debug, Clone, Default)]
pub struct Translucent {
    /// Material name.
    pub name: String,
    /// Linear attenuation coefficient.
    pub attenuation: [f64; 3],
    /// Refractive index (default: 1 1 1).
    pub refractive_index: [f64; 3],
    /// Surface roughness (default: 0).
    pub roughness: f64,
}

impl Translucent {
    /// Create a new translucent material.
    pub fn new(name: impl Into<String>, attenuation: [f64; 3]) -> Self {
        Self {
            name: name.into(),
            attenuation,
            refractive_index: [1.0, 1.0, 1.0],
            roughness: 0.0,
        }
    }

    /// Set the refractive index.
    pub fn set_refractive_index(&mut self, refractive_index: [f64; 3]) -> &mut Self {
        self.refractive_index = refractive_index;
        self
    }

    /// Set the roughness.
    pub fn set_roughness(&mut self, roughness: f64) -> &mut Self {
        self.roughness = roughness;
        self
    }
}

/// Collection of all materials extension resources.
#[derive(Debug, Clone, Default)]
pub struct MaterialResources {
    /// 2D texture resources.
    pub texture_2ds: Vec<Texture2D>,
    /// Color groups.
    pub color_groups: Vec<ColorGroup>,
    /// Texture 2D groups.
    pub texture_2d_groups: Vec<Texture2DGroup>,
    /// Composite materials.
    pub composite_materials: Vec<CompositeMaterials>,
    /// Multi-properties.
    pub multi_properties: Vec<MultiProperties>,
    /// PBR specular display properties.
    pub pb_specular_display_properties: Vec<PBSpecularDisplayProperties>,
    /// PBR metallic display properties.
    pub pb_metallic_display_properties: Vec<PBMetallicDisplayProperties>,
    /// PBR specular texture display properties.
    pub pb_specular_texture_display_properties: Vec<PBSpecularTextureDisplayProperties>,
    /// PBR metallic texture display properties.
    pub pb_metallic_texture_display_properties: Vec<PBMetallicTextureDisplayProperties>,
    /// Translucent display properties.
    pub translucent_display_properties: Vec<TranslucentDisplayProperties>,
}

impl MaterialResources {
    /// Create a new empty materials resources collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a texture 2D.
    pub fn add_texture_2d(&mut self, texture: Texture2D) {
        self.texture_2ds.push(texture);
    }

    /// Add a color group.
    pub fn add_color_group(&mut self, color_group: ColorGroup) {
        self.color_groups.push(color_group);
    }

    /// Add a texture 2D group.
    pub fn add_texture_2d_group(&mut self, texture_group: Texture2DGroup) {
        self.texture_2d_groups.push(texture_group);
    }

    /// Add composite materials.
    pub fn add_composite_materials(&mut self, composite: CompositeMaterials) {
        self.composite_materials.push(composite);
    }

    /// Add multi-properties.
    pub fn add_multi_properties(&mut self, multi_props: MultiProperties) {
        self.multi_properties.push(multi_props);
    }

    /// Add PBR specular display properties.
    pub fn add_pb_specular_display_properties(&mut self, props: PBSpecularDisplayProperties) {
        self.pb_specular_display_properties.push(props);
    }

    /// Add PBR metallic display properties.
    pub fn add_pb_metallic_display_properties(&mut self, props: PBMetallicDisplayProperties) {
        self.pb_metallic_display_properties.push(props);
    }

    /// Add PBR specular texture display properties.
    pub fn add_pb_specular_texture_display_properties(&mut self, props: PBSpecularTextureDisplayProperties) {
        self.pb_specular_texture_display_properties.push(props);
    }

    /// Add PBR metallic texture display properties.
    pub fn add_pb_metallic_texture_display_properties(&mut self, props: PBMetallicTextureDisplayProperties) {
        self.pb_metallic_texture_display_properties.push(props);
    }

    /// Add translucent display properties.
    pub fn add_translucent_display_properties(&mut self, props: TranslucentDisplayProperties) {
        self.translucent_display_properties.push(props);
    }
}