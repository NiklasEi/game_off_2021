use crate::player::{LocalPlayer, PlayerCamera, Player};
use crate::actions::{Actions, TurnDirection};
use bevy::math::Mat2;
use bevy::prelude::*;
use std::ops::{Mul, Deref};
use ggrs::GameInput;

const CAMERA_DISTANCE: f32 = 90.;

#[derive(Component)]
pub struct Orient;

// #[derive(Reflect, Component)]
pub struct PlayerOrientations(pub Vec<Orientation>);

pub enum Orientation {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Orientation {
    pub fn camera_position(&self) -> Vec3 {
        match self {
            &Orientation::North => Vec3::new(-CAMERA_DISTANCE * 2f32.sqrt(), CAMERA_DISTANCE, 0.),
            &Orientation::NorthEast => {
                Vec3::new(-CAMERA_DISTANCE, CAMERA_DISTANCE, -CAMERA_DISTANCE)
            }
            &Orientation::East => Vec3::new(0., CAMERA_DISTANCE, -CAMERA_DISTANCE * 2f32.sqrt()),
            &Orientation::SouthEast => {
                Vec3::new(CAMERA_DISTANCE, CAMERA_DISTANCE, -CAMERA_DISTANCE)
            }
            &Orientation::South => Vec3::new(CAMERA_DISTANCE * 2f32.sqrt(), CAMERA_DISTANCE, 0.),
            &Orientation::SouthWest => Vec3::new(CAMERA_DISTANCE, CAMERA_DISTANCE, CAMERA_DISTANCE),
            &Orientation::West => Vec3::new(0., CAMERA_DISTANCE, CAMERA_DISTANCE * 2f32.sqrt()),
            &Orientation::NorthWest => {
                Vec3::new(-CAMERA_DISTANCE, CAMERA_DISTANCE, CAMERA_DISTANCE)
            }
        }
    }

    fn turn_clockwise(&mut self) {
        match self {
            &mut Orientation::North => *self = Orientation::NorthEast,
            &mut Orientation::NorthEast => *self = Orientation::East,
            &mut Orientation::East => *self = Orientation::SouthEast,
            &mut Orientation::SouthEast => *self = Orientation::South,
            &mut Orientation::South => *self = Orientation::SouthWest,
            &mut Orientation::SouthWest => *self = Orientation::West,
            &mut Orientation::West => *self = Orientation::NorthWest,
            &mut Orientation::NorthWest => *self = Orientation::North,
        }
    }

    fn turn_anti_clockwise(&mut self) {
        match self {
            &mut Orientation::North => *self = Orientation::NorthWest,
            &mut Orientation::NorthEast => *self = Orientation::North,
            &mut Orientation::East => *self = Orientation::NorthEast,
            &mut Orientation::SouthEast => *self = Orientation::East,
            &mut Orientation::South => *self = Orientation::SouthEast,
            &mut Orientation::SouthWest => *self = Orientation::South,
            &mut Orientation::West => *self = Orientation::SouthWest,
            &mut Orientation::NorthWest => *self = Orientation::West,
        }
    }

    fn to_vec(&self) -> Vec2 {
        match self {
            &Orientation::North => Vec2::new(1., 0.),
            &Orientation::NorthEast => Vec2::new(1., 1.).normalize(),
            &Orientation::East => Vec2::new(0., 1.),
            &Orientation::SouthEast => Vec2::new(-1., 1.).normalize(),
            &Orientation::South => Vec2::new(-1., 0.),
            &Orientation::SouthWest => Vec2::new(-1., -1.).normalize(),
            &Orientation::West => Vec2::new(0., -1.),
            &Orientation::NorthWest => Vec2::new(1., -1.).normalize(),
        }
    }

    pub fn orient_movement(&self, movement: Vec2) -> Vec2 {
        let angle = Orientation::North.to_vec().angle_between(self.to_vec());
        Mat2::from_angle(angle).mul(movement)
    }
}

pub fn turn_camera(
    mut orient: Query<&mut Transform, (With<Orient>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Orient>)>,
    mut orientation: ResMut<PlayerOrientations>,
    mut player_query: Query<&Player>,
    inputs: Res<Vec<GameInput>>,
    local_player: Res<LocalPlayer>,
) {
    let mut local_player_turned = false;
    for player in player_query.iter_mut() {
        let input = inputs[player.handle as usize].buffer[0];
        let action: Actions = input.into();

        if let Some(turn) = &action.turn {
            if player.handle as usize == local_player.handle {
                local_player_turned = true;
            }
            if turn == &TurnDirection::Clockwise {
                orientation.0[player.handle as usize].turn_clockwise();
            } else {
                orientation.0[player.handle as usize].turn_anti_clockwise();
            }
        }
    }
    if !local_player_turned {
        return;
    }
    let player_position = orient
        .get_mut(local_player.entity)
        .expect("Player not oriented")
        .translation;
    for mut transform in orient.iter_mut() {
        let current_position = transform.translation;
        transform.look_at(orientation.0[local_player.handle].camera_position() + current_position, Vec3::Y);
    }
    let mut camera_transform = camera.single_mut();
    *camera_transform =
        Transform::from_translation(orientation.0[local_player.handle].camera_position() + player_position)
            .looking_at(player_position, Vec3::Y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_does_not_turn_input() {
        let movement = Vec2::new(1., 0.);
        assert_eq!(
            movement,
            Orientation::North.orient_movement(movement.clone())
        );

        let movement = Vec2::new(0., 1.);
        assert_eq!(
            movement,
            Orientation::North.orient_movement(movement.clone())
        );
    }

    #[test]
    fn neast_turns_input_right() {
        assert_eq!(
            Vec2::new(0., 1.),
            Orientation::East.orient_movement(Vec2::new(1., 0.))
        );

        assert_eq!(
            Vec2::new(-1., 0.),
            Orientation::East.orient_movement(Vec2::new(0., 1.))
        );
    }
}
