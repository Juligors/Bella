use bevy::prelude::*;

use super::inspector::EguiFocusState;

pub struct RestartPlugin;

impl Plugin for RestartPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .add_systems(
                OnEnter(SimulationState::LoadConfig),
                |mut ns: ResMut<NextState<SimulationState>>| ns.set(SimulationState::Menu),
            )
            .add_systems(
                OnEnter(SimulationState::Menu),
                |mut ns: ResMut<NextState<SimulationState>>| ns.set(SimulationState::LoadAssets),
            )
            .add_systems(
                OnEnter(SimulationState::LoadAssets),
                |mut ns: ResMut<NextState<SimulationState>>| {
                    ns.set(SimulationState::TerrainGeneration)
                },
            )
            .add_systems(
                OnEnter(SimulationState::TerrainGeneration),
                |mut ns: ResMut<NextState<SimulationState>>| {
                    ns.set(SimulationState::OrganismGeneration)
                },
            )
            .add_systems(
                OnEnter(SimulationState::OrganismGeneration),
                |mut ns: ResMut<NextState<SimulationState>>| ns.set(SimulationState::PreSimulation),
            )
            .add_systems(
                OnEnter(SimulationState::PreSimulation),
                |mut ns: ResMut<NextState<SimulationState>>| ns.set(SimulationState::Simulation),
            )
            .add_systems(
                Update,
                restart_simulation_based_on_keyboard_input
                    .run_if(in_state(EguiFocusState::IsNotFocused)),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimulationState {
    #[default]
    LoadConfig,
    Menu,
    LoadAssets,
    TerrainGeneration,
    OrganismGeneration,
    PreSimulation,
    Simulation,
}

fn restart_simulation_based_on_keyboard_input(
    current_state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) && **current_state == SimulationState::Simulation
    {
        next_state.set(SimulationState::LoadConfig)
    }
}
