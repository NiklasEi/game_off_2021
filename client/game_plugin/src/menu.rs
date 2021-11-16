use crate::loading::FontAssets;
use crate::GameState;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

pub struct MenuPlugin;


const CODE_CHARS: &'static [u8] = b"ABCDEFGHKLMNOPRSTUVWXYZ";
const ACCEPTED_KEY_INPUT: [KeyCode; 26] = [
    KeyCode::A,
    KeyCode::B,
    KeyCode::C,
    KeyCode::D,
    KeyCode::E,
    KeyCode::F,
    KeyCode::G,
    KeyCode::H,
    KeyCode::K,
    KeyCode::L,
    KeyCode::M,
    KeyCode::N,
    KeyCode::O,
    KeyCode::P,
    KeyCode::Q,
    KeyCode::R,
    KeyCode::S,
    KeyCode::T,
    KeyCode::U,
    KeyCode::V,
    KeyCode::W,
    KeyCode::X,
    KeyCode::Y,
    KeyCode::Z,
    KeyCode::Back,
    KeyCode::Return
];

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonMaterials>()
            .init_resource::<GameSessionState>()
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(click_new_game_button)
                    .with_system(listen_for_input),
            )
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(clean_menu_ui));
    }
}

#[derive(Component)]
struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        }
    }
}

#[derive(Component)]
struct NewGameButton;

#[derive(Component)]
struct JoinGameButton;

#[derive(Component)]
struct JoinGameText;

#[derive(Component)]
struct NewGameText;

#[derive(Component)]
struct UiElement;

#[derive(Default)]
pub struct GameSessionState {
    pub code: String,
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_materials: Res<ButtonMaterials>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .insert(UiElement)
        .insert(NewGameButton)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Play".to_string(),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                })
                .insert(NewGameText);
        });
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .insert(UiElement)
        .insert(JoinGameButton)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "".to_string(),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                })
                .insert(JoinGameText);
        });
}

fn listen_for_input(
    mut game_session_state: ResMut<GameSessionState>,
    input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut text_query: Query<&mut Text, With<JoinGameText>>,
) {
    input
        .get_just_pressed()
        .filter(|key| ACCEPTED_KEY_INPUT.contains(key))
        .map(|key| {
            let mut text = text_query.single_mut();
            if key == &KeyCode::Return {
                state.set(GameState::Lobby).unwrap();
                return;
            } else if key == &KeyCode::Back {
                text.sections[0].value.pop();
            } else {
                text.sections[0].value.push(format!("{:?}", *key).remove(0));
            }
            game_session_state.code = text.sections[0].value.clone();
            warn!("Current code is {:?}", game_session_state.code);
        })
        .for_each(drop);
}

type ButtonInteraction<'a> = (&'a Interaction, &'a mut Handle<ColorMaterial>);

fn click_new_game_button(
    button_materials: Res<ButtonMaterials>,
    mut state: ResMut<State<GameState>>,
    mut game_session_state: ResMut<GameSessionState>,
    mut interaction_query: Query<ButtonInteraction, (Changed<Interaction>, With<NewGameButton>)>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                let code = create_code();
                game_session_state.code = code;
                state.set(GameState::Lobby).unwrap();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn clean_menu_ui(mut commands: Commands, ui_elements: Query<Entity, With<UiElement>>) {
    for entity in ui_elements.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn create_code() -> String {
    let mut rng = thread_rng();
    (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_CHARS.len());
            CODE_CHARS[idx] as char
        })
        .collect()
}
