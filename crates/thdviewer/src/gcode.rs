use std::fmt;
use bevy::{
    prelude::*,
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    mesh::PrimitiveTopology, reflect::TypePath,
};
use thdfiler::gcode::{reader, error::Error, define::Arrow};

pub static EXTENSIONS: &[&str] = &["gcode"];

/// Plugin to register GCode asset loader
pub struct GCodePlugin;

impl Plugin for GCodePlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<GCodeLoader>(EXTENSIONS);
        app.init_asset::<GCodeAsset>();
    }

    fn finish(&self, app: &mut App) {
        app.register_asset_loader(GCodeLoader);
    }
}

/// Asset containing GCode mesh with empty and solid line
#[derive(Asset, TypePath, Debug)]
pub struct GCodeAsset {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// Asset loader for GCode files
#[derive(Default)]
pub struct GCodeLoader;

impl AssetLoader for GCodeLoader {
    type Asset = GCodeAsset;
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
        
        // Read GCode from bytes
        let gdata = reader::read_bytes(&bytes)?;
        
        if !gdata.lines.is_empty() {
            let mut arrow = Arrow::default();
            let mut points: Vec<[f32; 3]> = Vec::new();
            let mut colors: Vec<Vec4> = Vec::new();

            let solid = Color::srgb(0.0, 1.0, 0.0).to_srgba().to_vec4();
            let empty = Color::srgb(0.4, 0.5, 0.6).to_srgba().to_vec4();
            
            for line in gdata.lines.iter() {
                let route = line.route();
                let march = route.march(&mut arrow);
                // Convert points to Y-up coordinates
                let positions = march.get_positions();
                
                if !positions.is_empty() {
                    let length = positions.len();
                    points.extend(positions);
                    if march.solid {
                        colors.extend(vec![solid; length]);
                    } else {
                        colors.extend(vec![empty; length]);
                    }
                }
            }

            let mut line_strip = Mesh::new(PrimitiveTopology::LineStrip, default());
            line_strip.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
            line_strip.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            let mesh_handle = load_context.add_labeled_asset(
                GCodeLabel::Mesh(0).to_string(),
                line_strip,
            );
            let material = StandardMaterial {
                emissive: Color::srgb(0.0, 0.0, 0.0).to_linear(),
                unlit: true,
                ..default()
            };
            let mat_handle = load_context.add_labeled_asset(
                GCodeLabel::Material(0).to_string(),
                material
            );
            Ok(GCodeAsset { mesh: mesh_handle, material: mat_handle })
        } else {
            Err(Error::UnexpectedError("No lines found in GCode data".into()))
        }
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GCodeLabel {
    /// `Mesh.{}`: GCode display as a Bevy line [`Mesh`] 
    Mesh(usize),
    /// `Material.{}`: GCode display material [`StandardMaterial`]
    Material(usize),
}
 
impl fmt::Display for GCodeLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mesh(index) => f.write_str(&format!("Mesh.{}", index)),
            Self::Material(index) => f.write_str(&format!("Material.{}", index)),
        }
    }
}
 
impl GCodeLabel {
    /// Add this label to an asset path
    pub fn from_asset(&self, path: impl Into<AssetPath<'static>>) -> AssetPath<'static> {
        path.into().with_label(self.to_string())
    }
}