use std::fmt;
use bevy::{
    prelude::*,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    mesh::{Indices, PrimitiveTopology}, reflect::TypePath,
};
use thdfiler::amf::{reader, error::Error};

pub static EXTENSIONS: &[&str] = &["amf"];

/// Asset containing multiple meshes from an AMF file
#[derive(Asset, TypePath, Debug)]
pub struct AmfAsset {
    pub meshes: Vec<Handle<Mesh>>,
}

/// Plugin to register AMF asset loader
pub struct AmfPlugin;

impl Plugin for AmfPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<AmfLoader>(EXTENSIONS);
        app.init_asset::<AmfAsset>();
    }

    fn finish(&self, app: &mut App) {
        app.register_asset_loader(AmfLoader);
    }
}

/// Asset loader for AMF files
#[derive(Default)]
pub struct AmfLoader;

impl AssetLoader for AmfLoader {
    type Asset = AmfAsset;
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
        
        // Read AMF from bytes
        let document = reader::read_bytes(&bytes)?;
        
        if !document.objects.is_empty() {
            // Convert all AMF objects to Bevy meshes
            let mut view_meshes = Vec::new();
            
            for object in &document.objects {
                let mesh = &object.mesh;
                if !mesh.vertices.vertices.is_empty() && !mesh.volume.triangles.is_empty() {
                    let mut view_mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
                    
                    // Insert mesh data
                    view_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices.get_positions());
                    view_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.vertices.get_normals());
                    view_mesh.insert_indices(Indices::U32(mesh.volume.get_indices()));
                    
                    // Calculate mesh bounds for better rendering
                    view_mesh.duplicate_vertices();
                    view_mesh.compute_flat_normals();
                    
                    view_meshes.push(load_context.add_labeled_asset(AmfLabel::Mesh(view_meshes.len()).to_string(), view_mesh));
                }
            }
            
            Ok(AmfAsset { meshes: view_meshes })
        } else {
            Err(Error::UnexpectedError("No objects found in AMF data".into()))
        }
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmfLabel {
    /// `Mesh.{}`: AMF Mesh as a Bevy [`Mesh`]
    Mesh(usize),
}
 
impl fmt::Display for AmfLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mesh(index) => f.write_str(&format!("Mesh.{}", index)),
        }
    }
}
 
impl AmfLabel {
    /// Add this label to an asset path
    pub fn from_asset(&self, path: impl Into<AssetPath<'static>>) -> AssetPath<'static> {
        path.into().with_label(self.to_string())
    }
}