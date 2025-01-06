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
            .register_type::<EnergyDatav2>()
            .register_type::<ReproductionState>()
            .register_type::<Tissue>()
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
    pub max_hp: f32,
}

// impl Health{
//     pub fn current_hp_percentage(&self) -> f32{
//         self.hp / self.max * 100.0
//     }
// }

#[derive(Component, Reflect, Debug)]
pub struct EnergyDatav2 {
    // TODO: rename to EnergyData or something elese
    /// currently possesed glycogen/starch energy equivalent
    pub active_energy: Energy,
    /// storage limit of glycogen/starch energy equivalent
    pub max_active_energy: Energy,
    // pub energy: f32, // TODO: this should be only like a temporary thing, maybe even just local variable? And send it over event? That sounds good
    // pub production_efficiency: f32, // TODO: this is only for plants
    // pub energy_needed_for_survival_per_mass_unit: f32, // GENE: jako osobny
    // pub energy_needed_for_growth_per_mass_unit: f32, // GENE: jako osobny
    // pub grow_by: f32, // GENE: jako osobny

    // energy: 1000.,
    // production_efficiency: 0.01,
    // energy_needed_for_survival_per_mass_unit: 5.,
    // energy_needed_for_growth_per_mass_unit: 5.,
    // grow_by: 0.2,
}

impl EnergyDatav2 {
    pub fn consume_energy_with_tissue_if_needed(
        &mut self,
        tissue: &mut Tissue,
        energy_to_consume: Energy,
    ) -> Result<(), ()> {
        if self.active_energy >= energy_to_consume {
            self.active_energy -= energy_to_consume;

            return Ok(());
        }

        tissue.consume_energy_equivalent(energy_to_consume)
    }
}

#[derive(Component, Reflect, Debug)]
pub enum ReproductionState {
    Developing(i8),
    ReadyToReproduce,
    WaitingToReproduce(i8),
}

#[derive(Component, Reflect, Debug)]
pub struct Tissue {
    pub mass: f32,
    pub energy_per_mass_unit: f32,
    pub energy_consumption_per_mass_unit: f32,
}

impl Tissue {
    /// get fat/oil energy equivalent, no limit since there is basically no limit to how much fat/oil an organism can have
    pub fn get_stored_energy(&self) -> Energy {
        self.mass * self.energy_per_mass_unit
    }

    pub fn get_energy_consumption(&self) -> Energy {
        self.mass * self.energy_consumption_per_mass_unit
    }

    /// we assume that there is no density, or density of any tissue is equal to `1`, therefore `mass == size * size * size`
    pub fn get_size(&self) -> f32 {
        self.mass.powf(1.0 / 3.0)
    }

    pub fn store_energy(&mut self, energy: Energy) {
        self.mass += energy / self.energy_per_mass_unit;
    }

    pub fn consume_energy_equivalent(&mut self, energy_to_consume: Energy) -> Result<(), ()> {
        let mass_to_reduce = energy_to_consume * self.energy_per_mass_unit;

        if self.mass < mass_to_reduce {
            return Err(());
        }

        self.mass -= mass_to_reduce;

        Ok(())
    }
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
