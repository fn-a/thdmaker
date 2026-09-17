use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
struct InstructionsText;

fn setup(
    mut commands: Commands,
) {
    // UI Text
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Text::new(String::new()),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..Default::default()
        },
        TextColor(Color::WHITE),
        InstructionsText,
    ));
}

fn update(
    settings: Option<Res<TransformGizmoSettings>>,
    mut text: Single<&mut Text, With<InstructionsText>>,
) {
    let mode = settings.as_ref().map_or("N/A", |s| match s.mode {
        TransformGizmoMode::Translate => "Translate",
        TransformGizmoMode::Rotate => "Rotate",
        TransformGizmoMode::Scale => "Scale",
    });
    let space = settings.as_ref().map_or("N/A", |s| match s.space {
        TransformGizmoSpace::World => "World",
        TransformGizmoSpace::Local => "Local",
    });

    let txts = [
        "Left Click: Select Object",
        "Left Click Empty Space: Clear Selection",
        "Middle Click: Pan",
        "Right Click: Pan",
        "Scroll: Zoom",
        "Shift+Left Drag: Roll View",
        "Ctrl+Right Drag: Dolly Zoom",
        "F: Focus on Selected",
        "R: Reset Camera",
        "1: Translate | 2: Rotate | 3: Scale",
        "T: Toggle World/Local Space",
        "X: Toggle Wireframe",
        "S: Toggle Slicer Mode",
        "V: Toggle Transparent",
    ];

    text.0 = format!("{}\nGizmo: {mode} | Space: {space}", txts.join("\n"));
}