use super::organism::gene::Gene;
use bevy::prelude::*;
use config::Config;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{Normal, Uniform, WeightedIndex};
use serde::Deserialize;
use std::cell::RefCell;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_config);
    }
}

fn load_config(mut cmd: Commands) {
    let organism_config = Config::builder()
        .add_source(config::File::with_name("config/organisms.yaml"))
        .build()
        .expect("Can't read organism configuration!")
        .try_deserialize::<OrganismConfig>()
        .expect("Can't deserialize organism config to config struct!");

    let animal_config = Config::builder()
        .add_source(config::File::with_name("config/animals.yaml"))
        .build()
        .expect("Can't read animal configuration!")
        .try_deserialize::<AnimalConfig>()
        .expect("Can't deserialize animal config to config struct!");

    let plant_config = Config::builder()
        .add_source(config::File::with_name("config/plants.yaml"))
        .build()
        .expect("Can't read plant configuration!")
        .try_deserialize::<PlantConfig>()
        .expect("Can't deserialize plant config to config struct!");

    let terrain_config = Config::builder()
        .add_source(config::File::with_name("config/terrain.yaml"))
        .build()
        .expect("Can't read terrain configuration!")
        .try_deserialize::<TerrainConfig>()
        .expect("Can't deserialize terrain config to config struct!");

    let time_config = Config::builder()
        .add_source(config::File::with_name("config/time.yaml"))
        .build()
        .expect("Can't read time configuration!")
        .try_deserialize::<TimeConfig>()
        .expect("Can't deserialize time config to config struct!");

    let environment_config = Config::builder()
        .add_source(config::File::with_name("config/environment.yaml"))
        .build()
        .expect("Can't read environment configuration!")
        .try_deserialize::<EnvironmentConfig>()
        .expect("Can't deserialize environment config to config struct!");

    let data_collection_config = Config::builder()
        .add_source(config::File::with_name("config/data_collection.yaml"))
        .build()
        .expect("Can't read data collection configuration!")
        .try_deserialize::<DataCollectionConfig>()
        .expect("Can't deserialize data collection config to config struct!");

    cmd.insert_resource(SimConfig {
        organism: organism_config,
        animal: animal_config,
        plant: plant_config,
        terrain: terrain_config,
        time: time_config,
        environment: environment_config,
        data_collection: data_collection_config,
    });
}

#[derive(Resource, Debug)]
pub struct SimConfig {
    pub organism: OrganismConfig,
    pub animal: AnimalConfig,
    pub plant: PlantConfig,
    pub terrain: TerrainConfig,
    pub time: TimeConfig,
    pub environment: EnvironmentConfig,
    pub data_collection: DataCollectionConfig,
}

#[derive(Debug, Deserialize)]
pub struct OrganismConfig {
    pub max_health_gene_config: UnsignedFloatGeneConfig,
    pub max_active_energy_gene_config: UnsignedFloatGeneConfig,
    pub age_penalty_gene_config: UnsignedFloatGeneConfig,

    pub energy_to_survive_per_mass_unit_gene_config: UnsignedFloatGeneConfig,
    pub reproduction_energy_cost_gene_config: UnsignedFloatGeneConfig,

    pub maturity_age_gene_config: UnsignedIntGeneConfig,
    // TODO: this is for now to differenciate starting timers
    pub starting_age_dist: DiscreteDistribution,
    pub reproduction_cooldown_gene_config: UnsignedIntGeneConfig,

    pub starting_mass_dist: ContinuousDistribution,

    pub offspring_spawn_range: f32,
    pub max_energy_consumption_per_mass_unit: f32,
    pub carcass_mass_decay_percentage: f32,
}

#[derive(Debug, Deserialize)]
pub struct AnimalConfig {
    pub group_spawn_on_sand_chance: BooleanDistribution,
    pub group_size_dist: DiscreteDistribution,
    pub size_dist: ContinuousDistribution,
    pub diet_dist: DiscreteDistribution,
    pub max_health_dist: ContinuousDistribution,
    pub speed_dist: ContinuousDistribution,
    pub sight_range_dist: ContinuousDistribution,
    pub attack_range_dist: ContinuousDistribution,
    pub attack_damage_dist: ContinuousDistribution,

    // TODO: to już jest w Organism
    // pub development_time: i8,
    // pub waiting_for_reproduction_time: i8,
    pub carnivores_to_herbivores_ratio: f32,
}

#[derive(Debug, Deserialize)]
pub struct PlantConfig {
    pub energy_production_from_solar_efficiency_gene_config: UnsignedFloatGeneConfig,
    pub pollination_range_gene_config: UnsignedFloatGeneConfig,

    pub group_spawn_on_grass_chance: BooleanDistribution,
    pub group_size_dist: DiscreteDistribution,
}

#[derive(Debug, Deserialize)]
pub struct TerrainConfig {
    pub map_width: u32,
    pub map_height: u32,
    pub tile_size: f32,

    pub thermal_overlay_update_cooldown: f32,
    pub biome_overlay_update_cooldown: f32,
}

#[derive(Debug, Deserialize)]
pub struct TimeConfig {
    pub hour_length_in_frames: f32,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentConfig {
    pub starting_hour: u8,
    pub sun_energy_output_per_tile: f32,
    pub sun_energy_output_per_plant: f32,
    pub sun_day_energy_ratio: f32,
    pub sun_night_energy_ratio: f32,
}

#[derive(Debug, Deserialize)]
pub struct DataCollectionConfig {
    pub directory: String,
    pub plants_filename: String,
    pub animals_filename: String,
}

/////////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct UnsignedFloatGeneConfig {
    pub multiplier: f32,
    pub offset: f32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct UnsignedIntGeneConfig {
    pub max_value: u32,
    pub min_value: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum BooleanDistribution {
    Chance { chance: f32 },
}

impl BooleanDistribution {
    pub fn happened(&self) -> bool {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                BooleanDistribution::Chance { chance } => rng.gen_bool(*chance as f64),
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum DiscreteDistribution {
    Range {
        min: u32,
        max: u32,
    },
    Choice {
        choices: Vec<u32>,
    },
    WeightedChoice {
        choices: Vec<u32>,
        weights: Vec<f32>,
    },
}

impl DiscreteDistribution {
    pub fn sample(&self) -> u32 {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                DiscreteDistribution::Range { min, max } => rng.gen_range(*min..*max + 1),
                DiscreteDistribution::Choice { choices } => {
                    choices[rng.gen_range(0..choices.len())]
                }
                DiscreteDistribution::WeightedChoice { choices, weights } => {
                    let dist = WeightedIndex::new(weights).expect("Invalid weights");

                    choices[rng.sample(dist)]
                }
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ContinuousDistribution {
    Normal {
        mean: f32,
        std: f32,
        min: Option<f32>,
        max: Option<f32>,
    },
    Uniform {
        min: f32,
        max: f32,
    },
}

impl ContinuousDistribution {
    pub fn sample(&self) -> f32 {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                ContinuousDistribution::Normal {
                    mean,
                    std,
                    min,
                    max,
                } => {
                    let result = rng.sample(
                        Normal::new(*mean, *std).expect("Failed to create standard distribution"),
                    );

                    result.clamp(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
                }
                ContinuousDistribution::Uniform { min, max } => {
                    rng.sample(Uniform::new(*min, *max))
                }
            }
        })
    }
}
