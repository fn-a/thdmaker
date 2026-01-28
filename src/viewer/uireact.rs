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
    let texts = [
        "Left Click: Rotate",
        "Right Click: Pan",
        "Scroll: Zoom",
        "Ctrl+Left Click: Move Model",
        "R: Reset Camera",
        "X: Toggle Wireframe",
        "S: Toggle Slicer Mode",
        "V: Toggle Transparent",
    ];
    // UI Text
    commands.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
    }).with_child((
        Text::new(texts.join("\n")),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
        TextColor(Color::WHITE.into())
    ));
}