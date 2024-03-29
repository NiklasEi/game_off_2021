mod actions;
mod loading;
mod lobby;
mod menu;
mod orientation;
mod player;

use crate::actions::ActionsPlugin;
use crate::loading::LoadingPlugin;
use crate::lobby::LobbyPlugin;
use crate::menu::MenuPlugin;
use crate::orientation::PlayerOrientations;
use crate::player::PlayerPlugin;

use bevy_ggrs::{GGRSApp, GGRSPlugin};

use bevy::prelude::*;

const FPS: u32 = 60;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Lobby,
    Playing,
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Loading)
            .add_plugin(GGRSPlugin)
            .with_update_frequency(FPS)
            .register_rollback_type::<Transform>()
            .register_rollback_type::<PlayerOrientations>()
            .add_plugin(MenuPlugin)
            .add_plugin(LobbyPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(PlayerPlugin);

        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //         .add_plugin(LogDiagnosticsPlugin::default());
        // }
    }
}
