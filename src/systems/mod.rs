pub mod audio;
pub mod input;
pub mod movie;
pub mod character_controller;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((audio::plugin,));
}
