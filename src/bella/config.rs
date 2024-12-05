use bevy::prelude::*;
use config::Config;

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

#[derive(Resource)]
pub struct SimConfig {
    pub organism: OrganismConfig,
    pub animal: AnimalConfig,
    pub plant: PlantConfig,
    pub terrain: TerrainConfig,
    pub time: TimeConfig,
    pub environment: EnvironmentConfig,
}

#[derive(serde::Deserialize)]
pub struct OrganismConfig {}

#[derive(serde::Deserialize)]
pub struct AnimalConfig {
    pub group_spawn_chance_sand: f32,
    pub group_size_min: u32,
    pub group_size_max: u32,

    pub development_time: i8,
    pub waiting_for_reproduction_time: i8,

    pub carnivores_to_herbivores_ratio: f32,
}

#[derive(serde::Deserialize)]
pub struct PlantConfig {
    pub group_spawn_chance_grass: f32,
    pub group_size_min: u32,
    pub group_size_max: u32,

    pub development_time: i8,
    pub waiting_for_reproduction_time: i8,
}

#[derive(serde::Deserialize)]
pub struct TerrainConfig {
    pub map_radius: u32,
    pub hex_size: f32,

    pub thermal_overlay_update_cooldown: f32,
    pub biome_overlay_update_cooldown: f32,
}

impl TerrainConfig {
    pub fn hex_surface(&self) -> f32 {
        self.hex_size * self.hex_size * 3. * f32::sqrt(3.) / 8.
    }
}

#[derive(serde::Deserialize)]
pub struct TimeConfig {
    pub hour_length_in_frames: f32,
}

#[derive(serde::Deserialize)]
pub struct EnvironmentConfig {
    pub starting_hour: u8,
    pub sun_energy_output: f32,
    pub sun_day_energy_ratio: f32,
    pub sun_night_energy_ratio: f32,
}
