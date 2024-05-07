use std::time::Duration;

use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};

use crate::{
    state::TerrainOverlayState,
    system_set::InitializationSet,
    terrain::{TerrainType, ThermalConductor, TileMap},
};

use super::ThermalOverlayUpdateTimer;

pub struct TileVisualisationPlugin;

// NOTE: for now we leave it like this and just invoke from main
// TODO: this plugin used to be invoked in main. Now this should probably be split to temperature.rs and biome.rs?
// Or just put those functions in terrain.rs plugin? Może popatrz na gry napisane w Bevy
impl Plugin for TileVisualisationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                initialize_assets_map_temperature,
                initialize_assets_map_biomes,
            )
                .in_set(InitializationSet::TerrainVisualization),
        )
        .add_systems(
            Update,
            update_tile_color_for_biome.run_if(in_state(TerrainOverlayState::Bioms)),
        )
        .add_systems(
            Update,
            update_tile_color_for_thermal.run_if(in_state(TerrainOverlayState::Thermal)),
        )
        .add_systems(Update, handle_select_input);
    }
}

#[derive(Resource)]
pub struct AssetsMapTemperature {
    pub medium_materials: HashMap<i32, Handle<ColorMaterial>>,
    pub default_material_low: Handle<ColorMaterial>,
    pub default_material_high: Handle<ColorMaterial>,
    pub selected_material: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct AssetsMapBiomes {
    pub medium_type_materials: HashMap<TerrainType, Handle<ColorMaterial>>,
}

// TODO: this is an old attempt. Maybe something somewhat similar, but with better visualisation/logic separation?
// trait TempHashMap {
//     fn get_value_for(
//         &self,
//         temp: f32,
//         min_temp: f32,
//         max_temp: f32,
//         rate: f32,
//     ) -> Handle<ColorMaterial>;
// }
//
// impl TempHashMap for Map {
//     fn get_value_for(
//         &self,
//         temp: f32,
//         min_temp: f32,
//         max_temp: f32,
//         rate: f32,
//     ) -> Handle<ColorMaterial> {
//         if temp < min_temp {
//             self.default_material_low.clone()
//         } else if temp > max_temp {
//             self.default_material_high.clone()
//         } else {
//             let perc = (temp - min_temp) / (max_temp - min_temp);
//             let one_part = (max_temp - min_temp) / rate;
//             self.default_material_low.clone()
//         }
//     }
// }

fn initialize_assets_map_temperature(
    mut cmd: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let default_material_low = materials.add(Color::BLACK);
    let default_material_high = materials.add(Color::WHITE);
    let selected_material = materials.add(Color::AQUAMARINE);

    let mut medium_materials = HashMap::new();

    // create material for every i32 between min and max temperature
    let mut temp = ThermalConductor::min_temperature();
    while temp <= ThermalConductor::max_temperature() {
        medium_materials.insert(
            temp as i32,
            materials.add(Color::rgb(
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
fn initialize_assets_map_biomes(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let medium_type_materials = HashMap::from([
        (TerrainType::Stone, materials.add(Color::rgb(0.5, 0.5, 0.5))),
        (TerrainType::Dirt, materials.add(Color::rgb(0.6, 0.3, 0.0))),
        (TerrainType::Grass, materials.add(Color::rgb(0.2, 0.7, 0.2))),
        (TerrainType::Water, materials.add(Color::rgb(0.2, 0.4, 0.9))),
    ]);

    cmd.insert_resource(AssetsMapBiomes {
        medium_type_materials,
    });
}

fn update_tile_color_for_thermal(
    mut tiles: Query<(&mut Handle<ColorMaterial>, &ThermalConductor)>,
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

        // *handle = map
        //     .medium_materials
        //     .get(&(temp as &i32))
        //     .unwrap_or(&map.default_material_low)
        //     .clone();

        // let color_handles = map
        //     .medium_materials
        //     .iter()
        //     .filter(|(t, _)| *t == temp)
        //     .map(|(_, h)| h)
        //     .collect::<Vec<_>>();

        // *handle = if color_handles.len() > 0 {
        //     color_handles[0].clone()
        // } else {
        //     map.default_material.clone()
        // }
    }
}

fn update_tile_color_for_biome(
    mut tiles: Query<(&mut Handle<ColorMaterial>, &TerrainType)>,
    assets_map: Res<AssetsMapBiomes>,
) {
    for (mut handle, medium_type) in tiles.iter_mut() {
        *handle = assets_map
            .medium_type_materials
            .get(medium_type)
            .unwrap()
            .clone();
    }
}

// TODO: take a look at that and where it should go. If we don't separate logic anc visualisation then probably just thermal.rs?
fn handle_select_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut tiles: Query<(&mut Handle<ColorMaterial>, &mut ThermalConductor, Entity)>,
    map: Res<TileMap>,
    assets_map: Res<AssetsMapTemperature>,
) {
    let (camera, camera_transform) = cameras.single();
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if let Some(selected_entity) = map.world_pos_to_entity(pos) {
        for (mut handle, mut medium, entity) in tiles.iter_mut() {
            if entity == selected_entity {
                *handle = assets_map.selected_material.clone();
                medium.heat = medium.heat_capacity * ThermalConductor::max_temperature();
            }
        }
    }
}
