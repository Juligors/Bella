pub mod animal;
pub mod plant;

use bevy::prelude::*;

use self::{animal::AnimalPlugin, plant::PlantPlugin};

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AnimalPlugin, PlantPlugin));
    }
}

#[derive(Component)]
pub enum LifeState {
    Alive { hp: f32 },
    Dead,
}
