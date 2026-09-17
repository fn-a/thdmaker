use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use thdfiler::tris;
use thdslicer::{Slicer, Config};

/// Plugin to register slice plugin
pub struct SlicePlugin;

impl Plugin for SlicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default())
            .init_resource::<SliceState>()
            .add_systems(Update, (disperse, wireframe, transparent, sliceview));
    }
}

/// A model: the meshes the viewer's tools work on.
#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SliceModel;

/// Root of a spawned scene whose meshes are models.
#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SliceScene;

/// A model's toolpath, drawn while the slice view is on.
#[derive(Component)]
pub struct SlicePreview;

/// State for slice view
#[derive(Resource, Clone, Default)]
struct SliceState {
    /// Whether wireframe mode is enabled (X key)
    wireframed: bool,
    /// Whether slice mode is enabled (S key)
    sliced: bool,
    /// Whether transparent mode is enabled (V key)
    transparent: bool,
    /// Material the previews are drawn with, made once on first use.
    material: Option<Handle<StandardMaterial>>,
}

/// Mark the meshes that appear under a [`SliceScene`] root as models.
fn disperse(
    mut commands: Commands,
    added: Query<Entity, (With<Mesh3d>, Without<SliceModel>, Added<Mesh3d>)>,
    parents: Query<&ChildOf>,
    scenes: Query<(), With<SliceScene>>,
) {
    for entity in &added {
        let mut cursor = entity;
        while let Ok(relate) = parents.get(cursor) {
            cursor = relate.parent();
            if scenes.contains(cursor) {
                commands.entity(entity).insert(SliceModel);
                break;
            }
        }
    }
}

/// X: outline the models.
fn wireframe(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<SliceState>,
    models: Query<Entity, (With<SliceModel>, With<Mesh3d>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        state.wireframed = !state.wireframed;
    
        for entity in &models {
            if state.wireframed {
                commands.entity(entity).insert((
                    Wireframe,
                    WireframeColor {
                        color: Color::srgb(0.25, 0.95, 1.0),
                    },
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<(Wireframe, WireframeColor)>();
            }
        }
    }
}

/// V: make the models translucent, so the toolpath inside them shows through.
fn transparent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<SliceState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    models: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (With<SliceModel>, With<Mesh3d>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        state.transparent = !state.transparent;
        let alpha = if state.transparent { 0.3 } else { 1.0 };
    
        for (entity, material) in &models {
            // The material itself is edited: it is shared, and every model changes.
            if let Some(mut source) = materials.get_mut(&material.0) {
                source.base_color = source.base_color.with_alpha(alpha);
                source.alpha_mode = if state.transparent {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                };
            }
    
            // While sliced the part is hidden, so this is what brings it back.
            if state.sliced {
                commands.entity(entity).insert(if state.transparent {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}

/// S: draw every model's toolpath.
fn sliceview(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<SliceState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    models: Query<(Entity, &Mesh3d, &GlobalTransform), (With<SliceModel>, With<Mesh3d>)>,
    previews: Query<Entity, With<SlicePreview>>
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        state.sliced = !state.sliced;
    
        if !state.sliced {
            for (entity, ..) in &models {
                commands.entity(entity).insert(Visibility::Visible);
            }
            for entity in &previews {
                commands.entity(entity).despawn();
            }
        } else {
            // One unlit material serves every preview: the colours are per vertex.
            let pviewmatl = match state.material.clone() {
                Some(handle) => handle,
                None => {
                    let handle = materials.add(StandardMaterial {
                        unlit: true,
                        ..default()
                    });
                    state.material = Some(handle.clone());
                    handle
                }
            };
        
            for (entity, mesh, transform) in &models {
                if let Some(source) = meshes.get(&mesh.0) {
                    let config = Config::default();
                    let mut slicer = Slicer::new(&config);
                    // Sliced in world space, the frame the printer works in.
                    let world = transform.to_matrix();
                    if slicer.slice(&tris::Mesh::cheapest(source), &world).is_ok(){
                        let mut preview = slicer.preview();
                        // Baked back into the model's own space, so the child preview
                        // follows every move the handles make.
                        let matrix = world.inverse();
                        for point in preview.positions.iter_mut() {
                            *point = matrix.transform_point3(Vec3::from(*point)).to_array();
                        }
                
                        if !state.transparent {
                            commands.entity(entity).insert(Visibility::Hidden);
                        }

                        commands.spawn((
                            Mesh3d(meshes.add(Mesh::from(preview))),
                            MeshMaterial3d(pviewmatl.clone()),
                            ChildOf(entity),
                            // Visible outright: a visible child wins over a hidden parent.
                            Visibility::Visible,
                            SlicePreview,
                            // A click on the toolpath selects the model, and it blocks
                            // nothing behind it.
                            Pickable {
                                is_hoverable: true,
                                should_block_lower: false,
                            },
                        ));
                    }
                }
            }
        }
    }
}