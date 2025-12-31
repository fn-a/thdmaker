//! Core 3MF data model structures.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};
use super::primitive::*;
use super::booleanoperations::BooleanShape;
use super::displacement::{DisplacementResources, DisplacementMesh};
use super::material::MaterialResources;
use super::production::{ProductionItem, ProductionComponent, ProductionObject, ProductionBuild, Alternatives};
use super::slice::{SliceResources, SliceObject};
use super::volumetric::{VolumetricResources, LevelSet};
use super::implicit::ImplicitResources;

/// The root 3MF model containing all resources and build instructions.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Unit of measurement.
    pub unit: Unit,
    /// Language code (e.g., "en-US").
    pub language: Option<String>,
    /// Required extensions.
    pub required_extensions: Vec<String>,
    /// Recommended extensions.
    pub recommended_extensions: Vec<String>,
    /// Model metadata.
    pub metadatas: Vec<Metadata>,
    /// Resources (materials, objects).
    pub resources: Resources,
    /// Build instructions.
    pub build: Build,
}

impl Model {
    /// Create a new empty model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new model with the specified unit.
    pub fn with_unit(unit: Unit) -> Self {
        Self {
            unit,
            ..Default::default()
        }
    }

    /// Add metadata to the model.
    pub fn add_metadata(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadatas.push(Metadata {
            name: name.into(),
            value: value.into(),
            preserve: false,
            r#type: None,
        });
        self
    }

    /// Set the title metadata.
    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.add_metadata("Title", title)
    }

    /// Set the designer metadata.
    pub fn set_designer(&mut self, designer: impl Into<String>) -> &mut Self {
        self.add_metadata("Designer", designer)
    }

    /// Set the description metadata.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.add_metadata("Description", description)
    }

    /// Set the application metadata.
    pub fn set_application(&mut self, application: impl Into<String>) -> &mut Self {
        self.add_metadata("Application", application)
    }

    /// Add a base material group and return its ID.
    pub fn add_base_materials(&mut self, materials: Vec<BaseMaterial>) -> u32 {
        let id = self.resources.next_id();
        self.resources.base_materials.push(BaseMaterials {
            id,
            materials,
            display_properties_id: None,
        });
        id
    }

    /// Add an object and return its ID.
    pub fn add_object(&mut self, object: Object) -> u32 {
        let id = object.id;
        self.resources.objects.push(object);
        self.resources.set_next_id(id + 1);
        id
    }

    /// Create a new object with auto-assigned ID.
    pub fn gen_object(&mut self) -> Object {
        Object::new(self.resources.next_id())
    }

    /// Add an object to the build by ID with optional transform.
    pub fn add_build_item(&mut self, object_id: u32, transform: Option<Matrix3D>) -> &mut Self {
        self.build.items.push(Item::with_transform(object_id, transform));
        self
    }

    /// Get an object by ID.
    pub fn get_object(&self, id: u32) -> Option<&Object> {
        self.resources.objects.iter().find(|o| o.id == id)
    }

    /// Get a mutable object by ID.
    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut Object> {
        self.resources.objects.iter_mut().find(|o| o.id == id)
    }

    /// Get base materials by ID.
    pub fn get_base_material(&self, id: u32) -> Option<&BaseMaterials> {
        self.resources.base_materials.iter().find(|m| m.id == id)
    }

    /// Get mutable base materials by ID.
    pub fn get_base_material_mut(&mut self, id: u32) -> Option<&mut BaseMaterials> {
        self.resources.base_materials.iter_mut().find(|m| m.id == id)
    }

    /// Get displacement resources.
    pub fn get_displacement(&self) -> Option<&DisplacementResources> {
        self.resources.displacements.as_ref()
    }

    /// Get mutable displacement resources.
    pub fn get_displacements_mut(&mut self) -> Option<&mut DisplacementResources> {
        self.resources.displacements.as_mut()
    }

    /// Get materials resources.
    pub fn get_materials(&self) -> Option<&MaterialResources> {
        self.resources.materials.as_ref()
    }

    /// Get mutable materials resources.
    pub fn get_materials_mut(&mut self) -> Option<&mut MaterialResources> {
        self.resources.materials.as_mut()
    }

    /// Get volumetric resources.
    pub fn get_volumetric(&self) -> Option<&VolumetricResources> {
        self.resources.volumetric.as_ref()
    }

    /// Get mutable volumetric resources.
    pub fn get_volumetric_mut(&mut self) -> Option<&mut VolumetricResources> {
        self.resources.volumetric.as_mut()
    }

    /// Get implicit resources.
    pub fn get_implicit(&self) -> Option<&ImplicitResources> {
        self.resources.implicit.as_ref()
    }

    /// Get mutable implicit resources.
    pub fn get_implicit_mut(&mut self) -> Option<&mut ImplicitResources> {
        self.resources.implicit.as_mut()
    }

    /// Validate the model structure.
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate resource IDs
        let mut ids = std::collections::HashSet::new();
        for obj in &self.resources.objects {
            if !ids.insert(obj.id) {
                return Err(Error::DuplicateResourceId(obj.id));
            }
        }
        for mat in &self.resources.base_materials {
            if !ids.insert(mat.id) {
                return Err(Error::DuplicateResourceId(mat.id));
            }
        }

        // Validate build items reference valid objects
        for item in &self.build.items {
            if self.get_object(item.object_id).is_none() {
                return Err(Error::InvalidReference(format!(
                    "build item references non-existent object: {}",
                    item.object_id
                )));
            }
        }

        // Validate object types in build
        for item in &self.build.items {
            if let Some(obj) = self.get_object(item.object_id) {
                if obj.r#type == ObjectType::Other {
                    return Err(Error::InvalidReference(format!(
                        "build item cannot reference 'other' type object: {}",
                        item.object_id
                    )));
                }
            }
        }

        // Validate boolean shape objects
        for obj in &self.resources.objects {
            if obj.is_boolean_shape() {
                // Boolean shape objects cannot have pid or pindex
                if obj.pid.is_some() || obj.pindex.is_some() {
                    return Err(Error::InvalidAttribute {
                        name: "pid/pindex".to_string(),
                        message: format!(
                            "boolean shape object {} cannot have material properties",
                            obj.id
                        ),
                    });
                }

                // Validate boolean shape references
                if let Some(boolean_shape) = obj.get_boolean_shape() {
                    // Validate base object reference
                    if self.get_object(boolean_shape.object_id).is_none() {
                        return Err(Error::InvalidReference(format!(
                            "boolean shape {} references non-existent base object: {}",
                            obj.id, boolean_shape.object_id
                        )));
                    }

                    // Validate boolean operation references
                    for (i, boolean) in boolean_shape.booleans.iter().enumerate() {
                        if self.get_object(boolean.object_id).is_none() {
                            return Err(Error::InvalidReference(format!(
                                "boolean shape {} boolean[{}] references non-existent object: {}",
                                obj.id, i, boolean.object_id
                            )));
                        }
                    }

                    // Validate the boolean shape itself
                    if let Err(e) = boolean_shape.validate() {
                        return Err(Error::InvalidStructure(format!(
                            "boolean shape {} validation failed: {}",
                            obj.id, e
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// A group of metadata.
#[derive(Debug, Clone, Default)]
pub struct MetadataGroup {
    /// Metadata group.
    pub metadatas: Vec<Metadata>,
}

impl MetadataGroup {
    /// Create a new metadata group.
    pub fn new() -> Self {
        Self { metadatas: Vec::new() }
    }
}

/// Model metadata.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// Metadata name (e.g., "Title", "Designer").
    pub name: String,
    /// Metadata value.
    pub value: String,
    /// Whether to preserve this metadata on modification.
    pub preserve: bool,
    /// Optional data type (default: xs:string).
    pub r#type: Option<String>,
}

/// Collection of resources.
#[derive(Debug, Clone, Default)]
pub struct Resources {
    /// Base material groups.
    pub base_materials: Vec<BaseMaterials>,
    /// Optional displacement extension resources.
    pub displacements: Option<DisplacementResources>,
    /// Optional materials and properties extension resources.
    pub materials: Option<MaterialResources>,
    /// Optional slice extension resources.
    pub slices: Option<SliceResources>,
    /// Optional volumetric extension resources.
    pub volumetric: Option<VolumetricResources>,
    /// Optional implicit extension resources.
    pub implicit: Option<ImplicitResources>,
    /// Object definitions.
    pub objects: Vec<Object>,
    /// Internal ID counter.
    next_id: u32,
}

impl Resources {
    /// Get the next available resource ID.
    pub fn next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// Set the next ID (used when loading).
    pub fn set_next_id(&mut self, id: u32) {
        if id >= self.next_id {
            self.next_id = id;
        }
    }
}

/// Collection of base materials.
#[derive(Debug, Clone)]
pub struct BaseMaterials {
    /// Unique resource ID.
    pub id: u32,
    /// Individual materials.
    pub materials: Vec<BaseMaterial>,
    /// Optional material extension reference to display properties.
    pub display_properties_id: Option<u32>,
}

impl BaseMaterials {
    /// Create a new base materials group.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            materials: Vec::new(),
            display_properties_id: None,
        }
    }

    /// Add a material.
    pub fn add(&mut self, name: impl Into<String>, display_color: Color) -> usize {
        let index = self.materials.len();
        self.materials.push(BaseMaterial {
            name: name.into(),
            display_color,
        });
        index
    }
}

/// A single base material.
#[derive(Debug, Clone)]
pub struct BaseMaterial {
    /// Material name.
    pub name: String,
    /// Display color for rendering.
    pub display_color: Color,
}

impl BaseMaterial {
    /// Create a new base material.
    pub fn new(name: impl Into<String>, display_color: Color) -> Self {
        Self {
            name: name.into(),
            display_color,
        }
    }
}

/// Build instructions.
#[derive(Debug, Clone, Default)]
pub struct Build {
    /// Optional production extension data.
    pub production: Option<ProductionBuild>,
    /// Items to build.
    pub items: Vec<Item>,
}

impl Build {
    /// Create a new empty build.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to build.
    pub fn add_item(&mut self, item: Item) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Add an object by ID.
    pub fn add_object(&mut self, object_id: u32) -> &mut Self {
        self.items.push(Item::new(object_id));
        self
    }

    /// Add an object with transform.
    pub fn add_object_with_transform(&mut self, object_id: u32, transform: Matrix3D) -> &mut Self {
        self.items.push(Item {
            object_id,
            transform: Some(transform),
            part_number: None,
            production: None,
            metadata_group: MetadataGroup::new(),
        });
        self
    }
}

/// A single build item.
#[derive(Debug, Clone)]
pub struct Item {
    /// Reference to object ID.
    pub object_id: u32,
    /// Optional transformation matrix.
    pub transform: Option<Matrix3D>,
    /// Optional part number.
    pub part_number: Option<String>,
    /// Optional production extension data.
    pub production: Option<ProductionItem>,
    /// Item metadata group.
    pub metadata_group: MetadataGroup,
}

impl Item {
    /// Create a new item.
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            transform: None,
            part_number: None,
            production: None,
            metadata_group: MetadataGroup::new(),
        }
    }

    /// Create an item with transform.
    pub fn with_transform(object_id: u32, transform: Option<Matrix3D>) -> Self {
        Self {
            object_id,
            transform,
            part_number: None,
            production: None,
            metadata_group: MetadataGroup::new(),
        }
    }

    /// Set the transform.
    pub fn set_transform(&mut self, transform: Matrix3D) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set the part number.
    pub fn set_part_number(&mut self, part_number: impl Into<String>) -> &mut Self {
        self.part_number = Some(part_number.into());
        self
    }
}

/// A 3D object resource.
#[derive(Debug, Clone)]
pub struct Object {
    /// Unique resource ID.
    pub id: u32,
    /// Object type.
    pub r#type: ObjectType,
    /// Optional thumbnail path.
    pub thumbnail: Option<String>,
    /// Optional part number.
    pub part_number: Option<String>,
    /// Optional object name.
    pub name: Option<String>,
    /// Default property group ID.
    pub pid: Option<u32>,
    /// Default property index.
    pub pindex: Option<u32>,
    /// Optional displacement resource ID.
    pub did: Option<u32>,
    /// Optional production extension data.
    pub production: Option<ProductionObject>,
    /// Optional slice extension data.
    pub slice: Option<SliceObject>,
    /// Object metadata.
    pub metadata_group: MetadataGroup,
    /// Object content (mesh or components or extension structure).
    pub content: ObjectContent,
}

impl Object {
    /// Create a new object.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            r#type: ObjectType::Model,
            thumbnail: None,
            part_number: None,
            name: None,
            pid: None,
            pindex: None,
            did: None,
            production: None,
            slice: None,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::Mesh(Mesh::new()),
        }
    }

    /// Create a new object with a mesh.
    pub fn with_mesh(id: u32, mesh: Mesh) -> Self {
        Self {
            id,
            r#type: ObjectType::Model,
            thumbnail: None,
            part_number: None,
            name: None,
            pid: None,
            pindex: None,
            did: None,
            production: None,
            slice: None,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::Mesh(mesh),
        }
    }

    /// Create a new object with components.
    pub fn with_components(id: u32, components: Components) -> Self {
        Self {
            id,
            r#type: ObjectType::Model,
            thumbnail: None,
            part_number: None,
            name: None,
            pid: None,
            pindex: None,
            did: None,
            production: None,
            slice: None,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::Components(components),
        }
    }

    /// Create a new object with a boolean shape.
    pub fn with_boolean_shape(id: u32, boolean_shape: BooleanShape) -> Self {
        Self {
            id,
            r#type: ObjectType::Model,
            thumbnail: None,
            part_number: None,
            name: None,
            pid: None,
            pindex: None,
            did: None,
            production: None,
            slice: None,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::BooleanShape(boolean_shape),
        }
    }

    /// Create a new object with a displacement mesh.
    pub fn with_displacement_mesh(id: u32, displacement_mesh: DisplacementMesh) -> Self {
        Self {
            id,
            r#type: ObjectType::Model,
            thumbnail: None,
            part_number: None,
            name: None,
            pid: None,
            pindex: None,
            did: None,
            production: None,
            slice: None,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::DisplacementMesh(displacement_mesh),
        }
    }

    /// Check if this object is a mesh.
    pub fn is_mesh(&self) -> bool {
        matches!(self.content, ObjectContent::Mesh(_))
    }

    /// Check if this object has components.
    pub fn is_components(&self) -> bool {
        matches!(self.content, ObjectContent::Components(_))
    }

    /// Check if this object is a boolean shape.
    pub fn is_boolean_shape(&self) -> bool {
        matches!(self.content, ObjectContent::BooleanShape(_))
    }

    /// Check if this object is a displacement mesh.
    pub fn is_displacement_mesh(&self) -> bool {
        matches!(self.content, ObjectContent::DisplacementMesh(_))
    }

    /// Check if this object is alternatives.
    pub fn is_alternatives(&self) -> bool {
        matches!(self.content, ObjectContent::Alternatives(_))
    }

    /// Get the mesh if this object contains one.
    pub fn get_mesh(&self) -> Option<&Mesh> {
        match &self.content {
            ObjectContent::Mesh(mesh) => Some(mesh),
            _ => None,
        }
    }

    /// Get a mutable mesh if this object contains one.
    pub fn get_mesh_mut(&mut self) -> Option<&mut Mesh> {
        match &mut self.content {
            ObjectContent::Mesh(mesh) => Some(mesh),
            _ => None,
        }
    }

    /// Get the components if this object contains them.
    pub fn get_components(&self) -> Option<&Components> {
        match &self.content {
            ObjectContent::Components(components) => Some(components),
            _ => None,
        }
    }

    /// Get the boolean shape if this object contains one.
    pub fn get_boolean_shape(&self) -> Option<&BooleanShape> {
        match &self.content {
            ObjectContent::BooleanShape(boolean_shape) => Some(boolean_shape),
            _ => None,
        }
    }

    /// Get a mutable boolean shape if this object contains one.
    /// Get a mutable boolean shape if this object contains one.
    pub fn get_boolean_shape_mut(&mut self) -> Option<&mut BooleanShape> {
        match &mut self.content {
            ObjectContent::BooleanShape(boolean_shape) => Some(boolean_shape),
            _ => None,
        }
    }

    /// Get the displacement mesh if this object contains one.
    pub fn get_displacement_mesh(&self) -> Option<&DisplacementMesh> {
        match &self.content {
            ObjectContent::DisplacementMesh(displacement_mesh) => Some(displacement_mesh),
            _ => None,
        }
    }

    /// Get a mutable displacement mesh if this object contains one.
    pub fn get_displacement_mesh_mut(&mut self) -> Option<&mut DisplacementMesh> {
        match &mut self.content {
            ObjectContent::DisplacementMesh(displacement_mesh) => Some(displacement_mesh),
            _ => None,
        }
    }

    /// Get the alternatives if this object contains them.
    pub fn get_alternatives(&self) -> Option<&Alternatives> {
        match &self.content {
            ObjectContent::Alternatives(alternatives) => Some(alternatives),
            _ => None,
        }
    }

    /// Get a mutable alternatives if this object contains them.
    pub fn get_alternatives_mut(&mut self) -> Option<&mut Alternatives> {
        match &mut self.content {
            ObjectContent::Alternatives(alternatives) => Some(alternatives),
            _ => None,
        }
    }

    /// Set the mesh.
    pub fn set_mesh(&mut self, mesh: Mesh) -> &mut Self {
        self.content = ObjectContent::Mesh(mesh);
        self
    }

    /// Set the components.
    pub fn set_components(&mut self, components: Components) -> &mut Self {
        self.content = ObjectContent::Components(components);
        self
    }

    /// Set the boolean shape.
    pub fn set_boolean_shape(&mut self, boolean_shape: BooleanShape) -> &mut Self {
        self.content = ObjectContent::BooleanShape(boolean_shape);
        self
    }

    /// Set the displacement mesh.
    pub fn set_displacement_mesh(&mut self, displacement_mesh: DisplacementMesh) -> &mut Self {
        self.content = ObjectContent::DisplacementMesh(displacement_mesh);
        self
    }

    /// Set the a object.
    pub fn set_alternatives(&mut self, alternatives: Alternatives) -> &mut Self {
        self.content = ObjectContent::Alternatives(alternatives);
        self
    }

    /// Set the object name.
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the object type.
    pub fn set_type(&mut self, object_type: ObjectType) -> &mut Self {
        self.r#type = object_type;
        self
    }

    /// Set the default material.
    pub fn set_material(&mut self, pid: u32, pindex: u32) -> &mut Self {
        self.pid = Some(pid);
        self.pindex = Some(pindex);
        self
    }
}

/// Object type in the 3MF model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectType {
    /// Standard model object to be manufactured.
    #[default]
    Model,
    /// Solid support structure (filled).
    Solidsupport,
    /// Support structure (surface only, not filled).
    Support,
    /// Surface object (not a solid).
    Surface,
    /// Other object type (cannot be built directly).
    Other,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Model => write!(f, "model"),
            ObjectType::Solidsupport => write!(f, "solidsupport"),
            ObjectType::Support => write!(f, "support"),
            ObjectType::Surface => write!(f, "surface"),
            ObjectType::Other => write!(f, "other"),
        }
    }
}

impl FromStr for ObjectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "model" => Ok(ObjectType::Model),
            "solidsupport" => Ok(ObjectType::Solidsupport),
            "support" => Ok(ObjectType::Support),
            "surface" => Ok(ObjectType::Surface),
            "other" => Ok(ObjectType::Other),
            _ => Err(Error::InvalidAttribute {
                name: "type".to_string(),
                message: format!("unknown object type: {}", s),
            }),
        }
    }
}

/// Object content: either a mesh, components, boolean shape, or displacement mesh.
#[derive(Debug, Clone)]
pub enum ObjectContent {
    /// Triangle mesh.
    Mesh(Mesh),
    /// Component assembly.
    Components(Components),
    /// Boolean shape extension.
    BooleanShape(BooleanShape),
    /// Displacement mesh extension.
    DisplacementMesh(DisplacementMesh),
    /// Production alternative extension.
    Alternatives(Alternatives),
    /// Volumetric levelset extension.
    LevelSet(LevelSet),
}

impl Default for ObjectContent {
    fn default() -> Self {
        ObjectContent::Mesh(Mesh::default())
    }
}

/// A group of components.
#[derive(Debug, Clone)]
pub struct  Components {
    /// Components.
    pub components: Vec<Component>,
}

/// A component reference.
#[derive(Debug, Clone)]
pub struct Component {
    /// Referenced object ID.
    pub object_id: u32,
    /// Optional transformation.
    pub transform: Option<Matrix3D>,
    /// Optional production extension data.
    pub production: Option<ProductionComponent>,
}

impl Component {
    /// Create a new component.
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            transform: None,
            production: None,
        }
    }

    /// Create a component with transform.
    pub fn with_transform(object_id: u32, transform: Matrix3D) -> Self {
        Self {
            object_id,
            transform: Some(transform),
            production: None,
        }
    }
}