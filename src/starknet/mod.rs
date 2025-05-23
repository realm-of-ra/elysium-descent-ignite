pub mod config;
pub mod starknet;
pub mod tokio;

use bevy::prelude::*;

use starknet::StarknetPlugin;
use tokio::TokioPlugin;

pub struct NetworkingPlugin;
impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StarknetPlugin);
        app.add_plugins(TokioPlugin);
    }
}
