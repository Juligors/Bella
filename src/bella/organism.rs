pub mod animal;
pub mod plant;

use bevy::prelude::*;

use self::{animal::AnimalPlugin, plant::PlantPlugin};

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AnimalPlugin, PlantPlugin))
            .add_systems(
                Update,
                (
                    kill_organisms_with_health_below_zero,
                    kill_organisms_with_energy_below_zero,
                    make_ready_to_reproduce_if_possible,
                ),
            )
            .add_systems(PostUpdate, remove_dead_organisms)
            .register_type::<LifeState>()
            .register_type::<Size>()
            .register_type::<EnergyData>()
            .register_type::<ReproductionState>();
    }
}

#[derive(Component, Reflect)]
pub enum LifeState {
    Alive { hp: f32 },
    Dead,
}

#[derive(Component, Reflect)]
pub struct Size {
    base_size: f32,
    ratio: f32,
}

#[derive(Component, Reflect)]
pub struct EnergyData {
    energy: f32,
    production_efficiency: f32,
    energy_needed_for_survival_per_mass_unit: f32,
    energy_needed_for_growth_per_mass_unit: f32,
    grow_by: f32,
}

impl Size {
    pub fn real_size(&self) -> f32 {
        self.base_size * self.ratio
    }

    pub fn real_surface(&self) -> f32 {
        self.base_size * self.ratio.powi(2)
    }

    pub fn real_mass(&self) -> f32 {
        self.base_size * self.ratio.powi(3)
    }
}

#[derive(Component, Reflect)]
pub enum ReproductionState {
    Developing(i8),
    ReadyToReproduce,
    WaitingToReproduce(i8),
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

fn kill_organisms_with_energy_below_zero(mut query: Query<(&EnergyData, &mut LifeState)>) {
    for (energy_data, mut life_state) in query.iter_mut() {
        if energy_data.energy < 0. {
            *life_state = LifeState::Dead;
        }
    }
}

fn remove_dead_organisms(mut cmd: Commands, query: Query<(Entity, &LifeState)>) {
    for (entity, life_state) in query.iter() {
        if matches!(life_state, LifeState::Dead) {
            cmd.entity(entity).despawn_recursive();
        }
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
