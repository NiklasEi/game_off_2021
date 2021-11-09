use crate::actions::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP};
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_ggrs::{Rollback, RollbackIdProvider};
use ggrs::{GameInput, P2PSession};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player {
    pub handle: u32,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_player)
                .with_system(spawn_camera),
        )
        .add_rollback_system(move_player);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_player(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    p2p_session: Option<Res<P2PSession>>,
) {
    let num_players = p2p_session
        .map(|s| s.num_players())
        .expect("No GGRS session found");

    for handle in 0..num_players {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(textures.texture_bevy.clone().into()),
                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                ..Default::default()
            })
            .insert(Player { handle })
            .insert(Rollback::new(rip.next_id()));
    }
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player), With<Rollback>>,
    inputs: Res<Vec<GameInput>>,
) {
    let speed = 3.;

    for (mut player_transform, p) in player_query.iter_mut() {
        let input = inputs[p.handle as usize].buffer[0];
        if input & INPUT_UP != 0 && input & INPUT_DOWN == 0 {
            player_transform.translation.y -= speed;
        }
        if input & INPUT_UP == 0 && input & INPUT_DOWN != 0 {
            player_transform.translation.y += speed;
        }
        if input & INPUT_LEFT != 0 && input & INPUT_RIGHT == 0 {
            player_transform.translation.x -= speed;
        }
        if input & INPUT_LEFT == 0 && input & INPUT_RIGHT != 0 {
            player_transform.translation.x += speed;
        }
    }
}
