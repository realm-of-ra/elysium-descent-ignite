use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};
use bevy_lunex::*;

use super::Screen;
use crate::game::resources::MainTrack;
use crate::rendering::cameras::showcase::{ShowcaseCamera, ShowcaseCameraPlugin};
use crate::ui::styles::ElysiumDescentColorPalette;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::NewGame), NewGameScene::spawn)
        .add_systems(OnExit(Screen::NewGame), despawn_scene::<NewGameScene>)
        .init_resource::<MainTrack>()
        .add_plugins(ShowcaseCameraPlugin);
}

// ===== SYSTEMS =====

fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
struct NewGameScene;

// ===== NEW GAME IMPLEMENTATION =====

impl NewGameScene {
    fn spawn(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
        // mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Create the transparent render texture
        let image_handle = images.add(Image::clear_render_texture());

        // Spawn the 3D camera that will render to the texture
        commands.spawn((
            ShowcaseCamera::default(),
            Camera3d::default(),
            Camera::clear_render_to(image_handle.clone()).with_order(-1),
            // Set the render layers to be Default + 3D UI Debug for gizmos
            RenderLayers::from_layers(&[0, 2]),
            // A scene marker for later mass scene despawn, not UI related
            NewGameScene,
        ));

        // Spawn the model
        commands.spawn((
            SceneRoot(asset_server.load("models/person.glb#Scene0")),
            Transform::from_xyz(-0.3, -0.3, -1.0).with_scale(Vec3::new(0.8, 0.8, 0.8)),
            // A scene marker for later mass scene despawn, not UI related
            NewGameScene,
        ));

        // Spawn point light
        commands.spawn((
            PointLight {
                intensity: 10000.0,
                shadows_enabled: false,
                color: Color::ELYSIUM_DESCENT_RED.with_luminance(1.6),
                ..default()
            },
            // A scene marker for later mass scene despawn, not UI related
            NewGameScene,
        ));

        // Create UI
        commands.spawn((
            UiLayoutRoot::new_2d(),
            // Make the UI synchronized with camera viewport size
            UiFetchFromCamera::<0>,
            // A scene marker for later mass scene despawn, not UI related
            NewGameScene
        )).with_children(|ui| {

            // Spawn the background
            ui.spawn((
                Name::new("Background"),
                UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack(),
                Sprite::from_image(asset_server.load("images/ui/background.png")),
                UiDepth::Set(0.0),
            ));

            // Spawn the camera plane
            ui.spawn((
                Name::new("Camera"),
                UiLayout::window().full().pack(),
                Sprite::from_image(image_handle),
                UiEmbedding,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
            ));

            // Spawn return button
            ui.spawn((
                Name::new("Return"),
                UiLayout::window().pos(Rl((2.0, 4.0))).size(Rl((16.0, 8.0))).pack(),
                OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
            )).with_children(|ui| {
                // Spawn the image
                ui.spawn((
                    // You can define layouts for multiple states
                    UiLayout::new(vec![
                        (UiBase::id(), UiLayout::boundary().pos2(Rl(100.0)).wrap()),
                        (UiHover::id(), UiLayout::boundary().pos2(Rl(100.0)).x2(Rl(115.0)).wrap())
                    ]),
                    // Like this you can enable a state
                    UiHover::new().forward_speed(20.0).backward_speed(4.0),
                    // You can specify colors for multiple states
                    UiColor::new(vec![
                        (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.15)),
                        (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                    ]),
                    Sprite {
                        image: asset_server.load("images/ui/components/button_sliced_bottom_right.png"),
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
                        Text2d::new("Return"),
                        TextFont {
                            font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                            font_size: 64.0,
                            ..default()
                        },
                        // Make sure it does not cover the bounding zone of parent
                        Pickable::IGNORE,
                    ));
                });

            // Enable the transition on hover
            }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>)
            .observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| next.set(Screen::MainMenu) );

            // Spawn panel boundary
            ui.spawn((
                UiLayout::solid().size((879.0, 1600.0)).align_x(0.82).pack(),
            )).with_children(|ui| {

                ui.spawn((
                    UiLayout::window().x(Rl(50.0)).anchor(Anchor::TopCenter).size(Rl(100.0)).pack(),
                    Sprite::from(asset_server.load("images/ui/panel_full.png"))
                )).with_children(|ui| {

                    // Spawn the text
                    ui.spawn((
                        // For text always use window layout to position it
                        UiLayout::window().pos(Rl((53., 8.))).anchor(Anchor::TopCenter).pack(),
                        UiColor::from(Color::ELYSIUM_DESCENT_RED),
                        // You can control the size of the text
                        UiTextSize::from(Rh(5.0)),
                        // You can attach text like this
                        Text2d::new("New Character"),
                        TextFont {
                            font: asset_server.load("fonts/rajdhani/Rajdhani-SemiBold.ttf"),
                            font_size: 64.0,
                            ..default()
                        },
                    ));

                    // Spawn button boundary
                    ui.spawn((
                        Name::new("Button List"),
                        UiLayout::window().pos(Rl((50.0, 18.0))).anchor(Anchor::TopCenter).size(Rl((60.0, 62.0))).pack(),
                    )).with_children(|ui| {

                        // Spawn buttons
                        let gap = 2.0;
                        let size = 15.0;
                        let mut offset = 0.0;
                        for array in [
                            ( "Voice tone", (0..16).collect::<Vec<usize>>()),
                            ( "Skin tone", (0..16).collect()),
                            ( "Skin type", (0..16).collect()),
                            ( "Hairstyle", (0..16).collect()),
                            ( "Hair color", (0..16).collect()),
                            ( "Eyes", (0..16).collect()),
                        ] {

                            ui.spawn((
                                Name::new(array.0),
                                UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack(),
                            )).with_children(|ui| {

                                ui.spawn((
                                    UiLayout::window().size(Rl((100.0, 60.0))).pack(),
                                )).with_children(|ui| {
                                    // Spawn the image
                                    ui.spawn((
                                        UiLayout::window().full().pack(),
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
                                            Text2d::new(array.0.to_uppercase()),
                                            TextFont {
                                                font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                                                font_size: 64.0,
                                                ..default()
                                            },
                                            // Make sure it does not cover the bounding zone of parent
                                            Pickable::IGNORE,
                                        ));
                                    });
                                });

                                ui.spawn((
                                    UiLayout::window().y(Rl(65.0)).size(Rl((48.5, 35.0))).pack(),
                                    OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
                                )).with_children(|ui| {
                                    ui.spawn((
                                        UiLayout::window().full().pack(),
                                        UiHover::new().instant(true),
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.15)),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                                        ]),
                                        Sprite {
                                            image: asset_server.load("images/ui/components/button_sliced_bottom_left.png"),
                                            image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(32.0), ..default() }),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    )).with_children(|ui| {
                                        ui.spawn((
                                            Name::new("Chevron Left"),
                                            UiLayout::window().pos(Rl((50.0, 50.0))).anchor(Anchor::Center).size(Rh(65.0)).pack(),
                                            Sprite::from_image(asset_server.load("images/ui/components/chevron_left.png")),
                                            UiHover::new().forward_speed(20.0).backward_speed(20.0).curve(|v| v.round()),
                                            UiColor::new(vec![
                                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                                (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                                            ]),
                                        ));
                                    });
                                }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                                ui.spawn((
                                    UiLayout::window().x(Rl(51.5)).y(Rl(65.0)).size(Rl((48.5, 35.0))).pack(),
                                    OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
                                )).with_children(|ui| {
                                    ui.spawn((
                                        UiLayout::window().full().pack(),
                                        UiHover::new().instant(true),
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.15)),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                                        ]),
                                        Sprite {
                                            image: asset_server.load("images/ui/components/button_sliced_bottom_right.png"),
                                            image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(32.0), ..default() }),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    )).with_children(|ui| {
                                        ui.spawn((
                                            Name::new("Chevron Right"),
                                            UiLayout::window().pos(Rl((50.0, 50.0))).anchor(Anchor::Center).size(Rh(65.0)).pack(),
                                            Sprite::from_image(asset_server.load("images/ui/components/chevron_right.png")),
                                            UiHover::new().forward_speed(20.0).backward_speed(20.0).curve(|v| v.round()),
                                            UiColor::new(vec![
                                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                                (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2))
                                            ]),
                                        ));
                                    });
                                }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                            });

                            offset += gap + size;
                        }

                    });

                });

            });
        });

        commands
            .spawn((
                // Required to mark this as 3D
                UiRoot3d,
                // Use this constructor to init 3D NewGame
                UiLayoutRoot::new_3d(),
                // Provide default size instead of camera
                Dimension::from((0.818, 0.965)),
            ))
            .with_children(|ui| {
                // Spawn the panel
                ui.spawn((
                    Name::new("Panel"),
                    // Set the layout of this mesh
                    UiLayout::window().full().pack(),
                    // Provide a material to this mesh
                    // MeshMaterial3d(materials.add(StandardMaterial {
                    //     base_color_texture: Some(asset_server.load("images/ui/panel_ull.png")),
                    //     alpha_mode: AlphaMode::Blend,
                    //     unlit: true,
                    //     ..default()
                    // })),
                    // This component will tell Lunex to reconstruct this mesh as plane on demand
                    UiMeshPlane3d,
                ));
            });
    }
}
