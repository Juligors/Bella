#![allow(clippy::type_complexity)]

pub mod bella;

use bella::config::ConfigPlugin;
use bella::data_collection::DataCollectionPlugin;
use bella::environment::EnvironmentPlugin;
use bella::inspector::InspectorPlugin;
use bella::organism::OrganismPlugin;
use bella::pause::PausePlugin;
use bella::restart::RestartPlugin;
use bella::terrain::TerrainPlugin;
use bella::time::TimePlugin;
use bella::ui::UiPlugin;
use bella::window::MyWindowPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MeshPickingPlugin,
            MyWindowPlugin,
            ConfigPlugin,
            TimePlugin,
            EnvironmentPlugin,
            PausePlugin,
            RestartPlugin,
            DataCollectionPlugin,
        ))
        .add_plugins(UiPlugin)
        .add_plugins((TerrainPlugin, OrganismPlugin))
        .add_plugins(InspectorPlugin)
        .run();
}
