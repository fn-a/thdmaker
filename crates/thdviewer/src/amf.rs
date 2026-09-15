use std::fmt;
use bevy::{
    prelude::*,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    reflect::TypePath,
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
#[derive(Default, TypePath)]
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
            // The bridge normalises the units to millimetres, resolves the
            // constellation instances and bakes their transforms into the
            // vertices.
            let mut view_meshes = Vec::new();

            for mut view_mesh in Vec::<Mesh>::from(&document) {
                // Normals are optional: derive flat normals when the file
                // did not carry one for every vertex.
                if view_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_none() {
                    view_mesh.duplicate_vertices();
                    view_mesh.compute_flat_normals();
                }

                let view_label = AmfLabel::Mesh(view_meshes.len()).to_string();
                view_meshes.push(load_context.add_labeled_asset(view_label, view_mesh));
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