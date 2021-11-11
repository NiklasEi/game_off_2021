use crate::player::{LocalPlayer, PlayerCamera};
use crate::GameState;
use bevy::math::Mat2;
use bevy::prelude::*;
use std::ops::Mul;

const CAMERA_DISTANCE: f32 = 90.;

pub struct OrientationPlugin;

impl Plugin for OrientationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Orientation::North)
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(turn_camera));
    }
}

#[derive(Component)]
pub(crate) struct Orient;

pub(crate) enum Orientation {
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

    fn turn_clockwise(&self) -> Self {
        match self {
            &Orientation::North => Orientation::NorthEast,
            &Orientation::NorthEast => Orientation::East,
            &Orientation::East => Orientation::SouthEast,
            &Orientation::SouthEast => Orientation::South,
            &Orientation::South => Orientation::SouthWest,
            &Orientation::SouthWest => Orientation::West,
            &Orientation::West => Orientation::NorthWest,
            &Orientation::NorthWest => Orientation::North,
        }
    }

    fn turn_anti_clockwise(&self) -> Self {
        match self {
            &Orientation::North => Orientation::NorthWest,
            &Orientation::NorthEast => Orientation::North,
            &Orientation::East => Orientation::NorthEast,
            &Orientation::SouthEast => Orientation::East,
            &Orientation::South => Orientation::SouthEast,
            &Orientation::SouthWest => Orientation::South,
            &Orientation::West => Orientation::SouthWest,
            &Orientation::NorthWest => Orientation::West,
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

fn turn_camera(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut rotation_timer: Local<RotationTimer>,
    local_player: Query<Entity, With<LocalPlayer>>,
    mut orient: Query<&mut Transform, (With<Orient>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Orient>)>,
    mut orientation: ResMut<Orientation>,
) {
    rotation_timer.timer.tick(time.delta());
    if !input.pressed(KeyCode::Q) && !input.pressed(KeyCode::E) {
        return;
    }

    if !rotation_timer.timer.finished() {
        return;
    }
    rotation_timer.timer.reset();

    if input.pressed(KeyCode::Q) {
        *orientation = orientation.turn_anti_clockwise();
    } else {
        *orientation = orientation.turn_clockwise();
    }
    let player_position = orient
        .get_mut(local_player.get_single().expect("No player? O.o"))
        .expect("Player not oriented")
        .translation;
    for mut transform in orient.iter_mut() {
        let current_position = transform.translation;
        transform.look_at(orientation.camera_position() + current_position, Vec3::Y);
    }
    let mut camera_transform = camera.single_mut();
    *camera_transform =
        Transform::from_translation(orientation.camera_position() + player_position)
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
