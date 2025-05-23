use bevy::{prelude::*, render::view::RenderLayers};
use bevy_enhanced_input::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;

mod game;
mod screens;
mod starknet;
mod systems;

pub mod rendering;
pub mod ui;

pub use game::components::*;
pub use game::resources::MainTrack;
pub use starknet::NetworkingPlugin;
pub use systems::input::ElysiumInputPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_plugins(EnhancedInputPlugin)
            .add_plugins(AudioPlugin)
            .add_plugins(ElysiumInputPlugin)
            .add_plugins(NetworkingPlugin)
            .add_plugins(UiLunexPlugins)
            .add_plugins(screens::plugin)
            .add_plugins(systems::plugin);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 2,
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
        UiSourceCamera::<0>,
        Transform::from_translation(Vec3::Z * 1000.0),
    ));
}
