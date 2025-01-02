use bevy::prelude::*;
use config::Config;
use serde::Deserialize;

use super::distribution::{BooleanDistribution, ContinuousDistribution, DiscreteDistribution};

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_config);
    }
}

fn load_config(mut cmd: Commands) {
    let organism_config = Config::builder()
        .add_source(config::File::with_name("config/organisms.toml"))
        .build()
        .expect("Can't read organism configuration!")
        .try_deserialize::<OrganismConfig>()
        .expect("Can't deserialize organism config to config struct!");

    let animal_config = Config::builder()
        .add_source(config::File::with_name("config/animals.toml"))
        .build()
        .expect("Can't read animal configuration!")
        .try_deserialize::<AnimalConfig>()
        .expect("Can't deserialize animal config to config struct!");

    let plant_config = Config::builder()
        .add_source(config::File::with_name("config/plants.toml"))
        .build()
        .expect("Can't read plant configuration!")
        .try_deserialize::<PlantConfig>()
        .expect("Can't deserialize plant config to config struct!");

    let terrain_config = Config::builder()
        .add_source(config::File::with_name("config/terrain.toml"))
        .build()
        .expect("Can't read terrain configuration!")
        .try_deserialize::<TerrainConfig>()
        .expect("Can't deserialize terrain config to config struct!");

    let time_config = Config::builder()
        .add_source(config::File::with_name("config/time.toml"))
        .build()
        .expect("Can't read time configuration!")
        .try_deserialize::<TimeConfig>()
        .expect("Can't deserialize time config to config struct!");

    let environment_config = Config::builder()
        .add_source(config::File::with_name("config/environment.toml"))
        .build()
        .expect("Can't read environment configuration!")
        .try_deserialize::<EnvironmentConfig>()
        .expect("Can't deserialize environment config to config struct!");

    cmd.insert_resource(SimConfig {
        organism: organism_config,
        animal: animal_config,
        plant: plant_config,
        terrain: terrain_config,
        time: time_config,
        environment: environment_config,
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
}

#[derive(Debug, Deserialize)]
pub struct OrganismConfig {}

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

    pub development_time: i8,
    pub waiting_for_reproduction_time: i8,
    pub reproduction_range: f32,

    pub carnivores_to_herbivores_ratio: f32,
}

#[derive(Debug, Deserialize)]
pub struct PlantConfig {
    pub group_spawn_on_grass_chance: BooleanDistribution,
    pub group_size_dist: DiscreteDistribution,
    pub size_dist: ContinuousDistribution,
    pub max_health_dist: ContinuousDistribution,
    
    pub reproduction_range: f32,
    pub development_time: i8,
    pub waiting_for_reproduction_time: i8,
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
    pub sun_energy_output: f32,
    pub sun_day_energy_ratio: f32,
    pub sun_night_energy_ratio: f32,
}
