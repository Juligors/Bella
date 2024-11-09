use bevy::prelude::*;

pub struct TerrainOverlayStatePlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TerrainOverlayState {
    #[default]
    Bioms,
    Thermal,
}

// TODO: uncomment when we need it
// #[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
// pub enum SimState {
//     #[default]
//     Generation,
//     Simulation,
// }

impl Plugin for TerrainOverlayStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // .init_state::<SimState>()
            .init_state::<TerrainOverlayState>()
            .add_systems(Update, change_overlay_state_based_on_keyboard_input);
    }
}

fn change_overlay_state_based_on_keyboard_input(
    mut next_state: ResMut<NextState<TerrainOverlayState>>,
    // overlay_state: Res<State<TerrainOverlayState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        next_state.set(TerrainOverlayState::Bioms);
    }

    if keyboard_input.just_pressed(KeyCode::F2) {
        next_state.set(TerrainOverlayState::Thermal);
    }
}