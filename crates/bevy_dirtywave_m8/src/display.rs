use bevy::{prelude::*, window::WindowResolution};

const TITLE: &'static str = "Bevy M8";

pub struct M8DisplayPlugin;
impl Plugin for M8DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                mode: bevy::window::WindowMode::BorderlessFullscreen(
                    MonitorSelection::Primary
                ),
                resolution: WindowResolution::new(320, 240),
                title: TITLE.into(),
                ..default()
            }),
            ..default()
        }));
    }
}
