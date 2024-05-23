use std::time::Duration;

use bevy::prelude::*;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HourPassedEvent>()
            .add_systems(Startup, init_hourly_timer)
            .add_systems(PreUpdate, send_time_events);
    }
}

#[derive(Event)]
pub struct HourPassedEvent;

#[derive(Resource, Deref, DerefMut)]
struct HourlyTimer(Timer);

fn init_hourly_timer(mut cmd: Commands) {
    cmd.insert_resource(HourlyTimer(Timer::from_seconds(60.0, TimerMode::Repeating)));
}

fn send_time_events(
    mut ev_hour_passed: EventWriter<HourPassedEvent>,
    mut timer: ResMut<HourlyTimer>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ev_hour_passed.send(HourPassedEvent);
    }
}
