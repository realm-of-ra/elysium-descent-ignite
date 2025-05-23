pub mod gameplay;
pub mod main_menu;
pub mod new_game;
pub mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        main_menu::plugin,
        settings::plugin,
        new_game::plugin,
        gameplay::plugin,
    ));
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub(crate) enum Screen {
    #[default]
    Intro,
    MainMenu,
    GamePlay,
    NewGame,
    Settings,
    Inventory,
}
