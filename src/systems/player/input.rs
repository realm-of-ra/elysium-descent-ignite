use bevy::{log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LogPlugin::default())
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<PlayerControl>()
        .add_observer(player_binding);
}

fn player_binding(
    trigger: Trigger<Binding<PlayerControl>>,
    mut players: Query<&mut Actions<PlayerControl>>,
) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()));
    actions
        .bind::<Attack>()
        .to((MouseButton::Left, KeyCode::KeyF, GamepadButton::West));
}

#[derive(InputContext)]
struct PlayerControl;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Attack;
