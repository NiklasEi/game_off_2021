use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::lobby::LocalPlayerHandle;
use crate::orientation::{Orient, Orientation, turn_camera, PlayerOrientations};
use crate::GameState;
use bevy::prelude::*;
use bevy_ggrs::{GGRSApp, Rollback, RollbackIdProvider};
use ggrs::{GameInput, P2PSession};
use bevy::pbr::AmbientLight;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct LocalPlayer {
    pub handle: usize,
    pub entity: Entity
}

#[derive(Component)]
pub struct Player {
    pub handle: u32,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let mut rollback_schedule = Schedule::default();
        let mut default_stage = SystemStage::parallel();
        default_stage.add_system(move_player);
        default_stage.add_system(turn_camera);
        rollback_schedule.add_stage("default_rollback_stage", default_stage);
        app.insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.4,
        }).add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_player)
                .with_system(spawn_camera)
                .with_system(spawn_tree)
                .with_system(spawn_map),
        )
        .with_rollback_schedule(rollback_schedule);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_camera(
    mut commands: Commands,
    orientation: Res<PlayerOrientations>,
    local_player: Res<LocalPlayerHandle>,
) {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 100.0;
    camera.transform =
        Transform::from_translation(orientation.0[local_player.0].camera_position()).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands
        .spawn_bundle(camera)
        .insert(PlayerCamera);
}

fn spawn_tree(
    mut commands: Commands,
    texture_assets: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    orientation: Res<PlayerOrientations>,
    local_player: Res<LocalPlayerHandle>,
) {
    let tree_positions = vec![Vec3::new(64., 32., 0.), Vec3::new(0., 32., 250.)];
    for position in tree_positions {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(texture_assets.tree.clone().into()),
                transform: Transform::from_translation(position)
                    .looking_at(orientation.0[local_player.0].camera_position() + position, Vec3::Y),
                ..Default::default()
            })
            .insert(Orient);
    }
}

fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    texture_assets: Res<TextureAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for column in -5..=5 {
        for row in -5..=5 {
            let texture = if column == row {
                texture_assets.ground.clone()
            } else {
                texture_assets.grass.clone()
            };
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 64.0 })),
                material: materials.add(texture.into()),
                transform: Transform::from_translation(Vec3::new(
                    64. * column.clone() as f32,
                    0.,
                    64. * row as f32,
                )),
                ..Default::default()
            });
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    p2p_session: Option<Res<P2PSession>>,
    orientation: Res<PlayerOrientations>,
    local_player: Res<LocalPlayerHandle>,
) {
    let num_players = p2p_session
        .map(|s| s.num_players())
        .expect("No GGRS session found");

    for handle in 0..num_players {
        let mut entity = commands.spawn_bundle(SpriteBundle {
            material: materials.add(textures.player.clone().into()),
            transform: Transform::from_xyz(0., 32., 0.)
                .looking_at(orientation.0[local_player.0].camera_position(), Vec3::Y),
            ..Default::default()
        });
        entity
            .insert(Player { handle })
            .insert(Rollback::new(rip.next_id()))
            .insert(Orient);
        if local_player.0 == handle as usize {
            let entity_id = entity.id();
            commands.insert_resource(LocalPlayer {
                handle: handle as usize,
                entity: entity_id
            });
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player), With<Rollback>>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    orientation: Res<PlayerOrientations>,
    inputs: Res<Vec<GameInput>>,
    local_player: Res<LocalPlayerHandle>,
) {
    let speed = 3.;
    for (mut player_transform, player) in player_query.iter_mut() {
        let input = inputs[player.handle as usize].buffer[0];
        let action: Actions = input.into();

        if let Some(mut movement) = action.player_movement {
            movement = orientation.0[player.handle as usize].orient_movement(movement);
            movement *= speed;
            player_transform.translation += Vec3::new(movement.x, 0., movement.y);

            if local_player.0 == player.handle as usize {
                let mut camera_position = camera.single_mut();
                camera_position.translation = player_transform.translation + orientation.0[player.handle as usize].camera_position();
            }
        }
    }
}
