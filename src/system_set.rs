use bevy::prelude::*;

pub struct SystemSetPlugin;

impl Plugin for SystemSetPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Startup,
            (
                InitializationSet::ConfigLoad,
                InitializationSet::TerrainGeneration,
                InitializationSet::TerrainVisualization,
            )
                .chain(),
        );
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum InitializationSet {
    ConfigLoad,
    TerrainGeneration,
    TerrainVisualization,
}
