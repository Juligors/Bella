use bevy::prelude::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .add_systems(Update, change_overlay_state_based_on_keyboard_input);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

fn change_overlay_state_based_on_keyboard_input(
    current_state: Res<State<PauseState>>,
    mut next_state: ResMut<NextState<PauseState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_state.set(match **current_state {
            PauseState::Paused => PauseState::Running,
            PauseState::Running => PauseState::Paused,
        });
    }
}
