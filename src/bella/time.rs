use std::time::Duration;

use bevy::prelude::*;

use super::config::SimConfig;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HourPassedEvent>()
            .add_event::<DayPassedEvent>()
            .add_systems(Startup, (init_hourly_timer, init_daily_timer))
            .add_systems(PreUpdate, (send_hour_passed_event, send_day_passed_event));
    }
}

///////////////////// hourly /////////////////////

#[derive(Event)]
pub struct HourPassedEvent;

#[derive(Resource, Deref, DerefMut)]
struct HourlyTimer(Timer);

fn init_hourly_timer(mut cmd: Commands, config: Res<SimConfig>) {
    cmd.insert_resource(HourlyTimer(Timer::from_seconds(
        config.time.hour_length_in_frames,
        TimerMode::Repeating,
    )));
}

fn send_hour_passed_event(
    mut ev_hour_passed: EventWriter<HourPassedEvent>,
    mut timer: ResMut<HourlyTimer>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ev_hour_passed.send(HourPassedEvent);
    }
}

///////////////////// daily /////////////////////

#[derive(Event)]
pub struct DayPassedEvent;

#[derive(Resource, Deref, DerefMut)]
struct DailyTimer(Timer);

fn init_daily_timer(mut cmd: Commands, config: Res<SimConfig>) {
    cmd.insert_resource(DailyTimer(Timer::from_seconds(
        24. * config.time.hour_length_in_frames,
        TimerMode::Repeating,
    )));
}

fn send_day_passed_event(
    mut ev_day_passed: EventWriter<DayPassedEvent>,
    mut timer: ResMut<DailyTimer>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ev_day_passed.send(DayPassedEvent);
    }
}
