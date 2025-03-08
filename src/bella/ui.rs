pub mod camera;

use bevy::prelude::*;

use self::camera::MyCameraPlugin;

use super::time::{SimulationTime, TimeUnitPassedEvent};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MyCameraPlugin)
            .add_systems(Startup, setup_timer_ui)
            .add_systems(
                PostUpdate,
                update_timer_ui.run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
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
