#![allow(clippy::type_complexity)]

pub mod bella;

use bella::config::ConfigPlugin;
use bella::environment::EnvironmentPlugin;
use bella::organism::OrganismPlugin;
use bella::pause::PausePlugin;
use bella::plots::PlotsPlugin;
use bella::system_set::SystemSetPlugin;
use bella::terrain::TerrainPlugin;
use bella::time::TimePlugin;
use bella::ui::UiPlugin;
use bella::window::MyWindowPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MyWindowPlugin,
            ConfigPlugin,
            SystemSetPlugin,
            TimePlugin,
            EnvironmentPlugin,
            PausePlugin,
        ))
        .add_plugins((UiPlugin, PlotsPlugin))
        .add_plugins((TerrainPlugin, OrganismPlugin))
        .run();
}
