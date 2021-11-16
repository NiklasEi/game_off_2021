use crate::menu::GameSessionState;
use crate::orientation::{Orientation, PlayerOrientations};
use crate::{GameState, FPS};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ggrs::CommandsExt;
use ggrs::PlayerType;
use matchbox_socket::WebRtcNonBlockingSocket;

const INPUT_SIZE: usize = std::mem::size_of::<u8>();

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Args>()
            .add_system_set(
                SystemSet::on_enter(GameState::Lobby)
                    .with_system(lobby_startup)
                    .with_system(start_matchbox_socket),
            )
            .add_system_set(SystemSet::on_update(GameState::Lobby).with_system(lobby_system))
            .add_system_set(SystemSet::on_exit(GameState::Lobby).with_system(lobby_cleanup));
    }
}

fn start_matchbox_socket(mut commands: Commands, args: Res<Args>, task_pool: Res<IoTaskPool>, game_session_state: Res<GameSessionState>) {
    let room_url = format!("{}/{}", &args.matchbox, game_session_state.code.clone());
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcNonBlockingSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(Some(socket));
}

#[derive(Component)]
struct LobbyText;
#[derive(Component)]
struct LobbyUI;

fn lobby_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    session_state: Res<GameSessionState>,
) {
    // All this is just for spawning centered text.
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            material: materials.add(Color::rgb(0.43, 0.41, 0.38).into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    text: {
                        let style = TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 96.,
                            color: Color::BLACK,
                        };
                        let mut text = Text::with_section(
                            "Entering lobby...",
                            style.clone(),
                            Default::default(),
                        );
                        text.sections.push(TextSection {
                            value: format!("\nRoom: {}", session_state.code),
                            style,
                        });
                        text
                    },
                    ..Default::default()
                })
                .insert(LobbyText);
        })
        .insert(LobbyUI);
}

pub struct LocalPlayerHandle(pub usize);

fn lobby_system(
    mut app_state: ResMut<State<GameState>>,
    args: Res<Args>,
    mut socket: ResMut<Option<WebRtcNonBlockingSocket>>,
    mut commands: Commands,
    mut query: Query<&mut Text, With<LobbyText>>
) {
    let socket = socket.as_mut();

    socket.as_mut().unwrap().accept_new_connections();
    let connected_peers = socket.as_ref().unwrap().connected_peers().len() + 1;
    let remaining = args.players - connected_peers;
    query.single_mut().sections[0].value = format!("{} connected", connected_peers);

    if remaining > 0 {
        return;
    }
    info!("All peers have joined, going in-game");

    // consume the socket (currently required because ggrs takes ownership of its socket)
    let socket = socket.take().unwrap();

    // extract final player list
    let players = socket.players();

    // create a GGRS P2P session
    let mut p2p_session =
        ggrs::P2PSession::new_with_socket(args.players as u32, INPUT_SIZE, socket)
            .expect("failed to start with socket");

    // turn on sparse saving
    p2p_session.set_sparse_saving(true).unwrap();

    let mut orientations = vec![];

    for (i, player) in players.into_iter().enumerate() {
        p2p_session
            .add_player(player, i)
            .expect("failed to add player");

        orientations.push(Orientation::North);

        if player == PlayerType::Local {
            // set input delay for the local player
            p2p_session.set_frame_delay(2, i).unwrap();
            info!("Local player has handle {}", i);
            commands.insert_resource(LocalPlayerHandle(i));
        }
    }
    commands.insert_resource(PlayerOrientations(orientations));

    // set default expected update frequency (affects synchronization timings between players)
    p2p_session.set_fps(FPS).unwrap();

    // start the GGRS session
    commands.start_p2p_session(p2p_session);

    app_state
        .set(GameState::Playing)
        .expect("Tried to go in-game while already in-game");
}

fn lobby_cleanup(query: Query<Entity, With<LobbyUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub struct Args {
    pub matchbox: String,
    pub players: usize,
    pub log_filter: String,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            matchbox: "ws://127.0.0.1:3536".to_owned(),
            players: 2,
            log_filter: "info".to_owned(),
        }
    }
}
