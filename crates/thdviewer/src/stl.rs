use std::fmt;
use bevy::{
    prelude::*,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    reflect::TypePath,
};
use thdfiler::stl::{reader, error::Error};

pub static EXTENSIONS: &[&str] = &["stl"];

/// Plugin to register STL asset loader
pub struct StlPlugin;

impl Plugin for StlPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<StlLoader>(EXTENSIONS);
        app.init_asset::<StlAsset>();
    }

    fn finish(&self, app: &mut App) {
        app.register_asset_loader(StlLoader);
    }
}

/// Asset containing multiple meshes from an STL file
#[derive(Asset, TypePath, Debug)]
pub struct StlAsset {
    pub meshes: Vec<Handle<Mesh>>,
}

/// Asset loader for STL files
#[derive(Default, TypePath)]
pub struct StlLoader;

impl AssetLoader for StlLoader {
    type Asset = StlAsset;
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
        
        // Read STL from bytes
        let read_meshes = reader::read_bytes(&bytes)?;
        
        if !read_meshes.is_empty() {
            // The bridge maps the Z-up facets to the Y-up convention, keeps one
            // flat normal per facet (derived from the winding order when the file
            // stores a zero one) and turns facet colours into vertex colours.
            let mut view_meshes = Vec::new();

            for (i, read_mesh) in read_meshes.iter().enumerate() {
                let view_mesh: Mesh = read_mesh.into();
                view_meshes.push(load_context.add_labeled_asset(StlLabel::Mesh(i).to_string(), view_mesh));
            }
            
            Ok(StlAsset { meshes: view_meshes })
        } else {
            Err(Error::UnexpectedError("No meshes found in STL data".into()))
        }
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlLabel {
    /// `Mesh.{}`: STL Mesh as a Bevy [`Mesh`]
    Mesh(usize),
}
 
impl fmt::Display for StlLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mesh(index) => f.write_str(&format!("Mesh.{}", index)),
        }
    }
}
 
impl StlLabel {
    /// Add this label to an asset path
    pub fn from_asset(&self, path: impl Into<AssetPath<'static>>) -> AssetPath<'static> {
        path.into().with_label(self.to_string())
    }
}