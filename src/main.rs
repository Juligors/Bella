#![allow(clippy::type_complexity)] // for types like Bevy's Query

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
    let mut app = App::new();

    app.add_plugins((
        MeshPickingPlugin,
        MyWindowPlugin,
        ConfigPlugin,
        TimePlugin,
        EnvironmentPlugin,
        PausePlugin,
        RestartPlugin,
    ))
    .add_plugins(UiPlugin)
    .add_plugins((TerrainPlugin, OrganismPlugin))
    .add_plugins(InspectorPlugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DataCollectionPlugin);

    app.run();
}
