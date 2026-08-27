use bevy::prelude::*;

use super::camera::CameraState;

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MeshPickingPlugin, TransformGizmoPlugin))
            .add_systems(Update, update)
            .add_observer(select);
    }
}

fn select(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    mut camerate: ResMut<CameraState>,
    existing: Query<Entity, With<TransformGizmoFocus>>,
) {
    if click.button == PointerButton::Primary {
        // Remove focus from all entities
        for e in &existing {
            commands.entity(e).remove::<TransformGizmoFocus>();
        }
        // Add focus to clicked entity
        commands.entity(click.entity).insert(TransformGizmoFocus);
        camerate.focused = Some(click.entity);
    }
}

fn update(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<TransformGizmoSettings>,
) {
    // Transform gizmo mode keys
    if keyboard.just_pressed(KeyCode::Digit1) {
        settings.mode = TransformGizmoMode::Translate;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        settings.mode = TransformGizmoMode::Rotate;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        settings.mode = TransformGizmoMode::Scale;
    }
    if keyboard.just_pressed(KeyCode::KeyT) {
        settings.space = match settings.space {
            TransformGizmoSpace::World => TransformGizmoSpace::Local,
            TransformGizmoSpace::Local => TransformGizmoSpace::World,
        };
    }
}