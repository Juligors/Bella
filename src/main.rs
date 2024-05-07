pub mod config;
pub mod layer;
pub mod organism;
pub mod state;
pub mod system_set;
pub mod terrain;
pub mod time;
pub mod ui;

use bevy::prelude::*;

use config::ConfigPlugin;
use layer::LayerPlugin;
use organism::creature::CreaturePlugin;
use organism::plant::PlantPlugin;
use state::StatePlugin;
use system_set::SystemSetPlugin;
use terrain::visualization::TileVisualisationPlugin;
use terrain::TerrainPlugin;
use ui::camera::CameraPlugin;
use ui::window::BasicWindowPlugin;

fn main() {
    App::new()
        .add_plugins(SystemSetPlugin) // TODO: this should probably be done using states, not system_sets
        .add_plugins(ConfigPlugin)
        .add_plugins(StatePlugin)
        .add_plugins((BasicWindowPlugin, CameraPlugin, LayerPlugin))
        .add_plugins((
            TerrainPlugin,
            TileVisualisationPlugin,
            PlantPlugin,
            CreaturePlugin,
        ))
        .run();
}
