#![allow(clippy::type_complexity)] // for types like Bevy's Query

pub mod bella;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "bella_headless")]
    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(
            core::time::Duration::from_secs_f32(f32::MIN_POSITIVE),
        ));
    #[cfg(not(feature = "bella_headless"))]
    app.add_plugins((bella::window::MyWindowPlugin, MeshPickingPlugin)); // NOTE: it adds DefaultPlugins

    app.add_plugins((
        bella::config::ConfigPlugin,
        bella::ui_facade::UiFacadePlugin,
        bella::time::TimePlugin,
        bella::environment::EnvironmentPlugin,
        bella::pause::PausePlugin,
        bella::restart::RestartPlugin,
        bella::terrain::TerrainPlugin,
        bella::organism::OrganismPlugin,
    ));

    #[cfg(not(feature = "bella_headless"))]
    app.add_plugins((
        bella::ui::UiPlugin,
        bella::inspector::InspectorPlugin,
        bella::organism::animal::gizmos::AnimalGizmosPlugin,
    ));

    #[cfg(not(feature = "bella_web"))]
    app.add_plugins(bella::data_collection::DataCollectionPlugin);

    app.run();
}
