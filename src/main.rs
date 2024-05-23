pub mod bella;

use bevy::prelude::*;

use bella::config::ConfigPlugin;
use bella::organism::OrganismPlugin;
use bella::state::StatePlugin;
use bella::system_set::SystemSetPlugin;
use bella::terrain::TerrainPlugin;
use bella::time::TimePlugin;
use bella::ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins((ConfigPlugin, SystemSetPlugin, StatePlugin, TimePlugin))
        .add_plugins(UiPlugin)
        .add_plugins((TerrainPlugin, OrganismPlugin))
        .run();
}
