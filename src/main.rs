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
// TODO(LOGS)
// use bevy::log::tracing_subscriber::layer::SubscriberExt;
// use bevy::log::tracing_subscriber::util::SubscriberInitExt;
use bevy::prelude::*;

fn main() {
    // TODO(LOGS)
    // bevy::log::tracing_subscriber::registry()
    //     .with(bevy::log::tracing_subscriber::EnvFilter::from_default_env()) // Enable filtering by environment variables (optional)
    //     // .with(bevy::log::tracing_subscriber::fmt::layer().compact())
    //     .with(
    //         bevy::log::tracing_subscriber::fmt::layer()
    //             .event_format(bevy::log::tracing_subscriber::fmt::format().pretty()),
    //     )
    //     .with(bevy::log::tracing_subscriber::EnvFilter::new("debug"))
    //     .init();

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
