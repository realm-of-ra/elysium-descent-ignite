use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowResolution};
use elysium_descent_ignite::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Elysium Descent".into(),
                resolution: WindowResolution::new(1920.0, 1080.0).with_scale_factor_override(1.0),
                resizable: true,
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                mode: WindowMode::Windowed,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
