use std::fmt;
use std::iter;
use bevy::{
    prelude::*,
    camera::primitives::MeshAabb,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    mesh::{Indices, PrimitiveTopology},
    scene::Scene, reflect::TypePath,
};
use crate::tmf::{reader, error::Error};
use crate::tmf::define::model::ObjectType;

pub static EXTENSIONS: &[&str] = &["3mf"];

/// Plugin to register 3MF asset loader
pub struct TmfPlugin;

impl Plugin for TmfPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<TmfLoader>(EXTENSIONS);
        app.init_asset::<TmfAsset>();
    }

    fn finish(&self, app: &mut App) {
        app.register_asset_loader(TmfLoader);
    }
}

/// Asset containing a scene with meshes from a 3MF file
#[derive(Asset, TypePath, Debug)]
pub struct TmfAsset {
    pub scene: Handle<Scene>,
}

/// Asset loader for 3MF files
#[derive(Default)]
pub struct TmfLoader;

impl AssetLoader for TmfLoader {
    type Asset = TmfAsset;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        
        // Read 3MF from bytes
        let package = reader::read_bytes(&bytes)?;

        // Create a world to build the scene
        let mut world = World::new();

        let material = StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            metallic: 0.0,
            reflectance: 0.3,
            perceptual_roughness: 0.8,
            ..default()
        };

        let mut index = 0;
        let mut glbmin = Vec3::splat(f32::MAX);
        let mut glbmax = Vec3::splat(f32::MIN);
        let mut entities = Vec::new();

        // Process all models
        for model in iter::once(&package.model).chain(package.models.values()) {
            if !model.build.items.is_empty() {
                let obj_models = model.get_objects(ObjectType::Model);
                for obj_model in &obj_models {
                    if let Some(mesh) = obj_model.get_mesh() {
                        // Add mesh to world
                        let mut view_mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
                        view_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.get_positions());
                        view_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.get_normals());
                        view_mesh.insert_indices(Indices::U32(mesh.get_indices()));
                        view_mesh.duplicate_vertices();
                        view_mesh.compute_flat_normals();
                        
                        let mesh_aabb = view_mesh.compute_aabb();

                        // Add mesh to load_context to get a handle
                        let mesh_handle = load_context.add_labeled_asset(TmfLabel::Mesh(index).to_string(), view_mesh);
                        let mat_handle = load_context.add_labeled_asset(TmfLabel::Material(index).to_string(), material.clone());
                        index += 1;

                        let transform = if let Some(trans) = model.build.get_transform(obj_model.id) {
                            Transform::from_matrix(Mat4::from_cols_array(&trans.matrix_array_4x4()))
                        } else {
                            Transform::default()
                        };

                        if let Some(aabb) = mesh_aabb {
                            let cormin = aabb.min();
                            let cormax = aabb.max();
                            
                            let corners = [
                                Vec3::new(cormin.x, cormin.y, cormin.z), Vec3::new(cormin.x, cormin.y, cormax.z),
                                Vec3::new(cormin.x, cormax.y, cormin.z), Vec3::new(cormin.x, cormax.y, cormax.z),
                                Vec3::new(cormax.x, cormin.y, cormin.z), Vec3::new(cormax.x, cormin.y, cormax.z),
                                Vec3::new(cormax.x, cormax.y, cormin.z), Vec3::new(cormax.x, cormax.y, cormax.z),
                            ];

                            for corner in corners {
                                let wodpos = transform.transform_point(corner);
                                glbmin = glbmin.min(wodpos);
                                glbmax = glbmax.max(wodpos);
                            }
                        }

                        entities.push(world.spawn((
                            Mesh3d(mesh_handle),
                            MeshMaterial3d(mat_handle),
                            transform,
                        )).id());
                    }
                }
            }
        }
        
        if index > 0 {
            if glbmin.x != f32::MAX {
                // Calculate bottom center: X and Z at center, Y at minimum (bottom)
                let center = Vec3::new((glbmin.x + glbmax.x) / 2.0, glbmin.y, (glbmin.z + glbmax.z) / 2.0);
                for entity in entities {
                    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                        transform.translation -= center;
                    }
                }
            }
            // Create scene from the world
            let scene = Scene::new(world);
            let scene_handle = load_context.add_labeled_asset(TmfLabel::Scene.to_string(), scene);
            
            Ok(TmfAsset { scene: scene_handle })
        } else {
            Err(Error::InvalidStructure("No valid meshes found in 3MF data".into()))
        }
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmfLabel {
    /// `Scene`: 3MF Scene
    Scene,
    /// `Mesh.{}`: 3MF Mesh
    Mesh(usize),
    /// `Material.{}`: 3MF Material
    Material(usize),
}
 
impl fmt::Display for TmfLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scene => f.write_str("Scene"),
            Self::Mesh(index) => write!(f, "Mesh.{}", index),
            Self::Material(index) => write!(f, "Material.{}", index),
        }
    }
}
 
impl TmfLabel {
    /// Add this label to an asset path
    pub fn from_asset(&self, path: impl Into<AssetPath<'static>>) -> AssetPath<'static> {
        path.into().with_label(self.to_string())
    }
}