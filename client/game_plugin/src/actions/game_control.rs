use crate::actions::{Actions, TurnDirection};
use bevy::prelude::*;

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_LEFT: u8 = 1 << 2;
pub const INPUT_RIGHT: u8 = 1 << 3;
pub const INPUT_TURN_CLOCKWISE: u8 = 1 << 4;
pub const INPUT_TURN_ANTI_CLOCKWISE: u8 = 1 << 5;

impl From<&Actions> for u8 {
    fn from(actions: &Actions) -> Self {
        let mut input: u8 = 0;

        if let Some(movement) = actions.player_movement {
            if movement.x > 0. {
                input |= INPUT_UP;
            }
            if movement.x < 0. {
                input |= INPUT_DOWN;
            }
            if movement.y > 0. {
                input |= INPUT_RIGHT;
            }
            if movement.y < 0. {
                input |= INPUT_LEFT;
            }
        }

        if let Some(turn) = &actions.turn {
            if turn == &TurnDirection::Clockwise {
                input |= INPUT_TURN_CLOCKWISE;
            } else {
                input |= INPUT_TURN_ANTI_CLOCKWISE;
            }
        }

        input
    }
}

impl From<u8> for Actions {
    fn from(input: u8) -> Self {
        let mut player_movement = Vec2::ZERO;

        if input & INPUT_UP != 0 {
            player_movement.x = 1.;
        }
        if input & INPUT_DOWN != 0 {
            player_movement.x = -1.;
        }
        if input & INPUT_RIGHT != 0 {
            player_movement.y = 1.;
        }
        if input & INPUT_LEFT != 0 {
            player_movement.y = -1.;
        }

        if player_movement != Vec2::ZERO {
            player_movement = player_movement.normalize();
        }

        let turn = if input & INPUT_TURN_CLOCKWISE != 0 {
            Some(TurnDirection::Clockwise)
        } else if input & INPUT_TURN_ANTI_CLOCKWISE != 0 {
            Some(TurnDirection::AntiClockwise)
        } else {
            None
        };

        Actions {
            player_movement: Some(player_movement),
            turn,
        }
    }
}

pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
    TurnClockwise,
    TurnAntiClockwise,
}

impl GameControl {
    pub fn just_released(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_released(KeyCode::W)
                    || keyboard_input.just_released(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_released(KeyCode::S)
                    || keyboard_input.just_released(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_released(KeyCode::A)
                    || keyboard_input.just_released(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_released(KeyCode::D)
                    || keyboard_input.just_released(KeyCode::Right)
            }
            GameControl::TurnClockwise => keyboard_input.just_released(KeyCode::E),
            GameControl::TurnAntiClockwise => keyboard_input.just_released(KeyCode::Q),
        }
    }

    pub fn pressed(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right)
            }
            GameControl::TurnClockwise => keyboard_input.pressed(KeyCode::E),
            GameControl::TurnAntiClockwise => keyboard_input.pressed(KeyCode::Q),
        }
    }

    pub fn just_pressed(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.just_pressed(KeyCode::S)
                    || keyboard_input.just_pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.just_pressed(KeyCode::A)
                    || keyboard_input.just_pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.just_pressed(KeyCode::D)
                    || keyboard_input.just_pressed(KeyCode::Right)
            }
            GameControl::TurnClockwise => keyboard_input.just_pressed(KeyCode::E),
            GameControl::TurnAntiClockwise => keyboard_input.just_pressed(KeyCode::Q),
        }
    }
}
