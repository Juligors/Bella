use super::{config::SimulationConfig, pause::PauseState, restart::SimulationState};
use bevy::prelude::*;
use std::time::Duration;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimeUnitTimer>()
            .register_type::<DayTimer>()
            .register_type::<SimulationTime>()
            .add_event::<TimeUnitPassedEvent>()
            .add_event::<DayPassedEvent>()
            .add_systems(OnExit(SimulationState::Simulation), reset_timers)
            .add_systems(Startup, (init_time, setup_timer_ui))
            .add_systems(
                PreUpdate,
                (update_simulation_time, update_timer_ui)
                    .chain()
                    .run_if(on_event::<TimeUnitPassedEvent>),
            )
            .add_systems(
                PreUpdate,
                (
                    send_time_passed_events_if_needed,
                    send_day_passed_event_if_needed,
                )
                    .run_if(in_state(PauseState::Running)),
            );
    }
}

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct TimeUnitTimer(Timer);

#[derive(Resource, Reflect, Deref, DerefMut)]
pub struct DayTimer(Timer);

#[derive(Resource, Reflect)]
pub struct SimulationTime {
    time_units_passed: u64,
    time_units_per_day: u64,
}

impl SimulationTime {
    pub fn reset(&mut self) {
        self.time_units_passed = 0;
    }

    pub fn time_units_this_day(&self) -> u64{
        self.time_units_passed % self.time_units_per_day
    }

    pub fn days_passed(&self) -> u64 {
        self.time_units_passed / self.time_units_per_day
    }
}

#[derive(Event)]
pub struct TimeUnitPassedEvent;

#[derive(Event)]
pub struct DayPassedEvent;

fn init_time(mut commands: Commands, config: Res<SimulationConfig>) {
    commands.insert_resource(TimeUnitTimer(Timer::from_seconds(
        config.time.frames_per_time_unit as f32,
        TimerMode::Repeating,
    )));

    commands.insert_resource(DayTimer(Timer::from_seconds(
        config.time.time_units_per_day as f32,
        TimerMode::Repeating,
    )));

    commands.insert_resource(SimulationTime {
        time_units_passed: 0,
        time_units_per_day: config.time.time_units_per_day,
    });
}

fn send_time_passed_events_if_needed(
    mut ev_time_unit_passed: EventWriter<TimeUnitPassedEvent>,
    mut timer: ResMut<TimeUnitTimer>,
    mut simulation_time: ResMut<SimulationTime>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ev_time_unit_passed.send(TimeUnitPassedEvent);
        simulation_time.time_units_passed += 1;
    }
}

fn send_day_passed_event_if_needed(
    mut ew_day_passed: EventWriter<DayPassedEvent>,
    mut timer: ResMut<DayTimer>,
) {
    if timer.tick(Duration::from_secs(1)).just_finished() {
        ew_day_passed.send(DayPassedEvent);
    }
}

fn reset_timers(
    mut time_unit_timer: ResMut<TimeUnitTimer>,
    mut day_timer: ResMut<DayTimer>,
    mut simulation_time: ResMut<SimulationTime>,
) {
    time_unit_timer.reset();
    day_timer.reset();
    simulation_time.reset();
}

fn update_simulation_time(mut simulation_time: ResMut<SimulationTime>) {
    simulation_time.time_units_passed += 1;
}

///////////////////// timer ui /////////////////////

#[derive(Component)]
pub struct TimerUiTextMarker;

fn setup_timer_ui(mut commands: Commands) {
    commands.spawn((
        TimerUiTextMarker,
        Text("Day: 0\nTime unit: 0".to_string()),
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
    time_passed: Res<SimulationTime>,
) {
    let mut text = query.single_mut();
    text.0 = format!(
        "Day:{:>2}\nTime unit:{: >2}",
        time_passed.days_passed(),
        time_passed.time_units_this_day()
    );
}
