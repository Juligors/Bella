use bevy::prelude::*;

use crate::bella::ui_facade::EguiFocusState;

pub struct TerrainOverlayStatePlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TerrainOverlayState {
    #[default]
    Bioms,
    Thermal,
}

impl Plugin for TerrainOverlayStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TerrainOverlayState>().add_systems(
            Update,
            change_overlay_state_based_on_keyboard_input
                .run_if(in_state(EguiFocusState::IsNotFocused)),
        );
    }
}

fn change_overlay_state_based_on_keyboard_input(
    mut next_state: ResMut<NextState<TerrainOverlayState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        next_state.set(TerrainOverlayState::Bioms);
    }

    if keyboard_input.just_pressed(KeyCode::F2) {
        next_state.set(TerrainOverlayState::Thermal);
    }
}
