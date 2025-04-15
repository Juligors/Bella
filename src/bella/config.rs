use bevy::prelude::*;
use config::Config;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{Normal, Uniform, WeightedIndex};
use serde::Deserialize;
use std::cell::RefCell;

use super::organism::animal::Diet;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_config);
    }
}

fn load_config(mut cmd: Commands) {
    #[cfg(not(feature = "bella_web"))]
    let config = load_config_for_native();
    #[cfg(feature = "bella_web")]
    let config = load_config_for_wasm();

    cmd.insert_resource(config);
}

#[cfg(not(feature = "bella_web"))]
fn load_config_for_native() -> SimulationConfig {
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

    let animal_species_config = AnimalSpeciesAllConfig {
        herbivores_species_config: Config::builder()
            .add_source(config::File::with_name(
                "config/animal_species_herbivores.yaml",
            ))
            .build()
            .expect("Can't read animal species herbivores configuration!")
            .try_deserialize::<AnimalSpeciesConfig>()
            .expect("Can't deserialize animal species herbivores config to config struct!"),
        carnivores_species_config: Config::builder()
            .add_source(config::File::with_name(
                "config/animal_species_carnivores.yaml",
            ))
            .build()
            .expect("Can't read animal species carnivores configuration!")
            .try_deserialize::<AnimalSpeciesConfig>()
            .expect("Can't deserialize animal species carnivores config to config struct!"),
        omnivores_species_config: Config::builder()
            .add_source(config::File::with_name(
                "config/animal_species_omnivores.yaml",
            ))
            .build()
            .expect("Can't read animal species omnivores configuration!")
            .try_deserialize::<AnimalSpeciesConfig>()
            .expect("Can't deserialize animal species omnivores config to config struct!"),
    };

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
        .expect("Can't read data_collection configuration!")
        .try_deserialize::<DataCollectionConfig>()
        .expect("Can't deserialize data_collection config to config struct!");

    let window_config = Config::builder()
        .add_source(config::File::with_name("config/window.yaml"))
        .build()
        .expect("Can't read window configuration!")
        .try_deserialize::<WindowConfig>()
        .expect("Can't deserialize window config to config struct!");

    SimulationConfig {
        organism: organism_config,
        animal: animal_config,
        animal_species: animal_species_config,
        plant: plant_config,
        terrain: terrain_config,
        time: time_config,
        environment: environment_config,
        data_collection: data_collection_config,
        window: window_config,
    }
}

#[cfg(feature = "bella_web")]
fn load_config_for_wasm() -> SimulationConfig {
    let organism_config = OrganismConfig {
        max_health_gene_config: FloatGeneConfig::new(200.0, 0.0),
        max_active_energy_gene_config: FloatGeneConfig::new(1000.0, 0.0),
        reproduction_energy_cost_gene_config: FloatGeneConfig::new(200.0, 0.0),
        age_penalty_gene_config: FloatGeneConfig::new(0.8, 0.2),
        starting_age_dist: DiscreteDistribution::Range { min: 1, max: 24 },
        starting_mass_dist: ContinuousDistribution::Normal {
            mean: 10.0,
            std: 0.5,
            min: Some(1.0),
            max: None,
        },
        offspring_spawn_range: 100.0,
        max_energy_consumption_per_mass_unit: 9.0,
        carcass_mass_decay_percentage: 0.1,
    };

    let animal_config = AnimalConfig {
        group_spawn_chance: BooleanDistribution::Chance { chance: 0.2 },
        group_size_dist: DiscreteDistribution::Range { min: 2, max: 4 },
        size_dist: ContinuousDistribution::Normal {
            mean: 2.0,
            std: 1.0,
            min: Some(1.0),
            max: None,
        },
        diet_dist: DiscreteDistribution::WeightedChoice {
            choices: vec![0, 1, 2],
            weights: vec![0.4, 0.3, 0.3],
        },
        max_health_gene_config: FloatGeneConfig::new(65.0, 10.0),
        speed_gene_config: FloatGeneConfig::new(0.9, 0.1),
        sight_range_gene_config: FloatGeneConfig::new(300.0, 0.0),
        action_range_gene_config: FloatGeneConfig::new(20.0, 0.0),
        attack_damage_gene_config: FloatGeneConfig::new(5.0, 0.0),
        energy_to_survive_per_mass_unit_gene_config: FloatGeneConfig::new(1.0, 0.2),
        do_nothing_for_hours: 2,
        reproduction_cooldown_gene_config: IntGeneConfig::new(6, 18),
        maturity_age_gene_config: IntGeneConfig::new(12, 12),
    };

    let plant_config = PlantConfig {
        energy_production_from_solar_efficiency_gene_config: FloatGeneConfig::new(2.0, 0.0),
        nutrient_consumption_gene_config: FloatGeneConfig::new(2.0, 0.0),
        pollination_range_gene_config: FloatGeneConfig::new(1000.0, 0.0),
        energy_to_survive_per_mass_unit_gene_config: FloatGeneConfig::new(4.0, 1.0),
        group_spawn_on_grass_chance: BooleanDistribution::Chance { chance: 0.2 },
        group_size_dist: DiscreteDistribution::Range { min: 8, max: 16 },
        reproduction_cooldown_gene_config: IntGeneConfig::new(6, 18),
        maturity_age_gene_config: IntGeneConfig::new(6, 6),
    };

    let terrain_config = TerrainConfig {
        map_width: 50,
        map_height: 50,
        tile_size: 100.0,
        thermal_overlay_update_cooldown: 1.0,
        biome_overlay_update_cooldown: 60.0,
        nutrients_per_tile_dirt: 4.0,
        nutrients_per_tile_sand: 2.0,
    };

    let time_config = TimeConfig {
        frames_per_time_unit: 60,
        time_units_per_day: 24,
        close_after_n_days: None,
    };

    let environment_config = EnvironmentConfig {
        starting_hour: 0,
        sun_energy_output_per_tile: 1000.0,
        sun_energy_output_per_plant: 100.0,
        sun_day_energy_ratio: 1.0,
        sun_night_energy_ratio: 0.2,
        water_humidity: 1.0,
        humidity_spread_coefficient: 0.9,
    };

    // NOTE: won't be used on the web anyway
    let data_collection_config = DataCollectionConfig {
        directory: "data".into(),
        plants_filename: "plants.msgpack".into(),
        animals_filename: "animals.msgpack".into(),
    };

    // NOTE: won't be used on the web anyway
    let window_config = WindowConfig {
        width: 0,
        height: 0,
        initial_x: 0,
        initial_y: 0,
    };

    SimulationConfig {
        organism: organism_config,
        animal: animal_config,
        plant: plant_config,
        terrain: terrain_config,
        time: time_config,
        environment: environment_config,
        data_collection: data_collection_config,
        window: window_config,
    }
}

#[derive(Resource, Debug)]
pub struct SimulationConfig {
    pub organism: OrganismConfig,
    pub animal: AnimalConfig,
    pub animal_species: AnimalSpeciesAllConfig,
    pub plant: PlantConfig,
    pub terrain: TerrainConfig,
    pub time: TimeConfig,
    pub environment: EnvironmentConfig,
    pub data_collection: DataCollectionConfig,
    pub window: WindowConfig,
}

#[derive(Debug, Deserialize)]
pub struct OrganismConfig {
    pub max_health_gene_config: FloatGeneConfig,
    pub max_active_energy_gene_config: FloatGeneConfig,
    pub reproduction_energy_cost_gene_config: FloatGeneConfig,
    pub age_penalty_gene_config: FloatGeneConfig,

    pub starting_age_dist: DiscreteDistribution,
    pub starting_mass_dist: ContinuousDistribution,

    pub offspring_spawn_range: f32,
    pub carcass_mass_decay_percentage: f32,
}

#[derive(Debug, Deserialize)]
pub struct AnimalConfig {
    pub group_spawn_chance: BooleanDistribution,
    pub group_size_dist: DiscreteDistribution,
    pub size_dist: ContinuousDistribution,
    pub diet_dist: DiscreteDistribution,
    pub do_nothing_for_hours: u32,
}

#[derive(Debug)]
pub struct AnimalSpeciesAllConfig {
    herbivores_species_config: AnimalSpeciesConfig,
    carnivores_species_config: AnimalSpeciesConfig,
    omnivores_species_config: AnimalSpeciesConfig,
}
impl AnimalSpeciesAllConfig {
    pub fn get(&self, diet: &Diet) -> &AnimalSpeciesConfig {
        match *diet {
            Diet::Herbivore => &self.herbivores_species_config,
            Diet::Carnivore => &self.carnivores_species_config,
            Diet::Omnivore => &self.omnivores_species_config,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AnimalSpeciesConfig {
    pub max_health_gene_config: FloatGeneConfig,
    pub speed_gene_config: FloatGeneConfig,
    pub sight_range_gene_config: FloatGeneConfig,
    pub action_range_gene_config: FloatGeneConfig,
    pub attack_damage_gene_config: FloatGeneConfig,
    pub energy_to_survive_per_mass_unit_gene_config: FloatGeneConfig,
    pub reproduction_cooldown_gene_config: IntGeneConfig,
    pub maturity_age_gene_config: IntGeneConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlantConfig {
    pub energy_production_from_solar_efficiency_gene_config: FloatGeneConfig,
    pub nutrient_consumption_gene_config: FloatGeneConfig,
    pub pollination_range_gene_config: FloatGeneConfig,
    pub energy_to_survive_per_mass_unit_gene_config: FloatGeneConfig,

    pub group_spawn_on_grass_chance: BooleanDistribution,
    pub group_size_dist: DiscreteDistribution,
    pub reproduction_cooldown_gene_config: IntGeneConfig,
    pub maturity_age_gene_config: IntGeneConfig,
}

#[derive(Debug, Deserialize)]
pub struct TerrainConfig {
    pub map_width: u32,
    pub map_height: u32,
    pub tile_size: f32,

    pub thermal_overlay_update_cooldown: f32,
    pub biome_overlay_update_cooldown: f32,

    pub nutrients_per_tile_dirt: f32,
    pub nutrients_per_tile_sand: f32,
}

#[derive(Debug, Deserialize)]
pub struct TimeConfig {
    pub frames_per_time_unit: u64,
    pub time_units_per_day: u64,
    pub close_after_n_days: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentConfig {
    pub starting_hour: u8,
    pub sun_energy_output_per_tile: f32,
    pub sun_energy_output_per_plant: f32,
    pub sun_day_energy_ratio: f32,
    pub sun_night_energy_ratio: f32,

    pub water_humidity: f32,
    pub humidity_spread_coefficient: f32,
}

#[derive(Debug, Deserialize)]
pub struct DataCollectionConfig {
    pub directory: String,
    pub plants_filename: String,
    pub animals_filename: String,
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub initial_x: u32,
    pub initial_y: u32,
}

/////////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct FloatGeneConfig {
    pub multiplier: f32,
    pub offset: f32,
}

impl FloatGeneConfig {
    pub fn new(multiplier: f32, offset: f32) -> Self {
        Self { multiplier, offset }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct IntGeneConfig {
    pub max_value: u32,
    pub min_value: u32,
}

impl IntGeneConfig {
    pub fn new(min_value: u32, max_value: u32) -> Self {
        Self {
            max_value,
            min_value,
        }
    }
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
