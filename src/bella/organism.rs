pub mod animal;
pub mod carcass;
pub mod gene;
pub mod plant;

use std::time::Duration;

// use self::{animal::AnimalPlugin, plant::PlantPlugin};
use self::plant::PlantPlugin;
use super::time::HourPassedEvent;
use bevy::prelude::*;
use carcass::CarcassPlugin;
use gene::{GenePlugin, UnsignedFloatGene, UnsignedIntGene};

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlantPlugin, CarcassPlugin, GenePlugin))
            .register_type::<Health>()
            .register_type::<Age>()
            .register_type::<SexualMaturity>()
            .register_type::<EnergyDatav3>()
            .register_type::<OrganismEnergyEfficiency>()
            .add_event::<KillOrganismEvent>()
            .add_event::<ReproduceEvent>()
            .add_systems(
                Update,
                (
                    increase_age,
                    increase_sexual_maturity_level_for_youngs,
                    decrease_reproduction_cooldown_timer_and_add_ready_to_reproduce_marker,
                    consume_energy_to_survive,
                    adjust_size,
                )
                    .run_if(on_event::<HourPassedEvent>),
            );
    }
}

pub type Energy = f32;

#[derive(Bundle)]
pub struct OrganismBundle {
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    transform: Transform,

    health: Health,
    age: Age,
    sexual_maturity: SexualMaturity,
    energy_data: EnergyDatav3,
    organism_energy_efficiency: OrganismEnergyEfficiency,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Health {
    pub hp: f32,
    max_hp_gene: UnsignedFloatGene,
}

impl Health {
    pub fn new(gene: UnsignedFloatGene) -> Self {
        Self {
            hp: gene.phenotype() / 2.0,
            max_hp_gene: gene,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
/// TODO: rename to EnergyData or something else - maybe Tissue again or Tissues? Body?
pub struct EnergyDatav3 {
    /// currently possesed glycogen/starch energy equivalent
    pub active_energy: Energy,
    /// storage limit of glycogen/starch energy equivalent
    pub max_active_energy_gene: UnsignedFloatGene,

    /// current mass, equivalent of fat/body mass/oil
    pub mass: f32,
    /// energy stored as mass per mass unit
    pub energy_per_mass_unit_gene: UnsignedFloatGene,
}

pub enum HungerLevel {
    Satiated,
    Hungry,
}

impl EnergyDatav3 {
    pub fn new(
        max_active_energy_gene: UnsignedFloatGene,
        energy_per_mass_unit_gene: UnsignedFloatGene,
        mass: f32,
    ) -> Self {
        Self {
            active_energy: max_active_energy_gene.phenotype() / 2.0,
            max_active_energy_gene,
            mass,
            energy_per_mass_unit_gene,
        }
    }

    pub fn get_mass_equivalent_of_energy(&self, energy: Energy) -> f32 {
        energy / self.energy_per_mass_unit_gene.phenotype()
    }

    /// Get fat/oil energy equivalent, no limit since there is basically no limit to how much fat/oil an organism can have
    pub fn get_stored_energy(&self) -> Energy {
        self.mass * self.energy_per_mass_unit_gene.phenotype()
    }

    /// We assume that there is no density, or density of any tissue is equal to `1`, therefore `mass == size * size * size`
    pub fn get_size(&self) -> f32 {
        self.mass.powf(1.0 / 3.0)
    }

    /// TODO: right now it works for both animals and plants, but plants don't have hunger...
    pub fn get_hunger_level(&self) -> HungerLevel {
        let active_energy_percentage = self.active_energy / self.max_active_energy_gene.phenotype();
        if active_energy_percentage > 0.5 {
            HungerLevel::Satiated
        } else {
            HungerLevel::Hungry
        }
    }

    /// Store energy, first into active energy, and if there is not enough space left, increase to mass
    pub fn store_energy(&mut self, energy: Energy) {
        let free_active_energy_space = self.max_active_energy_gene.phenotype() - self.active_energy;
        if free_active_energy_space >= energy {
            self.active_energy += energy;
        } else {
            self.active_energy = self.max_active_energy_gene.phenotype();
            let energy_left_to_store = energy - free_active_energy_space;
            let new_mass = self.get_mass_equivalent_of_energy(energy_left_to_store);
            self.mass += new_mass;
        }
    }

    pub fn consume_from_active_energy_and_dmg_if_not_enough(
        &mut self,
        mut energy_to_consume: Energy,
    ) -> Energy {
        if self.active_energy >= energy_to_consume {
            self.active_energy -= energy_to_consume;

            0.0
        } else {
            energy_to_consume -= self.active_energy;
            self.active_energy = 0.0;

            energy_to_consume
        }
    }

    /// Consume energy, first from active energy, and if there is not enough of it, from mass
    pub fn try_to_consume_energy(&mut self, mut energy_to_consume: Energy) -> Result<(), ()> {
        if self.active_energy >= energy_to_consume {
            self.active_energy -= energy_to_consume;

            Ok(())
        } else {
            energy_to_consume -= self.active_energy;

            match self.try_to_consume_energy_directly_from_mass(energy_to_consume) {
                Ok(_) => {
                    self.active_energy = 0.0;

                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    fn try_to_consume_energy_directly_from_mass(
        &mut self,
        energy_to_consume: Energy,
    ) -> Result<(), ()> {
        let mass_to_remove = self.get_mass_equivalent_of_energy(energy_to_consume);

        if self.mass > mass_to_remove {
            self.mass -= mass_to_remove;

            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct OrganismEnergyEfficiency {
    pub energy_consumption_to_survive_per_mass_unit_gene: UnsignedFloatGene,
    pub reproduction_energy_cost_gene: UnsignedFloatGene,
}

impl OrganismEnergyEfficiency {
    pub fn new(
        energy_consumption_to_survive_per_mass_unit_gene: UnsignedFloatGene,
        reproduction_energy_cost_gene: UnsignedFloatGene,
    ) -> Self {
        Self {
            energy_consumption_to_survive_per_mass_unit_gene,
            reproduction_energy_cost_gene,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Age {
    pub value: u32,
    // TODO: to raczej nie powinno być takie ogólne, tylko osobne dla każdego komponentu, który osłabiany jest z wiekiem
    pub age_penalty_gene: UnsignedFloatGene,
}

impl Age {
    pub fn new(age_value: u32, age_penalty_gene: UnsignedFloatGene) -> Self {
        Self {
            value: age_value,
            age_penalty_gene,
        }
    }

    pub fn get_age_penalty(&self) -> f32 {
        self.age_penalty_gene.phenotype() * (self.value as f32).powf(1.0 / 4.0)
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct SexualMaturity {
    pub level: SexualMaturityLevel,
    pub maturity_age_gene: UnsignedIntGene,
    pub reproduction_cooldown_gene: UnsignedIntGene,
    // reproductions_left: u32,
    // max_reproduction_count_gene: UnsignedFloatGene,
}

impl SexualMaturity {
    pub fn new(
        maturity_age_gene: UnsignedIntGene,
        reproduction_cooldown_gene: UnsignedIntGene,
        starting_age: u32
    ) -> Self {
        Self {
            level: SexualMaturityLevel::Young {
                left_to_mature_timer: Timer::new(
                    Duration::from_secs(starting_age as u64),
                    TimerMode::Once,
                ),
            },
            reproduction_cooldown_gene,
            maturity_age_gene,
        }
    }

    pub fn reset_reproduction_cooldown(&mut self) {
        if let SexualMaturityLevel::Adult {
            reproduction_cooldown_timer,
        } = &mut self.level
        {
            reproduction_cooldown_timer.reset();
        } else {
            panic!("Trying to reset reproduction cooldown for not Adult");
        }
    }

    pub fn is_ready_to_reproduce(&self) -> bool {
        if let SexualMaturityLevel::Adult {
            reproduction_cooldown_timer,
        } = &self.level
        {
            return reproduction_cooldown_timer.finished();
        }

        false
    }
}

#[derive(Reflect, Debug, Clone)]
pub enum SexualMaturityLevel {
    Young { left_to_mature_timer: Timer },
    Adult { reproduction_cooldown_timer: Timer },
}

#[derive(Event)]
pub struct ReproduceEvent {
    pub parent1: Entity,
    pub parent2: Entity,
}

#[derive(Component)]
pub struct ReadyToReproduceMarker;

#[derive(Event)]
pub struct KillOrganismEvent {
    entity: Entity,
}

fn increase_age(mut query: Query<&mut Age>) {
    for mut age in query.iter_mut() {
        age.value += 1;
    }
}

fn increase_sexual_maturity_level_for_youngs(mut query: Query<&mut SexualMaturity>) {
    for mut sexual_maturity in query.iter_mut() {
        if let SexualMaturityLevel::Young {
            left_to_mature_timer,
        } = &mut sexual_maturity.level
        {
            if left_to_mature_timer
                .tick(Duration::from_secs(1))
                .just_finished()
            {
                sexual_maturity.level = SexualMaturityLevel::Adult {
                    reproduction_cooldown_timer: Timer::from_seconds(
                        sexual_maturity.reproduction_cooldown_gene.phenotype() as f32,
                        TimerMode::Once,
                    ),
                }
            }
        }
    }
}

fn decrease_reproduction_cooldown_timer_and_add_ready_to_reproduce_marker(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SexualMaturity)>,
) {
    for (entity, mut sexual_maturity) in query.iter_mut() {
        if let SexualMaturityLevel::Adult {
            reproduction_cooldown_timer,
        } = &mut sexual_maturity.level
        {
            if reproduction_cooldown_timer
                .tick(Duration::from_secs(1))
                .just_finished()
            {
                commands.entity(entity).insert(ReadyToReproduceMarker);
            };
        }
    }
}

fn consume_energy_to_survive(
    mut query: Query<(
        Entity,
        &mut EnergyDatav3,
        &OrganismEnergyEfficiency,
        &Age,
        &mut Health,
    )>,
) {
    for (entity, mut energy_data, energy_efficiency, age, mut health) in query.iter_mut() {
        let energy_to_survive = energy_data.mass
            * energy_efficiency
                .energy_consumption_to_survive_per_mass_unit_gene
                .phenotype()
            * age.get_age_penalty();

        trace!(
            "Trying to consume {} energy to survive, active: {}",
            energy_to_survive,
            energy_data.active_energy
        );

        let energy_left_to_consume =
            energy_data.consume_from_active_energy_and_dmg_if_not_enough(energy_to_survive);
        if energy_left_to_consume == 0.0 {
            debug!(
                "Had enough active energy to survive, energy left: {}",
                energy_data.active_energy
            );
        } else {
            debug!(
                "Didn't have enough energy, still needs to consume {} energy",
                energy_left_to_consume
            );
            let dmg = energy_left_to_consume / 10.0;
            health.hp -= dmg;
            debug!("Damaged by {}, hp left: {}", dmg, health.hp);
        }
    }
}

fn adjust_size(mut query: Query<(&mut Transform, &EnergyDatav3)>) {
    for (mut transform, energy_data) in query.iter_mut() {
        let new_size = energy_data.get_size();
        transform.scale = Vec3::splat(new_size);
        transform.translation.z = new_size / 2.0;
    }
}
