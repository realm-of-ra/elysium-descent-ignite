use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};
use bevy_lunex::*;

use super::Screen;
use crate::game::resources::MainTrack;
use crate::ui::styles::ElysiumDescentColorPalette;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MainTrack>();

    app.add_systems(
        OnEnter(Screen::GeneratingLevel),
        GeneratingLevelScene::spawn,
    )
    .add_systems(
        OnExit(Screen::GeneratingLevel),
        despawn_scene::<GeneratingLevelScene>,
    );
}

// ===== SYSTEMS =====

fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
struct GeneratingLevelScene;

// ===== GENERATING LEVEL IMPLEMENTATION =====

impl GeneratingLevelScene {
    fn spawn(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
    }
}
