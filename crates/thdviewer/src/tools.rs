use bevy::prelude::*;
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // Infinite Grid as ground plane
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            x_axis_color: Color::srgb(1.0, 0.0, 0.49),
            z_axis_color: Color::srgb(0.0, 0.49, 1.0),
            minor_line_color: Color::srgb(0.3, 0.3, 0.3),
            major_line_color: Color::srgb(0.25, 0.25, 0.25),
            fadeout_distance: 100.0,
            dot_fadeout_strength: 0.25,
            scale: 1.0,
        },
        Name::new("Infinite Grid"),
    ));
}