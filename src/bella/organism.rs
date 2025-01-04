pub mod animal;
pub mod carcass;
pub mod plant;

use bevy::prelude::*;
use carcass::CarcassPlugin;

use self::{animal::AnimalPlugin, plant::PlantPlugin};

use super::pause::PauseState;

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AnimalPlugin, PlantPlugin, CarcassPlugin))
            .register_type::<Health>()
            .register_type::<Size>()
            .register_type::<EnergyData>()
            .register_type::<ReproductionState>()
            .register_type::<Meat>()
            .register_type::<PlantMatter>()
            .add_systems(
                Update,
                (make_ready_to_reproduce_if_possible,).run_if(in_state(PauseState::Running)),
            );
    }
}

type Energy = f32;

#[derive(Component, Reflect, Debug)]
pub struct Health {
    pub hp: f32,
    pub max: f32,
}

// impl Health{
//     pub fn current_hp_percentage(&self) -> f32{
//         self.hp / self.max * 100.0
//     }
// }

#[derive(Component, Reflect, Debug)]
pub struct Size {
    pub size: f32,
}

impl Size {
    pub fn real_size(&self) -> f32 {
        self.size
    }

    pub fn real_surface(&self) -> f32 {
        self.size.powi(2)
    }

    pub fn real_volume(&self) -> f32 {
        self.size.powi(3)
    }
}

#[derive(Component, Reflect, Debug)]
pub struct EnergyData {
    pub energy: f32, // TODO: this should be only like a temporary thing, maybe even just local variable? And send it over event? That sounds good
    pub production_efficiency: f32, // TODO: this is only for plants
    pub energy_needed_for_survival_per_mass_unit: f32,
    pub energy_needed_for_growth_per_mass_unit: f32,
    pub grow_by: f32,
}

#[derive(Component, Reflect, Debug)]
pub enum ReproductionState {
    Developing(i8),
    ReadyToReproduce,
    WaitingToReproduce(i8),
}

#[derive(Component, Reflect, Debug)]
pub struct Meat {
    pub current_energy: Energy,
}

#[derive(Component, Reflect, Debug)]
pub struct PlantMatter {
    pub current_energy: Energy,
}

fn make_ready_to_reproduce_if_possible(mut query: Query<&mut ReproductionState>) {
    for mut reproduction_state in query.iter_mut() {
        match *reproduction_state {
            ReproductionState::Developing(remaining_time) => {
                if remaining_time <= 0 {
                    *reproduction_state = ReproductionState::ReadyToReproduce;
                }
            }
            ReproductionState::WaitingToReproduce(cooldown) => {
                if cooldown <= 0 {
                    *reproduction_state = ReproductionState::ReadyToReproduce;
                }
            }
            ReproductionState::ReadyToReproduce => (),
        }
    }
}
