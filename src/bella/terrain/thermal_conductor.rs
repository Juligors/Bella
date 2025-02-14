use super::{
    terrain_overlay_state::TerrainOverlayState,
    tile::{Tile, TileLayout},
};
use crate::bella::{
    config::SimulationConfig, environment::Sun, restart::SimulationState, time::TimeUnitPassedEvent,
};
use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
use std::time::Duration;

pub struct ThermalConductorPlugin;

impl Plugin for ThermalConductorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThermalConductor>()
            .add_systems(
                OnEnter(SimulationState::LoadAssets),
                initialize_assets_map_temperature,
            )
            .add_systems(
                Update,
                update_tile_color_for_thermal
                    .run_if(in_state(TerrainOverlayState::Thermal))
                    .run_if(in_state(SimulationState::Simulation)),
            )
            .add_systems(
                Update,
                handle_select_input
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(TerrainOverlayState::Thermal)),
            )
            .add_systems(
                Update,
                accumulate_energy_from_solar.run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
}

#[derive(Component, Reflect, Debug)]
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

    pub fn min_heat(&self) -> f32 {
        self.heat_capacity * ThermalConductor::min_temperature()
    }

    pub fn max_heat(&self) -> f32 {
        self.heat_capacity * ThermalConductor::max_temperature()
    }

    pub fn clamp_heat(&mut self) {
        let _ = self.heat.clamp(self.min_heat(), self.max_heat());
    }

    pub fn get_heat_lose(&self) -> f32 {
        ThermalConductor::default_heat_lose() * self.temperature()
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

    const fn default_heat_lose() -> f32 {
        200.
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ThermalOverlayUpdateTimer(Timer);

pub fn init_thermal_overlay_update_timer(mut cmd: Commands, config: Res<SimulationConfig>) {
    cmd.insert_resource(ThermalOverlayUpdateTimer(Timer::from_seconds(
        config.terrain.thermal_overlay_update_cooldown,
        TimerMode::Repeating,
    )));
}

pub fn update_temperatures(
    mut query: Query<(&Tile, &mut ThermalConductor)>,
    tile_layout: Res<TileLayout>,
) {
    // for (tile, mut thermal_conductor) in query.iter_mut() {
    //     let neighbours: Vec<_> = tile_layout
    //         .get_neighbour_entities(tile.col, tile.row)
    //         .iter()
    //         .flat_map(|&entity| query.get(entity).ok())
    //         .collect();

    // let heat_diff = neighbours
    //     .iter()
    //     .map(|(&tile, &mut thermal_conductor)| {
    //         let temp_diff = media_org.temperature() - media_org[neighbour_i].temperature();
    //         media_org[i].thermal_conductivity * temp_diff
    //     })
    //     .sum();
    // }

    // for x in 0..tile_layout.cols {
    //     for y in 0..tile_layout.rows {
    //         let neighbours: Vec<(&Tile, &mut ThermalConductor)> = tile_layout
    //             .get_neighbour_entities(x, y)
    //             .iter()
    //             .filter_map(|entity| query.get(entity))
    //             .collect();

    //         let heat_diff = neighbours
    //             .iter()
    //             .map(|(&tile, &mut thermal_conductor)| {
    //                 let temp_diff = media_org.temperature() - media_org[neighbour_i].temperature();
    //                 media_org[i].thermal_conductivity * temp_diff
    //             })
    //             .sum();

    //         heat_diffs.push(heat_diff);
    //     }
    // }

    let (tiles, thermal_conductors): (Vec<_>, Vec<_>) = query.iter().unzip();

    // we will set all heat at once to make sure order of iteration doesn't matter and energy is conserved
    let mut heat_diffs: Vec<f32> = Vec::with_capacity(thermal_conductors.capacity());

    for i in 0..thermal_conductors.len() {
        let all_neighbours = tile_layout.get_neighbour_entities(tiles[i].col, tiles[i].row);

        let heat_diff = all_neighbours
            .iter()
            .map(|&neighbour_entity| {
                let (_, neighbour_thermal_conductor) = query
                    .get(neighbour_entity)
                    .expect("This entity doesn't exist (it should)");
                let temp_diff =
                    thermal_conductors[i].temperature() - neighbour_thermal_conductor.temperature();
                thermal_conductors[i].thermal_conductivity * temp_diff
            })
            .sum();

        heat_diffs.push(heat_diff);
    }

    // set all heat at once
    for (i, (_, mut thermal_conductor)) in query.iter_mut().enumerate() {
        thermal_conductor.heat -= heat_diffs[i];
    }
}

fn accumulate_energy_from_solar(mut terrain: Query<&mut ThermalConductor>, sun: Res<Sun>) {
    for mut thermal_conductor in terrain.iter_mut() {
        thermal_conductor.heat += sun.get_energy_part_for_tile();

        thermal_conductor.heat -= thermal_conductor.get_heat_lose();

        thermal_conductor.clamp_heat();
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
    let default_material_high = materials.add(Color::linear_rgb(1., 1., 1.));
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
    mut tiles: Query<(&mut MeshMaterial3d<StandardMaterial>, &ThermalConductor)>,
    assets_map: Res<AssetsMapTemperature>,
    mut timer: ResMut<ThermalOverlayUpdateTimer>,
) {
    if !timer.tick(Duration::from_secs(1)).just_finished() {
        return;
    }

    for (mut mesh_material, medium) in tiles.iter_mut() {
        let temp = medium.heat / medium.heat_capacity;

        mesh_material.0 = if temp < ThermalConductor::min_temperature() {
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

// TODO: this might be useful, but probably needs to be reworked with bevy_picking (maybe some more general solution allowing for changing of anything?)
fn handle_select_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut tiles: Query<(Entity, &mut ThermalConductor)>,
    tile_layout: Res<TileLayout>,
) {
    // let (camera, camera_transform) = camera.into_inner();
    // let Some(cursor_position) = window.into_inner().cursor_position() else {
    //     return;
    // };

    // let Ok(pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
    //     return;
    // };

    // let length = ray.origin.length();

    // if let Some(selected_entity) = tile_layout.get_entity_for_position(pos) {
    //     let (_, mut thermal_conductor) = tiles.get_mut(selected_entity).unwrap();
    //     thermal_conductor.heat = thermal_conductor.max_heat();
    // }
}
