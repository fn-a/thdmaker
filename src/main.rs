use glob::glob;
use clap::Parser;
use bevy::prelude::*;
use thdmaker::util;
use thdmaker::viewer::{
    camera, uireact, slice::SlicePlugin,
    stl::{EXTENSIONS as STL_EXTS, StlPlugin, StlLabel},
    amf::{EXTENSIONS as AMF_EXTS, AmfPlugin, AmfLabel},
    tmf::{EXTENSIONS as TMF_EXTS, TmfPlugin, TmfLabel},
    gcode::{EXTENSIONS as GCODE_EXTS, GCodePlugin, GCodeLabel},
};

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
            .insert_resource(AmbientLight {
                color: Color::srgb(0.8, 0.8, 0.9),
                brightness: 0.3,
                ..default()
            })
            .add_plugins(StlPlugin)
            .add_plugins(AmfPlugin)
            .add_plugins(TmfPlugin)
            .add_plugins(GCodePlugin)
            .add_plugins(SlicePlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update)
            .add_plugins(camera::CameraPlugin)
            .add_plugins(uireact::UIReactPlugin)
            .insert_resource(GlobalState { models })
            .run();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    global_state: Res<GlobalState>,
) {
    let intensity = 1500.0;
    // Key light - main light from top-right
    commands.spawn((
        DirectionalLight {
            illuminance: intensity,
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 0.8, 0.0)),
    ));
    // Fill light - softer light from bottom-left
    commands.spawn((
        DirectionalLight {
            illuminance: intensity * 0.4,
            color: Color::srgb(0.95, 0.95, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.5, -0.6, 0.0)),
    ));
    // Rim light - backlight from behind
    commands.spawn((
        DirectionalLight {
            illuminance: intensity * 0.5,
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.2, 3.14, 0.0)),
    ));

    // Load specified files
    let mut entities = Vec::new();
    for (file, ext) in global_state.models.iter() {
        if TMF_EXTS.contains(&ext.as_str()) {
            let scene = asset_server.load(TmfLabel::Scene.from_asset(file.clone()));
            let entity = commands.spawn(SceneRoot(scene));
            entities.push(entity.id());
        } else if AMF_EXTS.contains(&ext.as_str()) {
            let entity = commands.spawn((
                Mesh3d(asset_server.load(AmfLabel::Mesh(0).from_asset(file.clone()))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 0.5),
                    metallic: 0.0,
                    reflectance: 0.3,
                    perceptual_roughness: 0.8,
                    ..default()
                })),
                Transform::from_translation(Vec3::ZERO),
            ));
            entities.push(entity.id());
        } else if STL_EXTS.contains(&ext.as_str()) {
            let entity = commands.spawn((
                Mesh3d(asset_server.load(StlLabel::Mesh(0).from_asset(file.clone()))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 0.5),
                    metallic: 0.0,
                    reflectance: 0.3,
                    perceptual_roughness: 0.8,
                    ..default()
                })),
                Transform::from_translation(Vec3::ZERO),
            ));
            entities.push(entity.id());
        } else if GCODE_EXTS.contains(&ext.as_str()) {
            let mat_asset = GCodeLabel::Material(0).from_asset(file.clone());
            let mat_handle = asset_server.load::<StandardMaterial>(mat_asset);
            let entity = commands.spawn((
                Mesh3d(asset_server.load(GCodeLabel::Mesh(0).from_asset(file.clone()))),
                MeshMaterial3d(mat_handle),
                Transform::default(),
            ));
            entities.push(entity.id());
        }
    }

    if entities.is_empty() {
        let material = MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            metallic: 0.0,
            reflectance: 0.3,
            perceptual_roughness: 0.8,
            ..default()
        }));
        let entity = commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
            material.clone(),
            Transform::from_xyz(-8.0, 2.0, -8.0),
        ));
        entities.push(entity.id());
        let entity = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(2.0))),
            material.clone(),
            Transform::from_xyz(8.0, 2.0, -8.0),
        ));
        entities.push(entity.id());
        let entity = commands.spawn((
            Mesh3d(meshes.add(Extrusion::new(Annulus::new(2.0, 4.0), 2.0))),
            material,
            // Transform::from_xyz(0.0, 4.0, 0.0),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, 1.0, 0.0)),
        ));
        entities.push(entity.id());
    }
}

fn update(mut gizmos: Gizmos) {
    gizmos.grid(
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(20, 20),
        Vec2::new(1.0, 1.0),
        Color::srgb(0.5, 0.5, 0.5),
    ).outer_edges();
    
    gizmos.axes(
        Transform::from_translation(Vec3::ZERO),
        2.0,
    );
}