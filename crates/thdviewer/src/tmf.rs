use std::fmt;
use bevy::{
    prelude::*,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    world_serialization::WorldAsset, reflect::TypePath,
};
use thdfiler::tmf::{reader, error::Error};

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
    pub scene: Handle<WorldAsset>,
}

/// Asset loader for 3MF files
#[derive(Default, TypePath)]
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

        // The bridge resolves the build items, the component hierarchy and the
        // units, groups the materials into vertex colours and bakes every
        // transform into the vertices.
        let meshes: Vec<Mesh> = (&package).into();

        if !meshes.is_empty() {
            // World bounds of the model, used for the bottom centering below.
            let mut glbmin = Vec3::splat(f32::MAX);
            let mut glbmax = Vec3::splat(f32::MIN);
            for mesh in &meshes {
                if let Some(positions) = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|v| v.as_float3())
                {
                    for p in positions {
                        let p = Vec3::from_array(*p);
                        glbmin = glbmin.min(p);
                        glbmax = glbmax.max(p);
                    }
                }
            }
    
            // Bottom center: X and Z at center, Y at minimum (bottom)
            let offset = if glbmin.x == f32::MAX {
                Vec3::ZERO
            } else {
                Vec3::new(
                    (glbmin.x + glbmax.x) / 2.0,
                    glbmin.y,
                    (glbmin.z + glbmax.z) / 2.0,
                )
            };
    
            let gray = StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.5),
                metallic: 0.0,
                reflectance: 0.3,
                perceptual_roughness: 0.8,
                ..default()
            };
    
            // Meshes whose materials became vertex colours need a white base.
            let white = StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            };
    
            // Create a world to build the scene
            let mut world = World::new();
    
            for (index, mut view_mesh) in meshes.into_iter().enumerate() {
                // 3MF carries no normals: derive flat ones, which suits the hard
                // edges of CAD geometry.
                if view_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_none() {
                    view_mesh.duplicate_vertices();
                    view_mesh.compute_flat_normals();
                }
    
                let material = if view_mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some() {
                    white.clone()
                } else {
                    gray.clone()
                };
    
                let mesh_handle = load_context.add_labeled_asset(TmfLabel::Mesh(index).to_string(), view_mesh);
                let mat_handle = load_context.add_labeled_asset(TmfLabel::Material(index).to_string(), material);
    
                world.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(mat_handle),
                    Transform::from_translation(-offset),
                ));
            }
    
            // Create scene from the world
            let scene = WorldAsset::new(world);
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