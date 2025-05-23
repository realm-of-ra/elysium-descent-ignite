use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};
use bevy_lunex::*;

use super::Screen;
use crate::game::resources::MainTrack;
use crate::ui::styles::ElysiumDescentColorPalette;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), SettingsScene::spawn)
        .add_systems(OnExit(Screen::Settings), despawn_scene::<SettingsScene>)
        .init_resource::<MainTrack>();
}

// ===== SYSTEMS =====

fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
struct SettingsScene;

// ===== SETTINGS SCENE IMPLEMENTATION =====

impl SettingsScene {
    fn spawn(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
    ) {
        // Create the transparent render texture
        let image_handle = images.add(Image::clear_render_texture());

        // Create embedd camera that will render to the texture
        let texture_camera = commands
            .spawn((
                Camera2d,
                Camera::clear_render_to(image_handle.clone()).with_order(-1),
                // This filters out all the normal entities
                RenderLayers::layer(1),
                // A scene marker for later mass scene despawn, not UI related
                SettingsScene,
            ))
            .id();

        // Create UI
        commands.spawn((
            UiLayoutRoot::new_2d(),
            // Make the UI synchronized with camera viewport size
            UiFetchFromCamera::<0>,
            // A scene marker for later mass scene despawn, not UI related
            SettingsScene
        )).with_children(|ui| {

            // Spawn the background
            ui.spawn((
                Name::new("Background"),
                UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack(),
                Sprite::from_image(asset_server.load("images/ui/background.png")),
            ));

            // Spawn the settings content
            ui.spawn((
                UiLayout::solid().size((3.0, 3.0)).align_y(-1.0).pack(),
            )).with_children(|ui| {

                // Spawn the tab bar
                ui.spawn((
                    UiLayout::window().size(Rl((100.0, 8.0))).pack(),
                )).with_children(|ui| {

                    // Spawn left chevron
                    ui.spawn((
                        Name::new("Chevron Left"),
                        UiLayout::window().pos(Rl((5.0, 50.0))).anchor(Anchor::Center).size(Rh(35.0)).pack(),
                        Sprite::from_image(asset_server.load("images/ui/components/chevron_left.png")),
                        UiHover::new().instant(true),
                        UiColor::new(vec![
                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                            (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                        ]),
                    )).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                    // Spawn right chevron
                    ui.spawn((
                        Name::new("Chevron Right"),
                        UiLayout::window().pos(Rl((95.0, 50.0))).anchor(Anchor::Center).size(Rh(35.0)).pack(),
                        Sprite::from_image(asset_server.load("images/ui/components/chevron_right.png")),
                        UiHover::new().instant(true),
                        UiColor::new(vec![
                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                            (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                        ]),
                    )).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                    // Spawn the control bar
                    ui.spawn((
                        UiLayout::window().x(Rl(10.0)).size(Rl((80.0, 100.0))).pack(),
                    )).with_children(|ui| {

                        let categories = ["Controls", "Sound", "Graphics", "Window"];
                        let pos = 100.0 / categories.len() as f32;
                        for (i, category) in categories.into_iter().enumerate() {

                            // Spawn the button
                            ui.spawn((
                                Name::new(category),
                                UiLayout::window().x(Rl(pos * i as f32)).size(Rl((pos, 100.0))).pack(),
                            )).with_children(|ui| {

                                // Spawn the background
                                ui.spawn((
                                    UiLayout::window().full().y(Rl(10.0)).height(Rl(80.0)).pack(),
                                    UiHover::new().forward_speed(20.0).backward_speed(5.0),
                                    UiColor::new(vec![
                                        (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.0)),
                                        (UiHover::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.4))
                                    ]),
                                    Sprite {
                                        image: asset_server.load("images/ui/components/button_symetric_sliced.png"),
                                        image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(32.0), ..default() }),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                )).with_children(|ui| {

                                    // Spawn the text
                                    ui.spawn((
                                        UiLayout::window().pos(Rl(50.0)).anchor(Anchor::Center).pack(),
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                                        ]),
                                        UiHover::new().instant(true),
                                        UiTextSize::from(Rh(50.0)),
                                        Text2d::new(category.to_ascii_uppercase()),
                                        TextFont {
                                            font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                                            font_size: 64.0,
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ));
                                });

                            // Add the observers
                            }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);
                        }

                    });

                });

                // Spawn the Bevy UI embedd
                ui.spawn((
                    UiLayout::boundary().y1(Rl(10.0)).pos2(Rl(100.0)).pack(),
                    Sprite::from_image(image_handle),
                    UiEmbedding,
                ));

            });
        });

        // The Bevy UI nodes must be here to work
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                // Render this ui to our new camera
                UiTargetCamera(texture_camera),
                // A scene marker for later mass scene despawn, not UI related
                SettingsScene,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Dummy text. Controls goes here."),
                    TextFont {
                        font_size: 64.0,
                        font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                        ..default()
                    },
                    TextColor::WHITE,
                ));
            });
    }
}
