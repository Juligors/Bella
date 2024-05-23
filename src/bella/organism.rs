pub mod animal;
pub mod plant;

use bevy::prelude::*;

use self::{animal::AnimalPlugin, plant::PlantPlugin};

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AnimalPlugin, PlantPlugin))
            .add_systems(Update, kill_organisms_with_health_below_zero);
    }
}

#[derive(Component)]
pub enum LifeState {
    Alive { hp: f32 },
    Dead,
}

fn kill_organisms_with_health_below_zero(mut life_states: Query<&mut LifeState>) {
    for mut life_state in life_states.iter_mut() {
        if let LifeState::Alive { hp } = life_state.as_mut() {
            if *hp <= 0. {
                *life_state = LifeState::Dead;
            }
        }
    }
}
