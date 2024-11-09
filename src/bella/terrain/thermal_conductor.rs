use super::{terrain_overlay_state::TerrainOverlayState, TerrainPosition, TileMap};
use crate::bella::{config::SimConfig, pause::PauseState};
use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
use std::time::Duration;

pub struct ThermalConductorPlugin;

impl Plugin for ThermalConductorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_assets_map_temperature)
            .add_systems(
                Update,
                update_tile_color_for_thermal
                    .run_if(in_state(TerrainOverlayState::Thermal))
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(Update, handle_select_input);
    }
}

#[derive(Component)]
pub struct ThermalConductor {
    /// https://pl.wikipedia.org/wiki/Ciep%C5%82o
    pub heat: f32,
    /// With `heat_capacity` we can calculate heat-to-temperature conversion:
    /// `Q = C * delta_T`, where Q is heat, C is heat_capacity and delta_T is temperature difference.
    /// More: https://pl.wikipedia.org/wiki/Pojemno%C5%9B%C4%87_cieplna
    pub heat_capacity: f32,
    /// `Thermal_conductivity` controls how much Heat is transfered between 2 or more `Medium` instances with different temperatures.
    /// More: https://pl.wikipedia.org/wiki/Przewodno%C5%9B%C4%87_cieplna
    pub thermal_conductivity: f32,
}

impl ThermalConductor {
    pub fn temperature(&self) -> f32 {
        self.heat / self.heat_capacity
    }

    pub const fn min_temperature() -> f32 {
        0.
    }

    pub const fn max_temperature() -> f32 {
        100.
    }

    pub const fn default_heat_capacity() -> f32 {
        1000.
    }

    pub const fn default_thermal_conductivity() -> f32 {
        20.
    }

    pub const fn default_heat_lose() -> f32 {
        50.
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ThermalOverlayUpdateTimer(Timer);

pub fn init_thermal_overlay_update_timer(mut cmd: Commands, config: Res<SimConfig>) {
    cmd.insert_resource(ThermalOverlayUpdateTimer(Timer::from_seconds(
        config.terrain.thermal_overlay_update_cooldown,
        TimerMode::Repeating,
    )));
}

pub fn update_temperatures(mut query: Query<(&TerrainPosition, &mut ThermalConductor)>) {
    let (hexes, mut media_org): (Vec<_>, Vec<_>) = query.iter_mut().unzip();

    // we will set all heat at once to make sure order of iteration doesn't matter and energy is conserved
    let mut heat_diffs: Vec<f32> = Vec::with_capacity(media_org.capacity());

    for i in 0..media_org.len() {
        let all_neighbour_hexes = hexes[i].all_neighbors();

        let all_existing_neighbours = hexes
            .iter()
            .enumerate()
            .filter(|(_, hex)| all_neighbour_hexes.contains(hex))
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let heat_diff = all_existing_neighbours
            .iter()
            .map(|&neighbour_i| {
                let temp_diff = media_org[i].temperature() - media_org[neighbour_i].temperature();
                media_org[i].thermal_conductivity * temp_diff
            })
            .sum();

        heat_diffs.push(heat_diff);
    }

    // set all heat at once
    // also lower temperature by constant, like it's getting colder with time
    for i in 0..media_org.len() {
        media_org[i].heat -= heat_diffs[i] + ThermalConductor::default_heat_lose();

        // if temperature is below minimal set it to minimal
        if media_org[i].temperature() < ThermalConductor::min_temperature() {
            media_org[i].heat = ThermalConductor::min_temperature() * media_org[i].heat_capacity;
        }
    }
}

#[derive(Resource)]
pub struct AssetsMapTemperature {
    pub medium_materials: HashMap<i32, Handle<StandardMaterial>>,
    pub default_material_low: Handle<StandardMaterial>,
    pub default_material_high: Handle<StandardMaterial>,
    pub selected_material: Handle<StandardMaterial>,
}
fn initialize_assets_map_temperature(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let default_material_low = materials.add(Color::BLACK);
    let default_material_high = materials.add(Color::WHITE);
    let selected_material = materials.add(Color::linear_rgb(0.2, 0.8, 0.8));

    let mut medium_materials = HashMap::new();

    // create material for every i32 between min and max temperature
    let mut temp = ThermalConductor::min_temperature();
    while temp <= ThermalConductor::max_temperature() {
        medium_materials.insert(
            temp as i32,
            materials.add(Color::srgb(
                (temp - ThermalConductor::min_temperature())
                    / (ThermalConductor::max_temperature() - ThermalConductor::min_temperature()),
                0.1,
                0.2,
            )),
        );
        temp += 1.0;
    }

    cmd.insert_resource(AssetsMapTemperature {
        selected_material,
        medium_materials,
        default_material_low,
        default_material_high,
    });
}
fn update_tile_color_for_thermal(
    mut tiles: Query<(&mut Handle<StandardMaterial>, &ThermalConductor)>,
    assets_map: Res<AssetsMapTemperature>,
    mut timer: ResMut<ThermalOverlayUpdateTimer>,
) {
    if !timer.tick(Duration::from_secs(1)).just_finished() {
        return;
    }

    for (mut handle, medium) in tiles.iter_mut() {
        let temp = medium.heat / medium.heat_capacity;

        *handle = if temp < ThermalConductor::min_temperature() {
            assets_map.default_material_low.clone()
        } else if temp > ThermalConductor::max_temperature() {
            assets_map.default_material_high.clone()
        } else {
            assets_map
            .medium_materials
            .get(&(temp as i32))
            .unwrap_or_else(|| panic!("There should be materials prepared for all int temperatures between min and max temperature. This temp: {}", temp))
            .clone()
        }
    }
}

fn handle_select_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut tiles: Query<(&mut Handle<StandardMaterial>, &mut ThermalConductor, Entity)>,
    map: Res<TileMap>,
) {
    let (camera, camera_transform) = cameras.single();
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if let Some(selected_entity) = map.world_pos_to_entity(pos) {
        for (_handle, mut medium, entity) in tiles.iter_mut() {
            if entity == selected_entity {
                medium.heat = medium.heat_capacity * ThermalConductor::max_temperature();
            }
        }
    }
}
