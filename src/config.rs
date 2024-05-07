use bevy::prelude::*;
use config::Config;

use crate::system_set::InitializationSet;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_config.in_set(InitializationSet::ConfigLoad));
    }
}

fn load_config(mut cmd: Commands) {
    let config = Config::builder()
        .add_source(config::File::with_name("config/basic.toml"))
        .build()
        .expect("Can't read configuration!")
        .try_deserialize::<SimConfig>()
        .expect("Can't deserialize config to config struct!");

    cmd.insert_resource(config);
}

#[derive(Resource, serde::Deserialize)]
pub struct SimConfig {
    pub creature_spawn_x: i32,
    pub creature_spawn_y: i32,

    pub plant_spawn_y: i32,
    pub plant_spawn_x: i32,

    pub map_radius: u32,
    pub hex_size: f32,

    pub thermal_overlay_update_cooldown: f32,
    pub biome_overlay_update_cooldown: f32,
}