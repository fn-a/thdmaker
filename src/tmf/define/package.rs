//! 3MF package (ZIP archive) handling.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::fmt::{Display, Formatter, Result};
use super::model::*;
use super::primitive::*;
use super::securecontent::KeyStore;
use super::path::*;

/// A 3MF package containing the model and associated resources.
#[derive(Debug, Clone, Default)]
pub struct Package {
    /// The main 3D model in the package.
    pub model: Model,
    /// The 3D models in the package with their paths.
    pub models: HashMap<String, Model>,
    /// The thumbnail image for the package.
    pub thumbnail: Option<Attachment>,
    /// Additional attachments with their paths (thumbnails, textures, etc.).
    pub attachments: HashMap<String, Attachment>,
    /// Content types for parts.
    pub content_types: ContentTypes,
    /// The default relationship for the package.
    pub relationships: Relationships,
    /// Relationships with paths as keys.
    pub relative_rels: HashMap<String, Relationships>,
    /// Optional key store file in the package from secure content extension.
    pub key_store: Option<KeyStore>,
}

/// A generic attachment in the package.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Attachment data.
    pub data: Vec<u8>,
    /// File extension (png, jpeg, texture, etc.).
    pub extension: String,
    /// Content type (image/png, image/jpeg, etc.).
    pub content_type: String,
}

impl Attachment {
    /// Create a PNG thumbnail.
    pub fn png(data: Vec<u8>) -> Self {
        Self {
            data,
            extension: Extension::PNG.to_string(),
            content_type: ContentType::PNG.to_string(),
        }
    }

    /// Create a jpg thumbnail.
    pub fn jpg(data: Vec<u8>) -> Self {
        Self {
            data,
            extension: Extension::JPG.to_string(),
            content_type: ContentType::JPG.to_string(),
        }
    }

    /// Create a JPEG thumbnail.
    pub fn jpeg(data: Vec<u8>) -> Self {
        Self {
            data,
            extension: Extension::JPEG.to_string(),
            content_type: ContentType::JPEG.to_string(),
        }
    }
}

/// Content types for parts.
#[derive(Debug, Clone, Default)]
pub struct ContentTypes {
    /// Default content type for parts.
    pub defaults: Vec<ContentTypeDefault>,
    /// Override content type for parts.
    pub overrides: Vec<ContentTypeOverride>,
    /// Unknown content types.
    pub unknowns: HashMap<String, Vec<HashMap<String, String>>>,
}

impl ContentTypes {
    pub fn ensure_default(&mut self, extension: &str, content_type: &str) {
        if !self.defaults.iter().any(|d| d.extension == extension.to_string()) {
            self.defaults.push(ContentTypeDefault {
                extension: extension.to_string(),
                content_type: content_type.to_string(),
            });
        }
    }

    pub fn ensure_override(&mut self, part_name: &str, content_type: &str) {
        if !self.overrides.iter().any(|o| o.part_name == part_name.to_string()) {
            self.overrides.push(ContentTypeOverride {
                part_name: part_name.to_string(),
                content_type: content_type.to_string(),
            });
        }
    }
}

/// Default content type for parts.
#[derive(Debug, Clone)]
pub struct ContentTypeDefault {
    pub extension: String,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct ContentTypeOverride {
    pub part_name: String,
    pub content_type: String,
}

/// A collection of relationships in the package.
#[derive(Debug, Clone, Default)]
pub struct Relationships {
    /// Relationships by type.
    pub relationships: Vec<Relationship>,
}

impl Relationships {
    /// Add a relationship to the collection.
    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    /// Get the next relationship ID.
    pub fn next_id(&self) -> String {
        "rel".to_owned() + &self.relationships.len().to_string()
    }

    /// Check if there are any relationships.
    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    /// Ensure a relationship of the given type and target exists.
    pub fn ensure(&mut self, r#type: &str, target: &str) {
        if !self.relationships.iter().any(|r| r.r#type == r#type.to_string() && r.target == target.to_string()) {
            self.relationships.push(Relationship {
                id: "rel".to_owned() + &self.relationships.len().to_string(),
                r#type: r#type.to_string(),
                target: target.to_string(),
            });
        }
    }
}

/// A relationship in the package.
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Relationship ID.
    pub id: String,
    /// Relationship type URI.
    pub r#type: String,
    /// Target path.
    pub target: String,
}

impl Package {
    /// Create a new empty package.
    pub fn new() -> Self {
        Self {
            model: Model::new(),
            models: HashMap::new(),
            thumbnail: None,
            attachments: HashMap::new(),
            content_types: ContentTypes::default(),
            relationships: Relationships::default(),
            relative_rels: HashMap::new(),
            key_store: None,
        }
    }

    /// Create a package with a model.
    pub fn with_model(model: Model) -> Self {
        Self {
            model: model.clone(),
            models: HashMap::new(),
            thumbnail: None,
            attachments: HashMap::new(),
            content_types: ContentTypes::default(),
            relationships: Relationships::default(),
            relative_rels: HashMap::new(),
            key_store: None,
        }
    }

    /// Set the main 3D model in the package.
    pub fn set_model(&mut self, model: Model) -> &mut Self {
        self.model = model;
        self
    }

    /// Add a 3D model to the package with its path.
    pub fn add_model(&mut self, path: impl Into<String>, model: Model) -> &mut Self {
        self.models.insert(path.into(), model);
        self
    }

    /// Set the content types for the package.
    pub fn set_content_types(&mut self, content_types: ContentTypes) {
        self.content_types = content_types;
    }

    pub fn set_relationships(&mut self, relationships: Relationships) {
        self.relationships = relationships;
    }

    /// Add a relationship to the package.
    pub fn add_relationships(&mut self, path: impl Into<String>, relationships: Relationships) {
        self.relative_rels.insert(path.into(), relationships);
    }

    /// Add an attachment to the package.
    pub fn add_attachment(&mut self, path: impl Into<String>, data: Vec<u8>) {
        let path = path.into();
        let path = path.as_str();
        let extension = extension(path);
        let path = absolute(path, None);
        let content_type = CTYPE_EXT_MAP[&extension].to_string();
        self.attachments.insert(
            path,
            Attachment {
                data,
                extension,
                content_type,
            }
        );
    }

    /// Get an attachment by path.
    pub fn get_attachment(&self, path: &str) -> Option<&[u8]> {
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        self.attachments.get(&path).map(|v| v.data.as_slice())
    }

    /// Set the key store in the package.
    pub fn set_key_store(&mut self, key_store: KeyStore) -> &mut Self {
        self.key_store = Some(key_store);
        self
    }

    /// Get the key store in the package.
    pub fn get_key_store(&self) -> Option<&KeyStore> {
        self.key_store.as_ref()
    }

    /// Set the model title.
    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.model.set_title(title);
        self
    }

    /// Set the designer name.
    pub fn set_designer(&mut self, designer: impl Into<String>) -> &mut Self {
        self.model.set_designer(designer);
        self
    }

    /// Set the description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.model.set_description(description);
        self
    }

    /// Set the application name.
    pub fn set_application(&mut self, application: impl Into<String>) -> &mut Self {
        self.model.set_application(application);
        self
    }

    /// Set the package thumbnail.
    pub fn set_thumbnail(&mut self, thumbnail: Attachment) {
        self.thumbnail = Some(thumbnail);
    }

    /// Set a PNG thumbnail.
    pub fn set_thumbnail_png(&mut self, data: Vec<u8>) -> &mut Self {
        self.set_thumbnail(Attachment::png(data));
        self
    }

    /// Set a JPG thumbnail.
    pub fn set_thumbnail_jpg(&mut self, data: Vec<u8>) -> &mut Self {
        self.set_thumbnail(Attachment::jpg(data));
        self
    }

    /// Set a JPEG thumbnail.
    pub fn set_thumbnail_jpeg(&mut self, data: Vec<u8>) -> &mut Self {
        self.set_thumbnail(Attachment::jpeg(data));
        self
    }

    /// Set the unit.
    pub fn set_unit(&mut self, unit: Unit) -> &mut Self {
        self.model.unit = unit;
        self
    }

    /// Add a cube to the model.
    pub fn add_cube(mut self, size: f64) -> Self {
        let mesh = Mesh::cube(size);
        let mut object = self.model.gen_object();
        object.set_name("Cube").set_mesh(mesh);
        let id = self.model.add_object(object);
        self.model.add_build_item(id, None);
        self
    }

    /// Add a cube with a specific position.
    pub fn add_cube_placed(mut self, size: f64, x: f64, y: f64, z: f64) -> Self {
        let mesh = Mesh::cube(size);
        let mut object = self.model.gen_object();
        object.set_name("Cube").set_mesh(mesh);
        let id = self.model.add_object(object);
        let transform = Matrix3D::translation(x, y, z);
        self.model.add_build_item(id, Some(transform));
        self
    }

    /// Add a colored cube.
    pub fn add_cube_colored(mut self, size: f64, color: Color) -> Self {
        // Add or get material
        let material_id = self.model.add_base_materials(vec![BaseMaterial::new("Color", color)]);
        let mesh = Mesh::cube(size);
        let mut object = self.model.gen_object();
        object.set_name("Cube").set_mesh(mesh).set_material(material_id, 0);
        let id = self.model.add_object(object);
        self.model.add_build_item(id, None);

        self
    }

    /// Add a custom mesh.
    pub fn add_mesh(mut self, mesh: Mesh) -> Self {
        let mut object = self.model.gen_object();
        object.set_mesh(mesh);
        let id = self.model.add_object(object);
        self.model.add_build_item(id, None);
        self
    }

    /// Add a custom mesh with a name.
    pub fn add_mesh_named(mut self, name: impl Into<String>, mesh: Mesh) -> Self {
        let mut object = self.model.gen_object();
        object.set_name(name).set_mesh(mesh);
        let id = self.model.add_object(object);
        self.model.add_build_item(id, None);
        self
    }
}

pub static EXT_CTYPE_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(Extension::JPEG_EXT.to_string(), ContentType::JPEG_CT.to_string());
    map.insert(Extension::JPG_EXT.to_string(), ContentType::JPG_CT.to_string());
    map.insert(Extension::PNG_EXT.to_string(), ContentType::PNG_CT.to_string());
    map.insert(Extension::RELATIONSHIP_EXT.to_string(), ContentType::RELATIONSHIP_CT.to_string());
    map.insert(Extension::MODEL_EXT.to_string(), ContentType::MODEL_CT.to_string());
    map.insert(Extension::TEXTURE_EXT.to_string(), ContentType::TEXTURE_CT.to_string());
    map
});

pub static CTYPE_EXT_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(ContentType::JPEG_CT.to_string(), Extension::JPEG_EXT.to_string());
    map.insert(ContentType::JPG_CT.to_string(), Extension::JPG_EXT.to_string());
    map.insert(ContentType::PNG_CT.to_string(), Extension::PNG_EXT.to_string());
    map.insert(ContentType::RELATIONSHIP_CT.to_string(), Extension::RELATIONSHIP_EXT.to_string());
    map.insert(ContentType::MODEL_CT.to_string(), Extension::MODEL_EXT.to_string());
    map.insert(ContentType::TEXTURE_CT.to_string(), Extension::TEXTURE_EXT.to_string());
    map
});

/// The path of the file in the package.
pub enum Pathway {
    Model,
    Thumbnail,
    ThumbnailJpg,
    ThumbnailPng,
    ThumbnailJpeg,
    ContentTypes,
    RelationshipsRoot,
    RelationshipsGlob,
}

impl Pathway {
    /// Default path for the 3D model part.
    pub const MODEL_PATH: &str = "/3D/3dmodel.model";
    /// Default path for the thumbnail part.
    pub const THUMBNAIL_PATH: &str = "/Metadata/thumbnail";
    pub const THUMBNAIL_JPG_PATH: &str = "/Metadata/thumbnail.jpg";
    pub const THUMBNAIL_PNG_PATH: &str = "/Metadata/thumbnail.png";
    pub const THUMBNAIL_JPEG_PATH: &str = "/Metadata/thumbnail.jpeg";
    /// Content types file path.
    pub const CONTENT_TYPES_PATH: &str = "/[Content_Types].xml";
    /// Root relationships file path.
    pub const RELS_ROOT_PATH: &str = "/_rels/.rels";
    /// Relationships glob pattern.
    pub const RELS_GLOB_EXT: &str = "/**/*/_rels/*.rels";
    /// Model glob pattern.
    pub const MODEL_GLOB_EXT: &str = "/3D/**/.*.model";
    /// Default path for the keystore part from secure content extension.
    pub const KEY_STORE_PATH: &str = "/Secure/keystore.xml";
}

impl Display for Pathway {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Model => write!(f, "{}", Self::MODEL_PATH),
            Self::Thumbnail => write!(f, "{}", Self::THUMBNAIL_PATH),
            Self::ThumbnailJpg => write!(f, "{}", Self::THUMBNAIL_JPG_PATH),
            Self::ThumbnailPng => write!(f, "{}", Self::THUMBNAIL_PNG_PATH),
            Self::ThumbnailJpeg => write!(f, "{}", Self::THUMBNAIL_JPEG_PATH),
            Self::ContentTypes => write!(f, "{}", Self::CONTENT_TYPES_PATH),
            Self::RelationshipsRoot => write!(f, "{}", Self::RELS_ROOT_PATH),
            Self::RelationshipsGlob => write!(f, "{}", Self::RELS_GLOB_EXT),
        }
    }
}

/// The extension of the file.
pub enum Extension {
    JPEG,
    JPG,
    PNG,
    Relationship,
    Model,
    Texture,
}

impl Extension {
    /// The extension of the JPEG file.
    pub const JPEG_EXT: &str = "jpeg";
    /// The extension of the JPG file.
    pub const JPG_EXT: &str = "jpg";
    /// The extension of the PNG file.
    pub const PNG_EXT: &str = "png";
    /// The extension of the Relationship file.
    pub const RELATIONSHIP_EXT: &str = "rels";
    /// The extension of the Model file.
    pub const MODEL_EXT: &str = "model";
    /// The extension of the Texture file.
    pub const TEXTURE_EXT: &str = "texture";
    /// The extension of the xml file, for example: keystore.xml file from secure content extension.
    pub const XML_EXT: &str = "xml";
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::JPEG => write!(f, "{}", Self::JPEG_EXT),
            Self::JPG => write!(f, "{}", Self::JPG_EXT),
            Self::PNG => write!(f, "{}", Self::PNG_EXT),
            Self::Relationship => write!(f, "{}", Self::RELATIONSHIP_EXT),
            Self::Model => write!(f, "{}", Self::MODEL_EXT),
            Self::Texture => write!(f, "{}", Self::TEXTURE_EXT),
        }
    }
}

/// The content type of the file.
pub enum ContentType {
    JPEG,
    JPG,
    PNG,
    Relationship,
    Model,
    PrintTicket,
    Texture,
}

impl ContentType {
    /// The content type of the JPEG file.
    pub const JPEG_CT: &str = "image/jpeg";
    /// The content type of the JPG file.
    pub const JPG_CT: &str = "image/jpeg";
    /// The content type of the PNG file.
    pub const PNG_CT: &str = "image/png";
    /// The content type of the Relationship file.
    pub const RELATIONSHIP_CT: &str = "application/vnd.openxmlformats-package.relationships+xml";
    /// The content type of the Model file.
    pub const MODEL_CT: &str = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml";
    /// The content type of the PrintTicket file.
    pub const PRINT_TICKET_CT: &str = "application/vnd.ms-printing.printticket+xml";
    /// The content type of the Texture file.
    pub const TEXTURE_CT: &str = "application/vnd.ms-package.3dmanufacturing-3dmodeltexture";
    /// The content type of the KeyStore file from secure content extension.
    pub const KEY_STORE_CT: &str = "application/vnd.ms-package.3dmanufacturing-keystore+xml";
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::JPEG => write!(f, "{}", Self::JPEG_CT),
            Self::JPG => write!(f, "{}", Self::JPG_CT),
            Self::PNG => write!(f, "{}", Self::PNG_CT),
            Self::Relationship => write!(f, "{}", Self::RELATIONSHIP_CT),
            Self::Model => write!(f, "{}", Self::MODEL_CT),
            Self::PrintTicket => write!(f, "{}", Self::PRINT_TICKET_CT),
            Self::Texture => write!(f, "{}", Self::TEXTURE_CT),
        }
    }
}

/// The namespace of the file.
pub enum Namespace {
    Core,
    TriangleSets,
    Model,
    Thumbnail,
    Relationship,
}

impl Namespace {
    /// 3MF Core namespace.
    pub const CORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";
    /// Content types namespace.
    pub const CONTENT_TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
    /// 3MF Model/StartPart type namespace.
    pub const MODEL_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";
    /// 3MF Thumbnail type namespace.
    pub const THUMBNAIL_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";
    /// 3MF Relationship type namespace.
    pub const RELATIONSHIP_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    /// 3MF PrintTicket type namespace.
    pub const PRINT_TICKET_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/printticket";
    /// 3MF MustPreserve type namespace.
    pub const MUST_PRESERVE_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships/mustpreserve";
    
    /// Triangle Sets namespace.
    pub const TRIANGLESETS_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07";
    /// Beam Lattice namespace.
    pub const BEAMLATTICE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02";
    /// Balls within Beam Lattice namespace.
    pub const BALLS_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07";
    /// 3MF Displacement Extension namespace.
    pub const DISPLACEMENT_NS: &str = "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10";
    /// 3MF Materials and Properties Extension namespace.
    pub const MATERIALS_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/material/2015/02";
    /// 3MF Production Extension namespace.
    pub const PRODUCTION_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
    /// 3MF Production Alternatives namespace.
    pub const ALTERNATIVES_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/production/alternatives/2021/04";
    /// 3MF Secure Content Extension namespace.
    pub const SECURECONTENT_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04";
    /// 3MF Secure Content Extension keystore namespace.
    pub const KEY_STORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/2019/04/keystore";
    /// 3MF Secure Content Extension encryptedfile namespace.
    pub const ENCRYPTED_FILE_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships/encryptedfile";
    /// 3MF Slice Extension namespace.
    pub const SLICE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07";
    /// 3MF Volumetric Extension namespace.
    pub const VOLUMETRIC_NS: &str = "http://schemas.3mf.io/3dmanufacturing/volumetric/2022/01";
    /// 3MF Implicit Extension namespace.
    pub const IMPLICIT_NS: &str = "http://schemas.3mf.io/3dmanufacturing/implicit/2023/12";
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Core => write!(f, "{}", Self::CORE_NS),
            Self::TriangleSets => write!(f, "{}", Self::TRIANGLESETS_NS), 
            Self::Model => write!(f, "{}", Self::MODEL_NS),
            Self::Thumbnail => write!(f, "{}", Self::THUMBNAIL_NS),
            Self::Relationship => write!(f, "{}", Self::RELATIONSHIP_NS),
        }
    }
}
