use std::f32::consts::{PI, FRAC_PI_2};
use bevy::prelude::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

#[derive(Resource, Default)]
pub struct CameraState {
    pub focused: Option<Entity>,
}

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            focus: Vec3::ZERO,
            radius: 15.0,
            pitch: -0.8,
            yaw: 0.8,
        }
    }
}

fn setup(mut commands: Commands) {
    let camera = OrbitCamera::default();
    let transform = orbitrans(&camera).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        transform,
        camera,
        TransformGizmoCamera,
        Name::new("Main Camera"),
    ));
}

fn orbitrans(camera: &OrbitCamera) -> Transform {
    let yawr = Quat::from_rotation_y(camera.yaw);
    let pitchr = Quat::from_rotation_x(camera.pitch);
    let rotation = yawr * pitchr;
    let position = camera.focus + rotation * Vec3::new(0.0, 0.0, camera.radius);
    Transform::from_translation(position).with_rotation(rotation)
}

pub fn update(
    windows: Query<&Window>,
    mumotion: Res<AccumulatedMouseMotion>,
    muscroll: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camerate: Res<CameraState>,
    gizstate: Res<TransformGizmoState>,
    mut camera: Query<(&mut OrbitCamera, &mut Transform, &Projection)>,
    focusq: Query<&GlobalTransform>,
) -> Result {
    let window = windows.single()?;
    let window_w = window.width();
    let window_h = window.height();

    let ctrled = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shifted = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let (mut orbit, mut transform, projection) = camera.single_mut()?;

    if keyboard.just_pressed(KeyCode::KeyF) {
        // F key: focus on selected entity
        if let Some(entity) = camerate.focused {
            if let Ok(gtrans) = focusq.get(entity) {
                orbit.focus = gtrans.translation();
                orbit.radius = 5.0;
                *transform = orbitrans(&orbit);
            }
        }
    } else if keyboard.just_pressed(KeyCode::KeyR) {
        // R key: reset camera
        *orbit = OrbitCamera::default();
        *transform = orbitrans(&orbit);
    } else {
        let delta = mumotion.delta;
        let scroll = muscroll.delta.y;
    
        // Orbit: left click drag (skip when gizmo is active)
        if mouse.pressed(MouseButton::Left) && !ctrled && !shifted && !gizstate.active {
            if delta.length_squared() > 0.0 {
                let sensitivity = 0.005;
                orbit.yaw -= delta.x * sensitivity;
                orbit.pitch -= delta.y * sensitivity;
                let plimit = FRAC_PI_2 - 0.01;
                orbit.pitch = orbit.pitch.clamp(-plimit, plimit);
            }
        }
    
        // Pan: middle click drag or right click drag
        if mouse.pressed(MouseButton::Middle) || (mouse.pressed(MouseButton::Right) && !ctrled) {
            if delta.length_squared() > 0.0 {
                let (fov, ratio) = if let Projection::Perspective(proj) = projection {
                    (proj.fov, proj.aspect_ratio)
                } else {
                    (PI / 4.0, window_w / window_h)
                };
                let scaled = delta * 1.0 * Vec2::new(fov * ratio, fov) / Vec2::new(window_w, window_h);
                let right = transform.rotation * Vec3::X;
                let up = transform.rotation * Vec3::Y;
                let translation = (-right * scaled.x + up * scaled.y) * orbit.radius;
                orbit.focus += translation;
            }
        }
    
        // Zoom: scroll wheel
        if scroll.abs() > 0.0 {
            let factor = 1.0 - scroll * 0.1;
            orbit.radius *= factor;
            orbit.radius = orbit.radius.clamp(0.1, 1000.0);
        }
    
        // Dolly zoom: ctrl + right click drag
        if mouse.pressed(MouseButton::Right) && ctrled {
            if delta.length_squared() > 0.0 {
                orbit.radius -= delta.y * orbit.radius * 0.005;
                orbit.radius = orbit.radius.clamp(0.1, 1000.0);
            }
        }
    
        // Roll: shift + left click drag (skip when gizmo is active)
        if mouse.pressed(MouseButton::Left) && shifted && !gizstate.active {
            if delta.length_squared() > 0.0 {
                let (_, _, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                let yawr = Quat::from_rotation_y(orbit.yaw);
                let pitchr = Quat::from_rotation_x(orbit.pitch);
                let rollr = Quat::from_rotation_z(roll - delta.x * 0.005);
                transform.rotation = yawr * pitchr * rollr;
            }
        } else {
            // Apply orbit state to transform
            let yawr = Quat::from_rotation_y(orbit.yaw);
            let pitchr = Quat::from_rotation_x(orbit.pitch);
            transform.rotation = yawr * pitchr;
            transform.translation = orbit.focus + transform.rotation * Vec3::new(0.0, 0.0, orbit.radius);
        }
    }

    Ok(())
}