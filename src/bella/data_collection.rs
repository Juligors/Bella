use super::{
    organism::{plant::PlantMarker, EnergyData, Health, Size},
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
                (save_plant_data,).run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Resource)]
pub struct DataCollectionDirectory(pub PathBuf);

fn initialize_data_collection_directory(mut cmd: Commands) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let dir_name = format!("simulation_{}", timestamp);
    let path = Path::new("data").join(dir_name);

    std::fs::create_dir_all(&path).expect("Can't ensure path to data collection directory exists");

    cmd.insert_resource(DataCollectionDirectory(path));
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
    path: Res<DataCollectionDirectory>,
    time: Res<SimTime>,
) {
    let path = path.0.join("plants.msgpack");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(path.exists())
        // .truncate(true)
        .create(true)
        .open(&path)
        .unwrap();

    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    let plants: Vec<Plant> = plants
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

    rmp_serde::encode::write(&mut writer, &plants).expect("Failed to serialize data");
}

// mod data_collection {
//     use super::*;
//     use crate::bella::{data_collection::DataCollectionDirectory, time::SimTime};

//     #[derive(Debug, serde::Serialize)]
//     pub struct Animal {
//         pub id: u64,
//         pub hour: u32,
//         pub day: u32,

//         pub is_herbivorous: bool,

//         pub health: f32,
//         pub size: f32,

//         pub energy: f32,
//         pub production_efficiency: f32,
//         pub energy_needed_for_survival_per_mass_unit: f32,
//         pub energy_needed_for_growth_per_mass_unit: f32,
//         pub grow_by: f32,
//     }

//     pub fn save_animal_data(
//         animals: Query<(Entity, &Health, &Size, &EnergyData, &Diet), With<AnimalMarker>>,
//         path: Res<DataCollectionDirectory>,
//         time: Res<SimTime>,
//     ) {
//         let path = path.0.join("animals.csv");
//         let mut file = std::fs::OpenOptions::new()
//             .write(true)
//             .append(path.exists())
//             // .truncate(true)
//             .create(true)
//             .open(path)
//             .unwrap();

//         let needs_headers = std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0)).unwrap() == 0;

//         let mut writer = csv::WriterBuilder::new()
//             .delimiter(b'|')
//             .has_headers(needs_headers)
//             .from_writer(file);

//         for x in animals.iter() {
//             let animal_record = Animal {
//                 id: x.0.to_bits(),

//                 hour: time.hours,
//                 day: time.days,

//                 is_herbivorous: matches!(x.4, Diet::Herbivorous),

//                 health: x.1.hp,
//                 size: x.2.size,
//                 energy: x.3.energy,
//                 production_efficiency: x.3.production_efficiency,
//                 energy_needed_for_survival_per_mass_unit: x
//                     .3
//                     .energy_needed_for_survival_per_mass_unit,
//                 energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
//                 grow_by: x.3.grow_by,
//             };

//             writer
//                 .serialize(&animal_record)
//                 .unwrap_or_else(|_| panic!("Couldn't serialize object {:?}", animal_record));
//         }

//         writer
//             .flush()
//             .expect("Couldn't save new animal data to a file");
//     }
// }
