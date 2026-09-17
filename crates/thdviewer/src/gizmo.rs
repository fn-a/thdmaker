use bevy::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::gizmos::aabb::ShowAabbGizmo;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::picking::pointer::{PointerAction, PointerInput};

use super::slice::SliceModel;

/// Click a model to select it; the handles sit in the middle of it.
pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MeshPickingPlugin, TransformGizmoPlugin))
            .init_resource::<Press>()
            .add_systems(Update, (update, vacancy, despawned))
            .add_observer(select)
            .add_observer(deselect);
    }
}

/// Wraps a selected part, so the handles sit at its centre instead of its origin.
#[derive(Component)]
struct Pivot {
    model: Entity,
    parent: Option<Entity>,
}

/// Left button state, for telling a drag from a click on empty space.
#[derive(Resource, Default)]
struct Press {
    down: bool,
    travelled: f32,
    pick: bool,
}

/// Ask for the selected part to be handed back.
#[derive(Event)]
struct Erase;

/// Click a model: take the handles, wrapped around the centre of its bounds.
fn select(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    mut press: ResMut<Press>,
    landed: Query<Has<Node>, Or<(With<Node>, With<SliceModel>)>>,
    parents: Query<&ChildOf>,
    focus: Query<(Entity, &Transform, Option<&Pivot>), With<TransformGizmoFocus>>,
    parts: Query<(&Transform, &GlobalTransform, Option<&Aabb>)>,
) {
    if click.button == PointerButton::Primary {
        // A hit marks the press, never unmarks it: a click bubbles to ancestors we do
        // not know, and only the press itself resets the flag.
        if landed.contains(click.entity) {
            press.pick = true;
        }

        // Known and not the overlay: a part.
        if landed.get(click.entity).is_ok_and(|ov| !ov) {
            // A click bubbles through a scene, so take the topmost part of the chain.
            let target = parents
                .iter_ancestors(click.entity)
                .take_while(|e| landed.get(*e).is_ok_and(|ov| !ov))
                .last()
                .unwrap_or(click.entity);

            // Hand the previous part back first: only one entity may carry the handles.
            if focus.iter().next()
                .map(|(carrier, _, pivot)| {
                    pivot.map_or(carrier, |pivot| pivot.model) != target
                }).unwrap_or(true)
            {
                commands.trigger(Erase);

                let color = Some(Color::srgb(1.0, 0.82, 0.16));
                if let Ok((local, global, bounds)) = parts.get(target) {
                    if let Some(bounds) = bounds {
                        // Centre of the bounds, in the parent's space: the wrapper is a
                        // sibling, so the part only slides over by the difference.
                        let centre = *global * Vec3::from(bounds.center);
                        let parent = parents
                            .get(target)
                            .ok()
                            .map(|child_of| child_of.parent());

                        let pivat = match parent.and_then(|parent| parts.get(parent).ok()) {
                            Some((_, global, _)) => global.to_matrix().inverse().transform_point3(centre),
                            None => centre,
                        };
                        // The wrapper carries the part's turn, for local-space mode.
                        let pivot = Transform {
                            translation: pivat,
                            rotation: local.rotation,
                            scale: Vec3::ONE,
                        };
                        let wrapped = Transform::from_matrix(pivot.to_matrix().inverse() * local.to_matrix());

                        let wrapper = commands.spawn((pivot, TransformGizmoFocus, Pivot { model: target, parent })).id();

                        if let Some(parent) = parent {
                            commands.entity(wrapper).insert(ChildOf(parent));
                        }

                        commands.entity(target).insert((wrapped, ShowAabbGizmo { color }, ChildOf(wrapper)));
                    } else {
                        // No bounds to find a middle with: the part takes the handles.
                        commands.entity(target).insert((TransformGizmoFocus, ShowAabbGizmo { color }));
                    }
                }
            }
        }
    }
}

/// Give the part back its own place and drop the wrapper.
fn deselect(
    _: On<Erase>,
    mut commands: Commands,
    focus: Query<(Entity, &Transform, Option<&Pivot>), With<TransformGizmoFocus>>,
    parts: Query<(&Transform, &GlobalTransform, Option<&Aabb>)>,
) {
    for (carrier, trans, wrapper) in &focus {
        if let Some(wrapper) = wrapper {
            // Fold in whatever the handles did, so the part stays where it was left.
            if let Ok((local, _, _)) = parts.get(wrapper.model) {
                commands.entity(wrapper.model).insert(*trans * *local);
            }
            match wrapper.parent {
                Some(parent) => {
                    commands.entity(wrapper.model).insert(ChildOf(parent));
                }
                None => {
                    commands.entity(wrapper.model).remove::<ChildOf>();
                }
            }
            commands.entity(wrapper.model).remove::<ShowAabbGizmo>();
            commands.entity(carrier).despawn();
        } else {
            // Focused without a wrapper.
            commands.entity(carrier).remove::<(TransformGizmoFocus, ShowAabbGizmo)>();
        }
    }
}

/// Follow the left button, to catch a click into empty space.
fn vacancy(
    mut commands: Commands,
    motion: Res<AccumulatedMouseMotion>,
    mut input: MessageReader<PointerInput>,
    mut press: ResMut<Press>,
) {
    // Counted every frame; the press resets it.
    press.travelled += motion.delta.length();
    for event in input.read() {
        match event.action {
            PointerAction::Press(PointerButton::Primary) => {
                *press = Press {
                    down: true,
                    ..default()
                };
            }
            PointerAction::Release(PointerButton::Primary) if press.down => {
                press.down = false;
                // A press that travelled is a drag; one that hit nothing clears.
                if !press.pick && press.travelled <= 4.0 {
                    commands.trigger(Erase);
                }
            }
            _ => {}
        }
    }
}

/// Drop wrappers whose part is gone.
fn despawned(
    mut commands: Commands,
    alive: Query<()>,
    pivots: Query<(Entity, &Pivot)>
) {
    for (pivot, wrapper) in &pivots {
        if alive.get(wrapper.model).is_err() {
            commands.entity(pivot).despawn();
        }
    }
}

fn update(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<TransformGizmoSettings>,
) {
    // Transform gizmo mode keys
    for key in keyboard.get_just_pressed() {
        match key {
            KeyCode::Digit1 => settings.mode = TransformGizmoMode::Translate,
            KeyCode::Digit2 => settings.mode = TransformGizmoMode::Rotate,
            KeyCode::Digit3 => settings.mode = TransformGizmoMode::Scale,
            KeyCode::KeyT => {
                settings.space = match settings.space {
                    TransformGizmoSpace::World => TransformGizmoSpace::Local,
                    TransformGizmoSpace::Local => TransformGizmoSpace::World,
                }
            }
            _ => {}
        }
    }
}