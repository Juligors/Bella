use bevy::prelude::*;

use super::{config::SimulationConfig, pause::PauseState, time::TimeUnitPassedEvent};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_sun).add_systems(
            Update,
            update_sun_with_time_passing.run_if(on_event::<TimeUnitPassedEvent>),
        );
    }
}

#[derive(Resource)]
pub struct Sun {
    day_time: u8,
    energy_output_per_tile: f32,
    energy_output_per_plant: f32,
    day_energy_ratio: f32,
    night_energy_ratio: f32,
}

impl Sun {
    pub fn is_day(&self) -> bool {
        (5..22).contains(&self.day_time)
    }

    pub fn get_energy_part_for_tile(&self) -> f32 {
        self.energy_output_per_tile * self.get_energy_ratio()
    }

    pub fn get_energy_for_plant(&self) -> f32 {
        self.energy_output_per_plant
    }

    fn get_energy_ratio(&self) -> f32 {
        if self.is_day() {
            self.day_energy_ratio
        } else {
            self.night_energy_ratio
        }
    }
}

fn create_sun(mut cmd: Commands, config: Res<SimulationConfig>) {
    cmd.insert_resource(Sun {
        day_time: config.environment.starting_hour,
        energy_output_per_tile: config.environment.sun_energy_output_per_tile,
        energy_output_per_plant: config.environment.sun_energy_output_per_plant,
        day_energy_ratio: config.environment.sun_day_energy_ratio,
        night_energy_ratio: config.environment.sun_night_energy_ratio,
    });
}

fn update_sun_with_time_passing(mut sun: ResMut<Sun>) {
    sun.day_time = (sun.day_time + 1) % 24;
    // sun.day_time = 12;
}
