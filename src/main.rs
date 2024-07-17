pub mod bella;

use bevy::prelude::*;

use bella::config::ConfigPlugin;
use bella::organism::OrganismPlugin;
use bella::state::StatePlugin;
use bella::system_set::SystemSetPlugin;
use bella::terrain::TerrainPlugin;
use bella::time::TimePlugin;
use bella::environment::EnvironmentPlugin;
use bella::ui::UiPlugin;
use bella::plots::PlotsPlugin;

fn main() {
    App::new()
        .add_plugins((ConfigPlugin, SystemSetPlugin, StatePlugin, TimePlugin, EnvironmentPlugin))
        .add_plugins((UiPlugin, PlotsPlugin))
        .add_plugins((TerrainPlugin, OrganismPlugin))
        .run();
}
