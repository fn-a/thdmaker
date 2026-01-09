use bevy::prelude::*;

pub struct UIReactPlugin;

impl Plugin for UIReactPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_uireact);
    }
}

fn setup_uireact(
    mut commands: Commands,
) {
    // UI Text
    commands.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
    }).with_child((
        Text::new("Left Click: Rotate, Right Click: Pan, Scroll: Zoom,\nCtrl+Left Click: Move Model, R: Reset Camera"),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
        TextColor(Color::WHITE.into())
    ));
}