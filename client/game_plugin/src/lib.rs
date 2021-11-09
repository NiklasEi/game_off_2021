mod actions;
mod loading;
mod lobby;
mod menu;
mod player;

use crate::actions::ActionsPlugin;
use crate::loading::LoadingPlugin;
use crate::lobby::LobbyPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy_ggrs::{GGRSApp, GGRSPlugin};

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
            .with_fps(FPS)
            .register_rollback_type::<Transform>()
            .insert_resource(FrameCount { frame: 0 })
            .register_rollback_type::<FrameCount>()
            .add_rollback_system(increase_frame_system)
            .add_plugin(LobbyPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(PlayerPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}

// You can also register resources. If your Component / Resource implements Hash, you can make use of `#[reflect(Hash)]`
// in order to allow a GGRS `SyncTestSession` to construct a checksum for a world snapshot
#[derive(Default, Reflect, Hash)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    println!("Frame is {}", frame_count.frame);
    frame_count.frame += 1;
}
