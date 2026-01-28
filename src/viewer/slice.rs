use bevy::{prelude::*, color::palettes::basic::AQUA};
use crate::slicer::{Slicer, Config};

/// Plugin to register slice plugin
pub struct SlicePlugin;

impl Plugin for SlicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle)
            .init_resource::<SliceState>();
    }
}

/// State for slice view
#[derive(Resource, Clone, Default)]
struct SliceState {
    /// Whether wireframe mode is enabled (X key)
    wireframed: bool,
    /// Whether slice mode is enabled (S key)
    sliced: bool,
    /// Whether transparent mode is enabled (V key)
    transparent: bool,
    /// Entity for slice mesh display
    entities: Vec<(Entity, Entity)>,
}

/// Toggle mode
fn toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<SliceState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Mesh3d, &Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut gizmos: Gizmos,
) {
    let mut pressed = (false, false);
    if keyboard.just_pressed(KeyCode::KeyX) {
        state.wireframed = !state.wireframed;
    } else if keyboard.just_pressed(KeyCode::KeyS) {
        state.sliced = !state.sliced;
        pressed.0 = true;
    } else if keyboard.just_pressed(KeyCode::KeyV) {
        state.transparent = !state.transparent;
        pressed.1 = true;
    }

    if pressed.0 || pressed.1 || state.wireframed {
        for (entity, handle, transform, material) in query.iter() {
            if let Some(mesh) = meshes.get(&handle.0) {
                if state.wireframed {
                    // Display wireframe for all meshes using gizmos
                    let vertices = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                        .map(|v| v.as_float3().unwrap_or_default())
                        .unwrap_or_default();
                    if let Some(indices) = mesh.indices() {
                        // Draw wireframe lines
                        for index in indices.iter().collect::<Vec<_>>().chunks(3) {
                            if index.len() == 3 {
                                let v0 = vertices[index[0]];
                                let v1 = vertices[index[1]];
                                let v2 = vertices[index[2]];
                                
                                let p0 = Vec3::new(v0[0], v0[1], v0[2]);
                                let p1 = Vec3::new(v1[0], v1[1], v1[2]);
                                let p2 = Vec3::new(v2[0], v2[1], v2[2]);
                                let p0 = transform.transform_point(p0);
                                let p1 = transform.transform_point(p1);
                                let p2 = transform.transform_point(p2);
                                gizmos.line(p0, p1, AQUA);
                                gizmos.line(p1, p2, AQUA);
                                gizmos.line(p2, p0, AQUA);
                            }
                        }
                    }
                }

                if pressed.1 && !state.sliced {
                    if let Some(mat) = materials.get_mut(&material.0) {
                        if state.transparent {
                            mat.base_color = mat.base_color.with_alpha(0.3);
                            mat.alpha_mode = AlphaMode::Blend;
                        } else {
                            mat.base_color = mat.base_color.with_alpha(1.0);
                            mat.alpha_mode = AlphaMode::Opaque;
                        }
                    }
                }

                if pressed.0 && state.sliced {
                    // Create slicer with default config
                    let config = Config::default();
                    let mut slicer = Slicer::new(&config);
                    
                    // Slice the mesh
                    if slicer.slice(mesh, transform).is_ok() {
                        // Get slice mesh and material
                        let (slice_mesh, slice_material) = slicer.strip();
                        
                        // Spawn slice entity
                        let mesh_handle = meshes.add(slice_mesh);
                        let material_handle = materials.add(slice_material);
                        
                        let slice_entity = commands.spawn((
                            Mesh3d(mesh_handle),
                            MeshMaterial3d(material_handle),
                        )).id();
                        if !state.transparent {
                            commands.entity(entity).insert(Visibility::Hidden);
                        }
                        state.entities.push((entity, slice_entity));
                    }
                }
            }
        }

        if pressed.0 && !state.sliced {
             // If slice mode is disabled, remove slice entity
            for (entity, slice) in state.entities.iter() {
                commands.entity(*slice).despawn();
                commands.entity(*entity).insert(Visibility::Visible);
            }
            state.entities.clear();
        }
    }
}