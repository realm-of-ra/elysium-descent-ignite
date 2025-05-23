use bevy::{picking::*, platform::collections::HashMap, prelude::*, sprite::Anchor};
use bevy_kira_audio::prelude::*;
use bevy_lunex::*;
// use std::time::Duration;
use vleue_kinetoscope::*;

use super::Screen;
use crate::game::resources::MainTrack;
use crate::systems::movie::*;
use crate::ui::styles::ElysiumDescentColorPalette;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    let priority_assets = PriorityAssets::default();
    app.insert_resource(priority_assets)
        .add_systems(PreStartup, preload)
        .add_plugins(AnimatedImagePlugin)
        .add_systems(OnEnter(Screen::Intro), IntroScene::spawn)
        .add_systems(OnExit(Screen::Intro), despawn_scene::<IntroScene>)
        .add_systems(OnEnter(Screen::MainMenu), MainMenuScene::spawn)
        .add_systems(OnExit(Screen::MainMenu), despawn_scene::<MainMenuScene>)
        .init_resource::<MainTrack>()
        .add_plugins(MoviePlugin);
}

// ===== SYSTEMS =====

fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// This system is run in PreStartup. It locks some assets from being freed when not used.
fn preload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut priority_assets: ResMut<PriorityAssets>,
) {
    // Load the WebP video with explicit type
    let video_handle: Handle<AnimatedImage> = asset_server.load("movies/intro.webp");

    // Insert it with the exact key that will be used later
    priority_assets
        .video
        .insert("intro".to_string(), video_handle.clone());

    // Debug output to check loading
    info!("Preloaded intro video: {:?}", video_handle);

    // Add the video handle to the asset lock too for extra safety
    commands.spawn(AssetLock {
        assets: vec![
            asset_server.load_folder("fonts").untyped(),
            asset_server.load_folder("images/ui").untyped(),
            video_handle.untyped(), // Include the WebP in the asset lock
        ],
    });

    // Audio assets
    commands.spawn(AssetLock {
        assets: vec![
            asset_server
                .load::<AudioSource>("audio/intro.ogg")
                .untyped(),
        ],
    });
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
pub struct AssetLock {
    #[allow(dead_code)]
    pub assets: Vec<UntypedHandle>,
}

/// Priority assets loaded before the game start
#[derive(Resource, Default)]
pub struct PriorityAssets {
    pub video: HashMap<String, Handle<AnimatedImage>>,
}

#[derive(Component)]
struct MainMenuScene;

#[derive(Component)]
struct IntroScene;

// Define the component
#[derive(Component)]
struct SkipIntroButton;

// ======= INTRO SCENE IMPLEMENTATION =======

impl IntroScene {
    fn spawn(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        priority_assets: Res<PriorityAssets>,
    ) {
        // Debug output
        info!(
            "Available priority video assets: {:?}",
            priority_assets.video.keys().collect::<Vec<_>>()
        );

        // Get the video handle, with fallback
        let video_handle = priority_assets
            .video
            .get("intro")
            .cloned()
            .unwrap_or_else(|| {
                info!("Fallback: Loading intro video directly");
                asset_server.load("movies/intro.webp")
            });

        // Create UI
        commands
            .spawn((UiLayoutRoot::new_2d(), UiFetchFromCamera::<0>, IntroScene))
            .with_children(|ui| {
                ui.spawn((
                    UiLayout::solid()
                        .size((1920.0, 1080.0))
                        .scaling(Scaling::Fill)
                        .pack(),
                    Movie::play(video_handle, asset_server.load("audio/intro.ogg"))
                        .playback(MoviePlayback::Stop),
                ))
                .observe(
                    |_event: Trigger<MovieEnded>, mut next: ResMut<NextState<Screen>>| {
                        next.set(Screen::MainMenu)
                    },
                );

                // Skip button
                ui.spawn((
                    Name::new("Skip Button"),
                    UiLayout::window()
                        .pos(Rl((95.0, 95.0)))
                        .anchor(Anchor::BottomRight)
                        .size(Rl((20.0, 6.0)))
                        .pack(),
                    OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
                ))
                .with_children(|button| {
                    // Image/background element
                    button
                        .spawn((
                            // Define layouts for different states like in your other buttons
                            UiLayout::new(vec![
                                (UiBase::id(), UiLayout::window().full()),
                                (UiHover::id(), UiLayout::window().x(Rl(1.0)).full()),
                            ]),
                            UiHover::new().forward_speed(20.0).backward_speed(4.0),
                            UiColor::new(vec![
                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.4)),
                                (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2)),
                            ]),
                            Sprite {
                                image: asset_server
                                    .load("images/ui/components/button_symetric_sliced.png"),
                                image_mode: SpriteImageMode::Sliced(TextureSlicer {
                                    border: BorderRect::all(32.0),
                                    ..default()
                                }),
                                ..default()
                            },
                            // Don't add Pickable::IGNORE here - we want this clickable
                        ))
                        .with_children(|ui| {
                            // Text
                            ui.spawn((
                                UiLayout::window()
                                    .pos(Rl(50.0))
                                    .anchor(Anchor::Center)
                                    .pack(),
                                UiColor::new(vec![
                                    (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                    (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2)),
                                ]),
                                UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                UiTextSize::from(Rh(50.0)),
                                Text2d::new("SKIP"),
                                TextFont {
                                    font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                                    font_size: 64.0,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                        });
                })
                // Add the hover effects and click - match your button pattern exactly
                .observe(hover_set::<Pointer<Over>, true>)
                .observe(hover_set::<Pointer<Out>, false>)
                .observe(
                    |_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                        info!("Skip button clicked!");
                        next.set(Screen::MainMenu);
                    },
                );
            });
    }
}

// ===== MAIN MENU IMPLEMENTATION =====

impl MainMenuScene {
    fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Create UI
        commands
            .spawn((
                UiLayoutRoot::new_2d(),
                UiFetchFromCamera::<0>,
                MainMenuScene,
            ))
            .with_children(|ui| {
                // Spawn the background
                ui.spawn((
                    // You can name your entites for easier debug
                    Name::new("Background"),
                    UiLayout::solid()
                        .size((1920.0, 1080.0))
                        .scaling(Scaling::Fill)
                        .pack(),
                    Sprite::from_image(asset_server.load("images/ui/background.png")),
                ));

                // Add the panel boundary
                ui.spawn((UiLayout::solid()
                    .size((881.0, 1600.0))
                    .align_x(-0.74)
                    .pack(),))
                    .with_children(|ui| {
                        // Spawn the panel
                        ui.spawn((
                            Name::new("Panel"),
                            UiLayout::window()
                                .x(Rl(50.0))
                                .anchor(Anchor::TopCenter)
                                .size(Rl(105.0))
                                .pack(),
                            Sprite::from_image(asset_server.load("images/ui/panel_menu.png")),
                        ));
                        // Spawn the logo boundary
                        ui.spawn((UiLayout::window()
                            .y(Rl(11.0))
                            .size(Rl((105.0, 20.0)))
                            .pack(),))
                            .with_children(|ui| {
                                // Spawn the logo
                                ui.spawn((
                                    Name::new("Logo"),
                                    UiLayout::solid().size((1240.0, 381.0)).pack(),
                                    Sprite::from_image(asset_server.load("images/ui/title.png")),
                                ));
                            });

                        ui.spawn((
                            UiLayout::window().pos(Rl((22.0, 33.0))).size(Rl((55.0, 34.0))).pack(),
                        )).with_children(|ui| {

                            // Spawn buttons
                            let gap = 3.0;
                            let size = 14.0;
                            let mut offset = 0.0;
                            for button in ["Continue", "New Game", "Load Game", "Settings", "Credits", "Quit Game"] {

                                // Spawn the button
                                let mut button_entity = ui.spawn((
                                    Name::new(button),
                                    UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack(),
                                    OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
                                ));
                                button_entity.with_children(|ui| {
                                    // Spawn the image
                                    ui.spawn((
                                        // You can define layouts for multiple states
                                        UiLayout::new(vec![
                                            (UiBase::id(), UiLayout::window().full()),
                                            (UiHover::id(), UiLayout::window().x(Rl(10.0)).full())
                                        ]),
                                        // Like this you can enable a state
                                        UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                        // You can specify colors for multiple states
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.15)),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                        ]),
                                        Sprite {
                                            image: asset_server.load("images/ui/components/button_symetric_sliced.png"),
                                            // Here we enable sprite slicing
                                            image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(32.0), ..default() }),
                                            ..default()
                                        },
                                        // Make sure it does not cover the bounding zone of parent
                                        Pickable::IGNORE,
                                    )).with_children(|ui| {

                                        // Spawn the text
                                        ui.spawn((
                                            // For text always use window layout to position it
                                            UiLayout::window().pos((Rh(40.0), Rl(50.0))).anchor(Anchor::CenterLeft).pack(),
                                            UiColor::new(vec![
                                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                                (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                            ]),
                                            UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                            // You can control the size of the text
                                            UiTextSize::from(Rh(60.0)),
                                            // You can attach text like this
                                            Text2d::new(button),
                                            TextFont {
                                                font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                                                font_size: 64.0,
                                                ..default()
                                            },
                                            // Make sure it does not cover the bounding zone of parent
                                            Pickable::IGNORE,
                                        ));

                                        // Spawn the fluff
                                        ui.spawn((
                                            // For text always use window layout to position it
                                            UiLayout::window().pos(Rl((90.0, 50.0))).anchor(Anchor::CenterRight).pack(),
                                            UiColor::new(vec![
                                                (UiBase::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(0.2)),
                                                (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                            ]),
                                            UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                            // You can control the size of the text
                                            UiTextSize::from(Rh(60.0)),
                                            // You can attach text like this
                                            Text2d::new("<-"),
                                            TextFont {
                                                font: asset_server.load("fonts/rajdhani/Rajdhani-Bold.ttf"),
                                                font_size: 64.0,
                                                ..default()
                                            },
                                        ));
                                    });

                                // Enable the transition on hover
                                }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                                // Assign a functionality to the buttons
                                match button {
                                    "New Game" => {
                                        button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                            // Change the state to settings
                                            next.set(Screen::NewGame);
                                        });
                                    },
                                    "Settings" => {
                                        button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                            // Change the state to settings
                                            next.set(Screen::Settings);
                                        });
                                    },
                                    "Continue" => {
                                        button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                            // Change the state to settings
                                            next.set(Screen::GamePlay);
                                        });
                                    },
                                    "Quit Game" => {
                                        button_entity.observe(|_: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>| {
                                            // Close the app
                                            exit.write(AppExit::Success);
                                        });
                                    },
                                    _ => {
                                        button_entity.observe(|c_trigger: Trigger<Pointer<Click>>, c_button: Query<NameOrEntity, With<UiLayout>>| {
                                          info!("Clicked: {}", c_button.get(c_trigger.target()).unwrap());
                                        });
                                    }
                                }

                                offset += gap + size;
                            }
                        });
                    });
            });
    }
}
