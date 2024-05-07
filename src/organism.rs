pub mod creature;
pub mod plant;

use bevy::prelude::*;

#[derive(Component)]
pub enum LifeState {
    Alive { hp: f32 },
    Dead,
}
