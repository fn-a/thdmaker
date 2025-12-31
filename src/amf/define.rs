/// Double, base double, white space collapse, pattern: ((\-|\+)?(([0-9]+(\.[0-9]+)?)|(\.[0-9]+))((e|E)(\-|\+)?[0-9]+)?)
/// NegOneToOne, base Double, min: -1.0, max: 1.0
/// Degrees, base Double, min: -360, max: 360
/// TextureBase, base string, white space collapse, pattern: ([0-9]|[a-z]|[A-Z]|\+|\/|\=)+
/// ColorValue, base string

use std::fmt::{Display, Formatter, Result};

/// AMF Document
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub unit: Option<Unit>,
    // Double
    pub version: Option<f64>,
    // xml:lang
    pub lang: Option<String>,
    // At least one object
    pub objects: Vec<Object>,
    // Can be empty
    pub materials: Vec<Material>,
    // Can be empty
    pub textures: Vec<Texture>,
    // Can be empty
    pub constellations: Vec<Constellation>,
    // Can be empty
    pub metadatas: Vec<Metadata>,
}

/// Unit Element
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Unit {
    Inch,
    Millimeter,
    #[default]
    Meter,
    Feet,
    Micron,
}

/// Object Element
#[derive(Debug, Clone, Default)]
pub struct Object {
    pub id: u32,
    // Can be empty
    pub colors: Vec<Color>,
    // Must have only one mesh
    pub mesh: Mesh,
    // Can be empty
    pub metadatas: Vec<Metadata>,
}

/// Metadata Element
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub r#type: String,
    // Element content text
    pub value: String,
}

/// Color Definition
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    // ColorValue
    pub r: String,
    // ColorValue
    pub g: String,
    // ColorValue
    pub b: String,
    // ColorValue
    pub a: Option<String>,
}

/// Mesh
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vertices,
    pub volume: Volume,
}

/// Set of vertices
#[derive(Debug, Clone, Default)]
pub struct Vertices {
    // At least one vertex
    pub vertices: Vec<Vertex>,
    // Can be empty
    pub edges: Vec<Edge>,
}

/// Vertex
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    // Can be empty
    pub metadatas: Vec<Metadata>,
    // Must have only one coordinates
    pub coordinates: Coordinates,
    // Can be empty
    pub colors: Vec<Color>,
    // Can be empty
    pub normals: Vec<Normal>,
}

/// Coordinate Points
#[derive(Debug, Clone, Default)]
pub struct Coordinates {
    // Double
    pub x: f64,
    // Double
    pub y: f64,
    // Double
    pub z: f64,
}

/// Normal Vector
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Normal {
    // NegOneToOne
    pub nx: f64,
    // NegOneToOne
    pub ny: f64,
    // NegOneToOne
    pub nz: f64,
}

/// Edge
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Edge {
    pub v1: u32,
    // NegOneToOne
    pub dx1: f64,
    // NegOneToOne
    pub dy1: f64,
    // NegOneToOne
    pub dz1: f64,
    pub v2: u32,
    // NegOneToOne
    pub dx2: f64,
    // NegOneToOne
    pub dy2: f64,
    // NegOneToOne
    pub dz2: f64,
}

/// Volume (including triangular data)
#[derive(Debug, Clone, Default)]
pub struct Volume {
    // Rename: materialid
    pub material_id: Option<u32>,
    pub r#type: Option<VolumeType>,
    // Can be empty
    pub metadatas: Vec<Metadata>,
    // At least four triangles
    pub triangles: Vec<Triangle>,
    // Can be empty
    pub colors: Vec<Color>,
}

/// Volume Type
#[derive(Debug, Clone, Default, PartialEq)]
pub enum VolumeType {
    #[default]
    Object,
    Support,
}

impl From<&str> for VolumeType {
    fn from(value: &str) -> Self {
        match value {
            "object" => Self::Object,
            "support" => Self::Support,
            _ => Self::Object,
        }
    }
}

impl Display for VolumeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Object => write!(f, "object"),
            Self::Support => write!(f, "support"),
        }
    }
}

/// Triangle
#[derive(Debug, Clone, Default)]
pub struct Triangle {
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
    pub texmap: Option<TexMap>,
    pub colors: Vec<Color>,
}

/// Texture Mapping
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TexMap {
    pub rtexid: Option<u32>,
    pub gtexid: Option<u32>,
    pub btexid: Option<u32>,
    pub atexid: Option<u32>,
    // utex and vtex 2D texture coordinates in the standard
    pub utex1: f64,
    pub utex2: f64,
    pub utex3: f64,
    pub vtex1: f64,
    pub vtex2: f64,
    pub vtex3: f64,
    // wtext 3D texture reservation
    pub wtex1: Option<f64>,
    pub wtex2: Option<f64>,
    pub wtex3: Option<f64>,
}

/// Material Definition
#[derive(Debug, Clone, Default)]
pub struct Material {
    pub id: u32,
    pub metadatas: Vec<Metadata>,
    pub composites: Vec<Composite>,
}

/// Composite Material
#[derive(Debug, Clone, Default)]
pub struct Composite {
    // Rename: materialid
    pub material_id: u32,
    // Element content text
    pub value: String,
}

/// Texture Definition
#[derive(Debug, Clone, Default)]
pub struct Texture {
    pub id: u32,
    // Pixel
    pub width: u32,
    // Pixel
    pub height: u32,
    // Pixel, useful for 3D texture, usually 1 for 2D texture
    pub depth: u32,
    pub tiled: bool,
    pub r#type: TextureType,
    // TextureBase, element content text,
    // The texture data is encoded in Base64 as binary data (such as PNG, JPEG images)
    pub value: String,
}

/// Texture Type
#[derive(Debug, Clone, Default, PartialEq)]
pub enum TextureType {
    #[default]
    Grayscale,
}

impl From<&str> for TextureType {
    fn from(value: &str) -> Self {
        match value {
            "grayscale" => Self::Grayscale,
            _ => Self::Grayscale,
        }
    }
}

impl Display for TextureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Grayscale => write!(f, "grayscale"),
        }
    }
}

/// Constellation
#[derive(Debug, Clone, Default)]
pub struct Constellation {
    pub id: u32,
    // At least two instances
    pub instances: Vec<Instance>,
}

/// Instance, reference to a objec
#[derive(Debug, Clone, Default)]
pub struct Instance {
    // Rename: objectid
    pub object_id: u32,
    // Double
    pub deltax: Option<f64>,
    // Double
    pub deltay: Option<f64>,
    // Double
    pub deltaz: Option<f64>,
    // Degrees
    pub rx: Option<f64>,
    // Degrees
    pub ry: Option<f64>,
    // Degrees
    pub rz: Option<f64>,
}

impl Unit {
    // Scaling unit mapping
    pub const SCALES: &[(Self, f64)] = &[
        (Self::Millimeter, 1.0),
        (Self::Inch, 25.4),
        (Self::Feet, 304.8),
        (Self::Meter, 1000.0),
        (Self::Micron, 0.001),
    ];
}

impl From<&str> for Unit {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "inch" => Unit::Inch,
            "millimeter" | "mm" => Unit::Millimeter,
            "meter" | "m" => Unit::Meter,
            "feet" | "foot" | "ft" => Unit::Feet,
            "micron" => Unit::Micron,
            _ => Unit::Meter,
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Millimeter => write!(f, "millimeter"),
            Self::Inch => write!(f, "inch"),
            Self::Feet => write!(f, "feet"),
            Self::Meter => write!(f, "meter"),
            Self::Micron => write!(f, "micron"),
        }
    }
}

impl Color {
    /// Create a white color
    pub fn white() -> Self {
        Self {
            r: "1.0".to_string(),
            g: "1.0".to_string(),
            b: "1.0".to_string(),
            a: Some("1.0".to_string()),
        }
    }

    /// Create a black color
    pub fn black() -> Self {
        Self {
            r: "0.0".to_string(),
            g: "0.0".to_string(),
            b: "0.0".to_string(),
            a: Some("1.0".to_string()),
        }
    }

    /// Create a red color
    pub fn red() -> Self {
        Self {
            r: "1.0".to_string(),
            g: "0.0".to_string(),
            b: "0.0".to_string(),
            a: Some("1.0".to_string()),
        }
    }

    /// Create a green color
    pub fn green() -> Self {
        Self {
            r: "0.0".to_string(),
            g: "1.0".to_string(),
            b: "0.0".to_string(),
            a: Some("1.0".to_string()),
        }
    }

    /// Create a blue color
    pub fn blue() -> Self {
        Self {
            r: "0.0".to_string(),
            g: "0.0".to_string(),
            b: "1.0".to_string(),
            a: Some("1.0".to_string()),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::white()
    }
}

impl Material {
    /// Create a new material
    pub fn new(id: u32) -> Self {
        Self {
            id,
            metadatas: Vec::new(),
            composites: Vec::new(),
        }
    }

    /// Set material name
    pub fn set_name(&mut self, value: &str) {
        // Remove the existing name metadata
        self.metadatas.retain(|m| m.r#type != "name");
        
        // Add new name metadata
        self.metadatas.push(Metadata {
            r#type: "name".to_string(),
            value: value.to_string(),
        });
    }

    /// Obtain the name of the material
    pub fn get_name(&self) -> Option<&str> {
        self.metadatas
            .iter()
            .find(|m| m.r#type == "name")
            .map(|m| m.value.as_str())
    }
}

impl Volume {
    /// Create a new volume
    pub fn new(material_id: Option<u32>) -> Self {
        Self {
            material_id,
            r#type: None,
            metadatas: Vec::new(),
            triangles: Vec::new(),
            colors: Vec::new(),
        }
    }

    /// Add a triangle to the volume
    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.triangles.push(triangle);
    }

    /// Set the name of the volume
    pub fn set_name(&mut self, value: &str) {
        // Remove the existing name metadata
        self.metadatas.retain(|m| m.r#type != "name");
        
        // Add new name metadata
        self.metadatas.push(Metadata {
            r#type: "name".to_string(),
            value: value.to_string(),
        });
    }

    /// Obtain the volume name
    pub fn get_name(&self) -> Option<&str> {
        self.metadatas
            .iter()
            .find(|m| m.r#type == "name")
            .map(|m| m.value.as_str())
    }
}

impl Mesh {
    /// Create a new mesh
    pub fn new() -> Self {
        Self {
            vertices: Vertices::default(),
            volume: Volume::default(),
        }
    }

    /// Set the vertices to the mesh
    pub fn set_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertices.vertices = vertices;
    }

    /// Add vertices to the mesh
    pub fn add_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertices.vertices.extend(vertices);
    }

    /// Set the edges to the mesh
    pub fn set_edge(&mut self, edges: Vec<Edge>) {
        self.vertices.edges = edges;
    }

    /// Add edges to the mesh
    pub fn add_edge(&mut self, edges: Vec<Edge>) {
        self.vertices.edges.extend(edges);
    }

    /// Set the volume of the mesh
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }
}

impl Object {
    /// Create a new object
    pub fn new(id: u32) -> Self {
        Self {
            id,
            metadatas: Vec::new(),
            colors: Vec::new(),
            mesh: Mesh::new(),
        }
    }

    /// Set the name of the object
    pub fn set_name(&mut self, value: &str) {
        // Remove the existing name metadata
        self.metadatas.retain(|m| m.r#type != "name");
        
        // Add new name metadata
        self.metadatas.push(Metadata {
            r#type: "name".to_string(),
            value: value.to_string(),
        });
    }

    /// Obtain the name of the object
    pub fn get_name(&self) -> Option<&str> {
        self.metadatas
            .iter()
            .find(|m| m.r#type == "name")
            .map(|m| m.value.as_str())
    }

    /// Set the mesh of the object
    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.mesh = mesh;
    }
}

// Helper functions for type conversions
impl Document {
    /// Create a new empty AMF document
    pub fn new() -> Self {
        Self {
            unit: None,
            version: None,
            lang: None,
            objects: Vec::new(),
            materials: Vec::new(),
            textures: Vec::new(),
            constellations: Vec::new(),
            metadatas: Vec::new(),
        }
    }

    /// Add an object to the AMF document
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Get the object with the specified ID
    pub fn get_object(&self, id: u32) -> Option<&Object> {
        self.objects.iter().find(|o| o.id == id)
    }

    /// Get the object with the specified ID mutably
    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut Object> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    /// Set the version of the AMF document
    pub fn set_version(&mut self, value: f64) {
        self.version = Some(value);
    }
}