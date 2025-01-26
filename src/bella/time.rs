use super::{config::SimConfig, pause::PauseState, restart::SimState};
use bevy::prelude::*;
use std::time::Duration;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HourlyTimer>()
            .register_type::<DailyTimer>()
            .register_type::<SimTime>()
            .add_event::<HourPassedEvent>()
            .add_event::<DayPassedEvent>()
            .insert_resource(SimTime { days: 0, hours: 0 })
            .add_systems(OnExit(SimState::Simulation), reset_timers)
            .add_systems(
                Startup,
                (init_hourly_timer, init_daily_timer, setup_timer_ui),
            )
            .add_systems(
                PreUpdate,
                (
                    update_simulation_time.run_if(on_event::<HourPassedEvent>),
                    update_timer_ui,
                )
                    .chain()
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(
                PreUpdate,
                (send_hour_passed_event, send_day_passed_event)
                    .run_if(in_state(PauseState::Running)),
            );
    }
}

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct DailyTimer(Timer);

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct HourlyTimer(Timer);

#[derive(Event)]
pub struct HourPassedEvent;

#[derive(Event)]
pub struct DayPassedEvent;

#[derive(Resource, Reflect)]
pub struct SimTime {
    pub days: u32,
    pub hours: u32,
}

fn init_hourly_timer(mut commands: Commands, config: Res<SimConfig>) {
    commands.insert_resource(HourlyTimer(Timer::from_seconds(
        config.time.hour_length_in_frames,
        TimerMode::Repeating,
    )));
}

fn init_daily_timer(mut commands: Commands, config: Res<SimConfig>) {
    commands.insert_resource(DailyTimer(Timer::from_seconds(
        24. * config.time.hour_length_in_frames,
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

fn send_day_passed_event(
    mut ew_day_passed: EventWriter<DayPassedEvent>,
    mut timer: ResMut<DailyTimer>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ew_day_passed.send(DayPassedEvent);
    }
}

fn update_simulation_time(mut time_passed: ResMut<SimTime>) {
    time_passed.hours += 1;

    if time_passed.hours == 24 {
        time_passed.hours = 0;
        time_passed.days += 1;
    }
}

fn reset_timers(
    mut hourly_timer: ResMut<HourlyTimer>,
    mut daily_timer: ResMut<DailyTimer>,
    mut sim_time: ResMut<SimTime>,
) {
    hourly_timer.reset();
    daily_timer.reset();
    *sim_time = SimTime { days: 0, hours: 0 };
}

///////////////////// timer ui /////////////////////

#[derive(Component)]
pub struct TimerUiTextMarker;

fn setup_timer_ui(mut commands: Commands) {
    let initial_hour = 0;
    let initial_day = 0;

    commands.spawn((
        TimerUiTextMarker,
        Text(format!("Day: {}\nHour: {}", initial_day, initial_hour)),
        TextColor::BLACK,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}

fn update_timer_ui(
    mut query: Query<&mut Text, With<TimerUiTextMarker>>,
    time_passed: Res<SimTime>,
) {
    for mut text in query.iter_mut() {
        text.0 = format!(
            "Day:  {: >2}\nHour: {: >2}",
            time_passed.days, time_passed.hours
        );
    }
}
