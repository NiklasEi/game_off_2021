mod game_control;
use game_control::*;

use bevy::prelude::*;
use bevy_ggrs::GGRSApp;
use ggrs::PlayerHandle;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .with_input_system(set_actions);
    }
}

#[derive(Default)]
pub struct Actions {
    pub player_movement: Option<Vec2>,
    pub turn: Option<TurnDirection>
}

#[derive(PartialEq)]
pub enum TurnDirection {
    Clockwise,
    AntiClockwise
}

fn set_actions(
    _handle: In<PlayerHandle>,
    time: Res<Time>,
    mut rotation_timer: Local<RotationTimer>,
    mut previous_action: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
) -> Vec<u8> {
    rotation_timer.timer.tick(time.delta());
    previous_action.turn = None;
    if (GameControl::TurnClockwise.pressed(&keyboard_input) || GameControl::TurnAntiClockwise.pressed(&keyboard_input)) && rotation_timer.timer.finished() {
        rotation_timer.timer.reset();

        if GameControl::TurnClockwise.pressed(&keyboard_input) {
            previous_action.turn = Some(TurnDirection::Clockwise);
        } else {
            previous_action.turn = Some(TurnDirection::AntiClockwise);
        }
    }

    if GameControl::Up.just_released(&keyboard_input)
        || GameControl::Up.pressed(&keyboard_input)
        || GameControl::Left.just_released(&keyboard_input)
        || GameControl::Left.pressed(&keyboard_input)
        || GameControl::Down.just_released(&keyboard_input)
        || GameControl::Down.pressed(&keyboard_input)
        || GameControl::Right.just_released(&keyboard_input)
        || GameControl::Right.pressed(&keyboard_input)
    {
        let mut player_movement = Vec2::ZERO;

        if GameControl::Up.just_released(&keyboard_input)
            || GameControl::Down.just_released(&keyboard_input)
        {
            if GameControl::Up.pressed(&keyboard_input) {
                player_movement.x = 1.;
            } else if GameControl::Down.pressed(&keyboard_input) {
                player_movement.x = -1.;
            } else {
                player_movement.x = 0.;
            }
        } else if GameControl::Up.just_pressed(&keyboard_input) {
            player_movement.x = 1.;
        } else if GameControl::Down.just_pressed(&keyboard_input) {
            player_movement.x = -1.;
        } else {
            player_movement.x = previous_action.player_movement.unwrap_or(Vec2::ZERO).x;
        }

        if GameControl::Right.just_released(&keyboard_input)
            || GameControl::Left.just_released(&keyboard_input)
        {
            if GameControl::Right.pressed(&keyboard_input) {
                player_movement.y = 1.;
            } else if GameControl::Left.pressed(&keyboard_input) {
                player_movement.y = -1.;
            } else {
                player_movement.y = 0.;
            }
        } else if GameControl::Right.just_pressed(&keyboard_input) {
            player_movement.y = 1.;
        } else if GameControl::Left.just_pressed(&keyboard_input) {
            player_movement.y = -1.;
        } else {
            player_movement.y = previous_action.player_movement.unwrap_or(Vec2::ZERO).y;
        }

        previous_action.player_movement = Some(player_movement);
    } else {
        previous_action.player_movement = None;
    }

    vec![(&*previous_action as &Actions).into()]
}

pub(crate) struct RotationTimer {
    timer: Timer,
}

impl Default for RotationTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, false),
        }
    }
}
