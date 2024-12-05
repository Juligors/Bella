use bevy::prelude::*;

pub struct RestartPlugin;

impl Plugin for RestartPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimState>()
            .add_systems(
                OnEnter(SimState::LoadConfig),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::Menu),
            )
            .add_systems(
                OnEnter(SimState::Menu),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::LoadAssets),
            )
            .add_systems(
                OnEnter(SimState::LoadAssets),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::TerrainGeneration),
            )
            .add_systems(
                OnEnter(SimState::TerrainGeneration),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::OrganismGeneration),
            )
            .add_systems(
                OnEnter(SimState::OrganismGeneration),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::PreSimulation),
            )
            .add_systems(
                OnEnter(SimState::PreSimulation),
                |mut ns: ResMut<NextState<SimState>>| ns.set(SimState::Simulation),
            )
            .add_systems(Update, restart_simulation_based_on_keyboard_input);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimState {
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
    current_state: Res<State<SimState>>,
    mut next_state: ResMut<NextState<SimState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) && **current_state == SimState::Simulation {
        next_state.set(SimState::LoadConfig)
    }
}
