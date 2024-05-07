// TODO: do we need it? every thing that needs timer/stopwatch should probably have its own one

// use std::time::Duration;

// use bevy::{prelude::*, time::Stopwatch};

// pub struct SimTimePlugin;

// impl Plugin for SimTimePlugin {
//     fn build(&self, app: &mut App) {
//         app.init_resource::<SimStopwatch>()
//             .add_systems(PreUpdate, tick_sim_stopwatch);
//     }
// }

// #[derive(Resource, Deref, DerefMut)]
// pub struct SimStopwatch(Stopwatch);

// impl Default for SimStopwatch {
//     fn default() -> Self {
//         SimStopwatch(Stopwatch::new())
//     }
// }

// /// System that increases simulation stopwatch by 1 unit (frame). We don't actually use seconds, we just treat seconds as frames.
// fn tick_sim_stopwatch(mut stopwatch: ResMut<SimStopwatch>) {
//     stopwatch.tick(Duration::from_secs(1));
// }
