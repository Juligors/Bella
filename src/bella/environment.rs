use bevy::prelude::*;

use super::{config::SimConfig, pause::PauseState, time::HourPassedEvent};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_sun).add_systems(
            Update,
            update_sun_with_time_passing
                .run_if(in_state(PauseState::Running))
                .run_if(on_event::<HourPassedEvent>),
        );
    }
}

#[derive(Resource)]
pub struct Sun {
    day_time: u8,
    energy_output: f32,
    day_energy_ratio: f32,
    night_energy_ratio: f32,
}

impl Sun {
    pub fn is_day(&self) -> bool {
        (5..22).contains(&self.day_time)
    }

    pub fn get_energy_part(&self, surface_percentage: f32) -> f32 {
        assert!(surface_percentage > 0.);
        assert!(surface_percentage <= 1.);

        self.energy_output * surface_percentage * self.get_energy_ratio()
    }

    fn get_energy_ratio(&self) -> f32 {
        if self.is_day() {
            self.day_energy_ratio
        } else {
            self.night_energy_ratio
        }
    }
}

fn create_sun(mut cmd: Commands, config: Res<SimConfig>) {
    cmd.insert_resource(Sun {
        day_time: config.environment.starting_hour,
        energy_output: config.environment.sun_energy_output,
        day_energy_ratio: config.environment.sun_day_energy_ratio,
        night_energy_ratio: config.environment.sun_night_energy_ratio,
    });
}

fn update_sun_with_time_passing(mut sun: ResMut<Sun>) {
    sun.day_time = (sun.day_time + 1) % 24;
}
