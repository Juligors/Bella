use super::{
    config::SimulationConfig,
    organism::{
        animal::{AnimalMarker, Diet},
        plant::{PlantEnergyEfficiency, PlantMarker},
        Energy, EnergyDatav3, Health, OrganismEnergyEfficiency,
    },
    time::{SimulationTime, TimeUnitPassedEvent},
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
                PostUpdate,
                (save_plant_data, save_animal_data).run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
}

#[derive(Resource, Deref)]
pub struct DirectoryPath(pub PathBuf);

fn initialize_data_collection_directory(mut cmd: Commands, config: Res<SimulationConfig>) {
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
    pub time_unit: u64,
    pub day: u64,
    // pub health: f32,

    // pub active_energy: Energy,
    // pub max_active_energy: Energy,
    // pub mass: f32,
    // pub energy_per_mass_unit: Energy,
    // pub size: f32,

    // pub production_efficiency: f32,
    // pub energy_needed_for_survival_per_mass_unit: f32,
}

pub fn save_plant_data(
    plants: Query<
        (
            Entity,
            // &Health,
            // &EnergyDatav3,
            // &OrganismEnergyEfficiency,
            // &PlantEnergyEfficiency,
        ),
        With<PlantMarker>,
    >,
    directory_path: Res<DirectoryPath>,
    time: Res<SimulationTime>,
    config: Res<SimulationConfig>,
) {
    let plants: Vec<_> = plants
        .iter()
        .map(
            |(
                entity,
                // health,
                // energy_data,
                // organism_energy_efficiency,
                // plant_organism_efficiency,
            )| Plant {
                id: entity.to_bits(),
                time_unit: time.time_units_this_day(),
                day: time.days_passed(),
                // health: health.hp,

                // active_energy: energy_data.active_energy,
                // max_active_energy: energy_data.max_active_energy,
                // mass: energy_data.mass,
                // energy_per_mass_unit: energy_data.energy_per_mass_unit,
                // size: energy_data.get_size(),

                // production_efficiency: plant_organism_efficiency.production_from_solar,
                // energy_needed_for_survival_per_mass_unit: organism_energy_efficiency
                //     .energy_consumption_to_survive_per_mass_unit,
            },
        )
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
    pub time_unit: u64,
    pub day: u64,

    pub diet: String,
    // pub health: f32,
    // pub size: f32,

    // pub energy: f32,
    // pub production_efficiency: f32,
    // pub energy_needed_for_survival_per_mass_unit: f32,
    // pub energy_needed_for_growth_per_mass_unit: f32,
    // pub grow_by: f32,
}

pub fn save_animal_data(
    animals: Query<(Entity, &Health, &EnergyDatav3, &Diet), With<AnimalMarker>>,
    directory_path: Res<DirectoryPath>,
    time: Res<SimulationTime>,
    config: Res<SimulationConfig>,
) {
    let animals: Vec<_> = animals
        .iter()
        .map(|x| Animal {
            id: x.0.to_bits(),
            time_unit: time.time_units_this_day(),
            day: time.days_passed(),

            diet: match x.3 {
                Diet::Carnivorous => "c",
                Diet::Herbivorous => "h",
                Diet::Omnivore => "o",
            }
            .to_string(),
            // health: x.1.hp,
            // size: x.2.size,
            // energy: x.3.energy,
            // production_efficiency: x.3.production_efficiency,
            // energy_needed_for_survival_per_mass_unit: x.3.energy_needed_for_survival_per_mass_unit,
            // energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
            // grow_by: x.3.grow_by,
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
