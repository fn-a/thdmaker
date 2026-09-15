//! Tris type definitions.
//!
//! tris is the intermediate exchange format that bridges the file formats, the slicer and preview engine.
//!
//! Zero cost conversions.
//!
//! Every vertex attribute is stored in a container that is binary compatible
//! with the matching preview attribute, so tris ⇄ preview is an ownership move.

use glam::Mat4;

/// A scene, derived from an obj file or a gltf scene.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    /// All objects.
    pub objects: Vec<Object>,

    /// Unit of the stored coordinates, which the loaders normalise to
    /// millimetres.
    pub unit: Unit,

    /// Scene up axis, which describes the coordinate convention actually
    /// stored.
    pub upa: Axis,

    /// Bounds of the stored vertex positions (cached).
    ///
    /// Object transforms are not applied, so the box is in object space.
    pub bounds: Aabb,
}

/// An object, derived from obj `o` / gltf Node.
#[derive(Clone, Debug)]
pub struct Object {
    /// Object name (obj `o` / gltf node name / 3MF file name).
    pub name: String,

    /// Mesh data.
    pub mesh: Mesh,

    /// Object level transform.
    ///
    /// Loaders keep the original transform; consumers that need world space
    /// coordinates fold it in themselves (the slicer honours it directly).
    pub transform: Mat4,

    /// Extensions.
    pub extensions: Extensions,
}

impl Default for Object {
    /// The identity transform, so a default object is at the origin.
    fn default() -> Self {
        Self {
            name: String::new(),
            mesh: Mesh::default(),
            transform: Mat4::IDENTITY,
            extensions: Extensions::default(),
        }
    }
}

/// A mesh.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    // Topology

    /// Primitive topology.
    ///
    /// Triangle topologies are produced by the file loaders, line strips by the
    /// slicer preview. Kept in sync with engine's `PrimitiveTopology`.
    pub topology: Topology,

    // Vertex attributes (length = positions.len())

    /// Vertex positions (gltf `POSITION`, `ATTRIBUTE_POSITION`).
    pub positions: Vec<[f32; 3]>,

    /// Vertex normals (gltf `NORMAL`, `ATTRIBUTE_NORMAL`).
    pub normals: Option<Vec<[f32; 3]>>,

    /// Texture coordinates (gltf `TEXCOORD_0`, `ATTRIBUTE_UV_0`).
    pub uvs: Option<Vec<[f32; 2]>>,

    /// Vertex colours, linear RGBA (gltf `COLOR_0`, `ATTRIBUTE_COLOR`).
    pub colors: Option<Vec<[f32; 4]>>,

    // Faces

    /// Indices (gltf `indices`). For triangle topologies the length is a
    /// multiple of three, for line topologies of two.
    pub indices: Vec<u32>,

    // Material segments (gltf `Primitive`)

    /// Material segments.
    ///
    /// Empty means every triangle uses the default material.  Storage is
    /// O(segments) instead of O(triangles).
    pub primitives: Vec<Primitive>,

    /// Material table (gltf `materials`).
    pub materials: Vec<Material>,

    /// Extensions.
    pub extensions: Extensions,
}

/// Primitive topology, mirrors engine's `PrimitiveTopology`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Topology {
    /// A list of points.
    PointList,
    /// A list of independent lines.
    LineList,
    /// A connected line strip (the slicer preview).
    LineStrip,
    /// A list of independent triangles.
    #[default]
    TriangleList,
    /// A connected triangle strip.
    TriangleStrip,
}

impl Topology {
    /// Returns `true` for the topologies that consume triangles.
    #[inline]
    pub const fn is_triangle(self) -> bool {
        matches!(self, Self::TriangleList | Self::TriangleStrip)
    }

    /// Returns `true` for the topologies that consume lines.
    pub const fn is_line(self) -> bool {
        matches!(self, Self::LineList | Self::LineStrip)
    }
}

/// A material segment, mirrors gltf's `Primitive`.
///
/// A contiguous run of `indices` sharing one material.
#[derive(Clone, Copy, Debug, Default)]
pub struct Primitive {
    /// Start offset into `indices`.
    pub offset: u32,

    /// Index count (a multiple of three for triangles).
    pub count: u32,

    /// Index into `Mesh::materials`; `u32::MAX` means no material, which is
    /// the default.
    pub material: u32,
}

impl Primitive {
    /// Creates a segment.
    #[inline]
    pub const fn new(offset: u32, count: u32, material: u32) -> Self {
        Self { offset, count, material }
    }

    /// First triangle covered by the segment (relative to `indices`).
    #[inline]
    pub const fn offset(&self) -> u32 {
        self.offset / 3
    }

    /// Number of triangles covered by the segment.
    #[inline]
    pub const fn count(&self) -> u32 {
        self.count / 3
    }
}

/// A material, the `.mtl` `newmtl` block.
///
/// | mtl | tris |
/// |-----|------|
/// | `Kd` | `diffuse` |
/// | `Ks` | `specular` |
/// | `Ke` | `emissive` |
/// | `Ns` | `shininess` |
/// | `d`  | `dissolve` |
/// | `map_Kd` | `map_diffuse` |
/// | `map_Ks` | `map_specular` |
/// | `bump`   | `map_normal` |
#[derive(Clone, Debug)]
pub struct Material {
    /// Material name.
    pub name: String,

    /// Diffuse colour, linear RGBA. Aligns with engine
    /// `StandardMaterial::base_color`.
    pub diffuse: [f32; 4],

    /// Specular colour, RGB.
    pub specular: [f32; 3],

    /// Emissive colour, RGB. Aligns with engine
    /// `StandardMaterial::emissive`.
    pub emissive: [f32; 3],

    /// Specular exponent, 0~1000.
    pub shininess: f32,

    /// Dissolve / opacity, 0~1.
    pub dissolve: f32,

    /// Diffuse map (`map_Kd`).
    pub map_diffuse: Option<Texture>,

    /// Specular map (`map_Ks`).
    pub map_specular: Option<Texture>,

    /// Normal map (`bump`).
    pub map_normal: Option<Texture>,

    /// Extensions.
    pub extensions: Extensions,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::new(),
            diffuse: [1.0, 1.0, 1.0, 1.0],
            specular: [0.0; 3],
            emissive: [0.0; 3],
            shininess: 0.0,
            dissolve: 1.0,
            map_diffuse: None,
            map_specular: None,
            map_normal: None,
            extensions: Extensions::default(),
        }
    }
}

/// A texture reference.
///
/// Holds no pixel data by default, so consumers can load it lazily.
#[derive(Clone, Debug, Default)]
pub struct Texture {
    /// Relative path (external obj file / path inside a 3mf package).
    pub path: String,

    /// Pixel data, filled on demand.
    pub data: Option<Vec<u8>>,

    /// MIME type, when known.
    pub mime: Option<String>,
}

/// Scene unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Unit {
    /// Millimetre.
    #[default]
    Millimeter,
    /// Centimetre.
    Centimeter,
    /// Metre.
    Meter,
    /// Inch.
    Inch,
}

/// Scene up axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    /// X up.
    X,
    /// Y up.
    Y,
    /// Z up.
    #[default]
    Z,
}

/// Axis aligned bounding box, named after engine's `Aabb`.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    /// Minimum corner.
    pub min: [f32; 3],
    /// Maximum corner.
    pub max: [f32; 3],
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }
}

impl Aabb {
    /// Whether the box holds no point.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min[0] > self.max[0]
    }

    /// Grows the box to contain `p`.
    #[inline]
    pub fn expand(&mut self, p: [f32; 3]) {
        for (i, value) in p.into_iter().enumerate() {
            if value < self.min[i] {
                self.min[i] = value;
            }
            if value > self.max[i] {
                self.max[i] = value;
            }
        }
    }

    /// Grows the box to contain `other`.
    #[inline]
    pub fn union(&mut self, other: &Aabb) {
        if other.is_empty() {
            return;
        }
        for i in 0..3 {
            if other.min[i] < self.min[i] {
                self.min[i] = other.min[i];
            }
            if other.max[i] > self.max[i] {
                self.max[i] = other.max[i];
            }
        }
    }

    /// Size.
    #[inline]
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Centre.
    #[inline]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

/// Extensions, mirroring gltf's `extensions`.
///
/// A `Vec` instead of a map: there are few entries, linear search is faster and
/// it keeps the type dependency free.
#[derive(Clone, Debug, Default)]
pub struct Extensions {
    /// Key/value entries.
    pub entries: Vec<(String, String)>,
}

impl Extensions {
    /// Whether there is no entry.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks a value up.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Inserts or replaces a value.
    #[inline]
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        if let Some(slot) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value.into();
        } else {
            self.entries.push((key, value.into()));
        }
    }
}

impl Scene {
    /// A scene in the normalised convention the loaders produce: millimetres
    /// and Y-up, which is what the engine and the slicer expect.
    pub fn normalized(objects: Vec<Object>) -> Self {
        let mut scene = Self {
            objects,
            unit: Unit::Millimeter,
            upa: Axis::Y,
            ..Default::default()
        };
        scene.recbounds();
        scene
    }

    /// Recomputes the scene bounds from the stored vertex positions.
    pub fn recbounds(&mut self) {
        self.bounds = Aabb::default();
        for o in &self.objects {
            for p in &o.mesh.positions {
                self.bounds.expand(*p);
            }
        }
    }
}

impl From<Object> for Scene {
    /// Wraps a single object into a normalised scene.
    fn from(object: Object) -> Self {
        Self::normalized(vec![object])
    }
}

impl From<Mesh> for Scene {
    /// Wraps a single mesh into a normalised scene.
    fn from(mesh: Mesh) -> Self {
        Self::normalized(vec![Object {
            mesh,
            ..Default::default()
        }])
    }
}
