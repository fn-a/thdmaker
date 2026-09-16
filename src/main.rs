use glob::glob;
use clap::Parser;
use bevy::prelude::*;
use thdfiler::util;
use thdviewer::{
    camera::CameraPlugin, ui::UIPlugin, slice::SlicePlugin,
    gizmo::GizmoPlugin, tools::ToolsPlugin,
    stl::{EXTENSIONS as STL_EXTS, StlPlugin, StlLabel},
    amf::{EXTENSIONS as AMF_EXTS, AmfPlugin, AmfLabel},
    tmf::{EXTENSIONS as TMF_EXTS, TmfPlugin, TmfLabel},
    gcode::{EXTENSIONS as GCODE_EXTS, GCodePlugin, GCodeLabel},
};

mod shapes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List all available model files without launching the viewer
    #[arg(short, long)]
    list: bool,

    /// Model files to load (e.g., "models/boat.3mf")
    #[arg(name = "FILES")]
    files: Vec<String>,
}

#[derive(Resource, Clone)]
struct GlobalState {
    models: Vec<(String, String)>,
}

fn main() {
    let args = Args::parse();

    let supports = []
        .iter()
        .chain(TMF_EXTS.iter())
        .chain(AMF_EXTS.iter())
        .chain(STL_EXTS.iter())
        .chain(GCODE_EXTS.iter())
        .collect::<Vec<_>>();
    // If list mode is enabled, exit without launching the viewer
    if args.list {
        println!("Available model files:");
        // Search for files
        for ext in supports {
            if let Ok(entries) = glob(&format!("assets/**/*.{}", ext)) {
                println!("{} files:", ext.to_uppercase());
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(model) = entry.to_str() {
                        println!("  - {}", model);
                    }
                }
            }
        }
    } else {
        let mut models = Vec::new();
        for file in &args.files {
            let ext = util::path::extension(file);
            if supports.contains(&&ext.as_str()) {
                models.push((file.clone(), ext));
            }
        }

        App::new()
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ThdMaker Viewer".into(),
                    ..default()
                }),
                ..default()
            }))
            // Set background color
            .insert_resource(ClearColor(Color::srgb(0.8, 0.8, 0.85)))
            // Ambient light - provides base illumination
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb(0.8, 0.8, 0.9),
                brightness: 0.3,
                ..default()
            })
            .add_plugins((StlPlugin, AmfPlugin, TmfPlugin, GCodePlugin, SlicePlugin))
            .add_plugins((CameraPlugin, UIPlugin, GizmoPlugin, ToolsPlugin))
            .add_systems(Startup, setup)
            .insert_resource(GlobalState { models })
            .run();
    }
}

// Key light - main light from top-right
fn key_light() -> impl Scene {
    bsn! {
        DirectionalLight {
            illuminance: 1500.0,
            color: Color::WHITE,
        }
        Transform { rotation: Quat::from_euler(EulerRot::XYZ, -0.6, 0.8, 0.0) }
    }
}

// Fill light - softer light from bottom-left
fn fill_light() -> impl Scene {
    bsn! {
        DirectionalLight {
            illuminance: 600.0,
            color: Color::srgb(0.95, 0.95, 1.0),
        }
        Transform { rotation: Quat::from_euler(EulerRot::XYZ, 0.5, -0.6, 0.0) }
    }
}

// Rim light - backlight from behind
fn rim_light() -> impl Scene {
    bsn! {
        DirectionalLight {
            illuminance: 750.0,
            color: Color::WHITE,
        }
        Transform { rotation: Quat::from_euler(EulerRot::XYZ, 0.2, std::f32::consts::PI, 0.0) }
    }
}

// A gray material
fn material_gray(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        metallic: 0.0,
        reflectance: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    })
}

// A model entity placed at the given position
fn assemble_model(
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    translation: Vec3,
) -> impl Scene {
    bsn! {
        Mesh3d(mesh)
        MeshMaterial3d::<StandardMaterial>(material)
        Transform { translation: translation }
    }
}

// Fallback demo shapes shown when no models were loaded
fn fallback_scene(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl SceneList {
    let material = material_gray(materials);
    shapes::demo(meshes, material)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    global_state: Res<GlobalState>,
) {
    // Three-point lighting rig
    commands.spawn_scene_list(bsn_list![key_light(), fill_light(), rim_light(),]);

    // Load specified files
    let mut loaded = false;
    for (file, ext) in global_state.models.iter() {
        if TMF_EXTS.contains(&ext.as_str()) {
            let scene = asset_server.load(TmfLabel::Scene.from_asset(file.clone()));
            commands.spawn_scene(bsn! {
                WorldAssetRoot(scene)
                Name(format!("Model: {file}"))
            });
            loaded = true;
        } else if AMF_EXTS.contains(&ext.as_str()) {
            let mesh = asset_server.load(AmfLabel::Mesh(0).from_asset(file.clone()));
            let material = material_gray(&mut materials);
            commands.spawn_scene(assemble_model(mesh, material, Vec3::ZERO));
            loaded = true;
        } else if STL_EXTS.contains(&ext.as_str()) {
            let mesh = asset_server.load(StlLabel::Mesh(0).from_asset(file.clone()));
            let material = material_gray(&mut materials);
            commands.spawn_scene(assemble_model(mesh, material, Vec3::ZERO));
            loaded = true;
        } else if GCODE_EXTS.contains(&ext.as_str()) {
            let mesh = asset_server.load(GCodeLabel::Mesh(0).from_asset(file.clone()));
            let material = asset_server.load(GCodeLabel::Material(0).from_asset(file.clone()));
            commands.spawn_scene(assemble_model(mesh, material, Vec3::ZERO));
            loaded = true;
        }
    }

    if !loaded {
        commands.spawn_scene_list(fallback_scene(&mut meshes, &mut materials));
    }
}