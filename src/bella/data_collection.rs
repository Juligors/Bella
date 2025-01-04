use super::{
    config::SimConfig,
    organism::{
        animal::{AnimalMarker, Diet},
        plant::PlantMarker,
        EnergyData, Health, Size,
    },
    time::{HourPassedEvent, SimTime},
};
use bevy::prelude::*;
use serde::Serialize;
use std::{
    io::BufWriter,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct DataCollectionPlugin;

impl Plugin for DataCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (initialize_data_collection_directory,))
            .add_systems(
                Update,
                (save_plant_data, save_animal_data).run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Resource, Deref)]
pub struct DirectoryPath(pub PathBuf);

fn initialize_data_collection_directory(mut cmd: Commands, config: Res<SimConfig>) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let dir_name = format!("simulation_{}", timestamp);
    let path = Path::new(&config.data_collection.directory).join(dir_name);

    std::fs::create_dir_all(&path).expect("Can't ensure path to data collection directory exists");

    cmd.insert_resource(DirectoryPath(path));
}

#[derive(Debug, Serialize)]
pub struct Plant {
    pub id: u64,
    pub hour: u32,
    pub day: u32,
    pub health: f32,
    pub size: f32,

    pub energy: f32,
    pub production_efficiency: f32,
    pub energy_needed_for_survival_per_mass_unit: f32,
    pub energy_needed_for_growth_per_mass_unit: f32,
    pub grow_by: f32,
}

pub fn save_plant_data(
    plants: Query<(Entity, &Health, &Size, &EnergyData), With<PlantMarker>>,
    directory_path: Res<DirectoryPath>,
    time: Res<SimTime>,
    config: Res<SimConfig>,
) {
    let plants: Vec<_> = plants
        .iter()
        .map(|x| Plant {
            id: x.0.to_bits(),
            hour: time.hours,
            day: time.days,
            health: x.1.hp,
            size: x.2.size,
            energy: x.3.energy,
            production_efficiency: x.3.production_efficiency,
            energy_needed_for_survival_per_mass_unit: x.3.energy_needed_for_survival_per_mass_unit,
            energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
            grow_by: x.3.grow_by,
        })
        .collect();

    save_data(
        &plants,
        &directory_path,
        &config.data_collection.plants_filename,
    );
}

#[derive(Debug, Serialize)]
pub struct Animal {
    pub id: u64,
    pub hour: u32,
    pub day: u32,

    pub is_herbivorous: bool,

    pub health: f32,
    pub size: f32,

    pub energy: f32,
    pub production_efficiency: f32,
    pub energy_needed_for_survival_per_mass_unit: f32,
    pub energy_needed_for_growth_per_mass_unit: f32,
    pub grow_by: f32,
}

pub fn save_animal_data(
    animals: Query<(Entity, &Health, &Size, &EnergyData, &Diet), With<AnimalMarker>>,
    directory_path: Res<DirectoryPath>,
    time: Res<SimTime>,
    config: Res<SimConfig>,
) {
    let animals: Vec<_> = animals
        .iter()
        .map(|x| Animal {
            id: x.0.to_bits(),

            hour: time.hours,
            day: time.days,

            is_herbivorous: matches!(x.4, Diet::Herbivorous),

            health: x.1.hp,
            size: x.2.size,
            energy: x.3.energy,
            production_efficiency: x.3.production_efficiency,
            energy_needed_for_survival_per_mass_unit: x.3.energy_needed_for_survival_per_mass_unit,
            energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
            grow_by: x.3.grow_by,
        })
        .collect();

    save_data(
        &animals,
        &directory_path,
        &config.data_collection.animals_filename,
    );
}

const BUFFER_CAPACITY: usize = 1024 * 1024;

fn save_data<T: Serialize>(elements: &[T], directory_path: &DirectoryPath, filename: &str) {
    let path = directory_path.join(filename);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(path.exists())
        // .truncate(true)
        .create(true)
        .open(&path)
        .unwrap();

    let mut writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
    rmp_serde::encode::write(&mut writer, &elements).expect("Failed to serialize data");
}
