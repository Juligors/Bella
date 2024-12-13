use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;

pub struct DataCollectionPlugin;

impl Plugin for DataCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_data_collection_directory);
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
