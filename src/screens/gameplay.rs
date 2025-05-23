use avian3d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

use super::Screen;
use crate::game::resources::MainTrack;
use crate::rendering::cameras::player_camera::*;
use crate::Player;
use crate::systems::character_controller::CharacterControllerBundle;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GamePlay), PlayingScene::spawn_environment)
        .add_systems(Update, PlayingScene::check_environment_loaded)
        .add_systems(OnExit(Screen::GamePlay), despawn_scene::<PlayingScene>)
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(ClearColor(Color::srgba_u8(135, 206, 250, 191)))
        .init_resource::<MainTrack>()
        .init_resource::<EnvironmentLoadTimer>();
}

// ===== SYSTEMS =====

fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
struct PlayingScene;

#[derive(Component)]
struct EnvironmentMarker;

#[derive(Resource)]
struct EnvironmentLoadTimer {
    timer: Timer,
}

impl Default for EnvironmentLoadTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
        }
    }
}

// ===== PLAYING SCENE IMPLEMENTATION =====

impl PlayingScene {
    fn spawn_environment(mut commands: Commands, assets: Res<AssetServer>) {
        commands.insert_resource(AmbientLight {
            color: Color::srgb_u8(68, 71, 88),
            brightness: 120.0,
            ..default()
        });

        // Environment (see the `collider_constructors` example for creating colliders from scenes)
        let scene_handle = assets.load("models/fantasy_environment_level_set.glb#Scene0");
        commands.spawn((
            Name::new("Environment"),
            EnvironmentMarker,
            SceneRoot(scene_handle),
            Transform {
                translation: Vec3::new(0.0, -1.5, 0.0),
                rotation: Quat::from_rotation_y(-core::f32::consts::PI * 0.5),
                scale: Vec3::splat(0.05), // ⬅️ scale environment down
            },
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
            DebugRender::default(),
        ));

        // Light
        commands.spawn((
            DirectionalLight {
                illuminance: 80_000.0, // bright midday sun
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_3,
                std::f32::consts::FRAC_PI_4,
                0.0,
            )),
        ));
    }

    fn check_environment_loaded(
        time: Res<Time>,
        mut timer: ResMut<EnvironmentLoadTimer>,
        commands: Commands,
        assets: Res<AssetServer>,
        environment_query: Query<&EnvironmentMarker>,
        collider_query: Query<&Collider>,
    ) {
        timer.timer.tick(time.delta());
        
        // Check if environment exists and has colliders
        if timer.timer.just_finished() {
            if let Ok(_) = environment_query.single() {
                // Check if any colliders have been generated
                if !collider_query.is_empty() {
                    info!("Environment and colliders loaded, spawning player");
                    Self::spawn_player(commands, assets);
                } else {
                    info!("Waiting for colliders to be generated...");
                    // Reset timer if colliders aren't ready
                    timer.timer.reset();
                }
            } else {
                info!("Waiting for environment to be loaded...");
                // Reset timer if environment isn't loaded
                timer.timer.reset();
            }
        }
    }

    fn spawn_player(mut commands: Commands, assets: Res<AssetServer>) {
        // Player
        commands.spawn((
            Name::new("Survivor Character"),
            Player,
            CharacterControllerBundle::new(Collider::capsule(0.5, 0.8)).with_movement(
                112.5,  // acceleration
                0.9,    // damping
                16.0,   // jump impulse (increased further)
            ),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(2.0),
            SceneRoot(assets.load("models/person.glb#Scene0")),
            Transform {
                translation: Vec3::new(10.0, 5.0, -60.0),
                scale: Vec3::splat(5.0),
                ..default()
            },
        ));

        // Camera
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform::from_xyz(0.0, 5.0, -8.0).looking_at(Vec3::new(0.0, 3.5, 0.0), Vec3::Y),
            FlyCam::default(),
        ));
    }
}
