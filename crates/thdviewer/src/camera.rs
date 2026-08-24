use std::f32::consts;
use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, update_camera);
    }
}

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 5.0, 10.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::default(),
    ));
}

pub fn update_camera(
    windows: Query<&Window>,
    mut ev_motion: MessageReader<MouseMotion>,
    mut ev_scroll: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut OrbitCamera, &mut Transform, &Projection), Without<Mesh3d>>,
    mut models: Query<&mut Transform, (With<Mesh3d>, Without<OrbitCamera>)>,
) -> Result {
    let window = windows.single()?;
    let window_w = window.width();
    let window_h = window.height();

    let mut panmov = Vec2::ZERO;
    let mut rotate = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut modmov = Vec2::ZERO;

    let ctrled = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if mouse.pressed(MouseButton::Left) && !ctrled {
        for ev in ev_motion.read() {
            rotate += ev.delta;
        }
    } else if mouse.pressed(MouseButton::Right) {
        for ev in ev_motion.read() {
            panmov += ev.delta;
        }
    } else if mouse.pressed(MouseButton::Left) && ctrled {
        for ev in ev_motion.read() {
            modmov += ev.delta;
        }
    }
    
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    
    // Move models
    if modmov.length_squared() > 0.0 {
        // Get camera transform and projection to calculate movement
        let (_, camera_trans, projection): (&OrbitCamera, &Transform, &Projection) = camera.single()?;
        let (fov, aspect_ratio) = if let Projection::Perspective(proj) = projection {
            (proj.fov, proj.aspect_ratio)
        } else {
            (consts::PI / 4.0, window_w / window_h)
        };
        
        // Scale mouse movement by FOV and window size
        let scaled = modmov * Vec2::new(fov * aspect_ratio, fov) / Vec2::new(window_w, window_h);
        
        // Calculate camera's right and forward vectors projected to XZ plane
        let right = camera_trans.rotation * Vec3::X;
        let forward = camera_trans.rotation * Vec3::Z;
        
        // Project to XZ plane (remove Y component)
        let right_xz = Vec3::new(right.x, 0.0, right.z).normalize();
        let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize();
        
        for mut model_trans in models.iter_mut() {
            // Calculate distance from camera to model
            let distance = (model_trans.translation - camera_trans.translation).length();
            
            // Scale movement by distance (like panning)
            let movement = (right_xz * scaled.x + forward_xz * scaled.y) * distance;
            
            model_trans.translation += movement;
        }
    }
    
    for (mut camera, mut transform, projection) in camera.iter_mut() {
        if keyboard.just_pressed(KeyCode::KeyR) {
            *camera = OrbitCamera::default();
            transform.translation = Vec3::new(0.0, 5.0, 10.0);
            transform.rotation = Quat::from_rotation_x(-0.4636476).mul_quat(Quat::from_rotation_y(0.0));
            continue;
        }
        if rotate.length_squared() > 0.0 {
            let delta_x = rotate.x / window_w * consts::PI * 2.0;
            let delta_y = rotate.y / window_h * consts::PI;
            
            // Get the offset from camera to focus point
            let offset = transform.translation - camera.focus;
            
            // Create rotation quaternions
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            
            // Apply rotations around the focus point
            let rotation = yaw * pitch;
            let offset = rotation.mul_vec3(offset);
            
            // Update camera position and rotation
            transform.translation = camera.focus + offset;
            transform.rotation = rotation * transform.rotation;
        } else if panmov.length_squared() > 0.0 {
            // make panning distance independent of resolution and FOV,
            let window = windows.single()?;
            let window_w = window.width();
            let window_h = window.height();
            if let Projection::Perspective(projection) = projection {
                panmov *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / Vec2::new(window_w, window_h);
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -panmov.x;
            let above = transform.rotation * Vec3::Y * panmov.y;
            // make panning proportional to distance away from focus point
            let translation = (right + above) * camera.radius;
            camera.focus += translation;
            transform.translation += translation;
        } else if scroll.abs() > 0.0 {
            camera.radius -= scroll * camera.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            camera.radius = f32::max(camera.radius, 0.05);
            transform.translation = camera.focus + Mat3::from_quat(transform.rotation).mul_vec3(Vec3::new(0.0, 0.0, camera.radius));
        }
    }
    Ok(())
}