pub mod terrain_overlay_state;
pub mod thermal_conductor;
pub mod tile;

use self::thermal_conductor::{
    init_thermal_overlay_update_timer, update_temperatures, ThermalConductor,
    ThermalConductorPlugin,
};
use super::{restart::SimState, time::HourPassedEvent};
use crate::bella::config::SimConfig;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    HybridMulti, Perlin,
};
use rand::Rng;
use terrain_overlay_state::{TerrainOverlayState, TerrainOverlayStatePlugin};
use tile::{Tile, TileLayout};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThermalConductorPlugin, TerrainOverlayStatePlugin))
            .add_systems(OnEnter(SimState::LoadAssets), initialize_assets_map_biomes)
            .add_systems(
                OnEnter(SimState::PreSimulation),
                init_thermal_overlay_update_timer, // TODO: do we still need it? Probably just use events
            )
            .add_systems(OnEnter(SimState::TerrainGeneration), generate_terrain)
            .add_systems(OnExit(SimState::Simulation), despawn_terrain)
            .add_systems(
                Update,
                update_tile_color_for_biome
                    .run_if(in_state(TerrainOverlayState::Bioms))
                    .run_if(in_state(SimState::Simulation)),
            );
        // .add_systems(
        //     Update,
        //     update_temperatures.run_if(on_event::<HourPassedEvent>),
        // );
    }
}

#[derive(Component)]
pub struct TerrainMarker;

fn generate_terrain(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<SimConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Can't read system time.")
        .as_secs();

    let rows_count = config.terrain.map_height;
    let cols_count = config.terrain.map_width;

    let mut rng = rand::thread_rng();

    let noise_map = PlaneMapBuilder::new(HybridMulti::<Perlin>::new(current_time as u32))
        .set_size(cols_count as usize, rows_count as usize)
        .build();

    // noise_map.write_to_file(std::path::Path::new("test.png"));

    let mut tile_layout = TileLayout::new(rows_count, cols_count, config.terrain.tile_size);

    let mesh = tile_layout.generate_mesh();
    let mesh_handle = meshes.add(mesh);

    for row in 0..rows_count {
        tile_layout.add_new_row();

        for col in 0..cols_count {
            let tile = Tile { row, col };
            let tile_position = tile_layout.get_tile_position(&tile);

            let noise_value = noise_map.get_value(col as usize, row as usize);
            let biome = match noise_value {
                x if x < 0.3 => BiomeType::Dirt,
                x if x < 0.6 => BiomeType::Sand,
                x if x < 1.0 => BiomeType::Water,
                _ => BiomeType::Water,
            };

            let heat_capacity = ThermalConductor::default_heat_capacity();
            let min_heat = heat_capacity * ThermalConductor::min_temperature();
            let max_heat = heat_capacity * ThermalConductor::max_temperature();
            let heat = rng.gen_range(min_heat..max_heat);
            let k = ThermalConductor::default_thermal_conductivity();
            let thermal_conductor = ThermalConductor {
                heat,
                heat_capacity,
                thermal_conductivity: k,
            };

            let transform = Transform::from_xyz(tile_position.x, tile_position.y, 0.)
                .with_scale(Vec3::splat(config.terrain.tile_size));

            let entity = cmd
                .spawn((
                    TerrainMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(materials.add(Color::linear_rgb(0.9, 0.3, 0.3))),
                    transform,
                    tile,
                    biome,
                    thermal_conductor,
                ))
                .observe(on_click_do_stuff)
                .id();

            tile_layout.add_new_tile_to_last_row(entity);
        }
    }

    cmd.insert_resource(tile_layout);
}

fn on_click_do_stuff(drag: Trigger<Pointer<Click>>, mut transforms: Query<&mut Transform>) {
    println!("ON CLICK");
    let transform = transforms.get_mut(drag.entity()).unwrap();
    dbg!(transform);
}

fn despawn_terrain(mut cmd: Commands, terrain: Query<Entity, With<TerrainMarker>>) {
    for terrain_entity in terrain.iter() {
        cmd.entity(terrain_entity).despawn_recursive();
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq)]
pub enum BiomeType {
    Stone,
    Sand,
    Dirt,
    Grass,
    Water,
}

#[derive(Resource)]
pub struct AssetsMapBiomes {
    pub medium_type_materials: HashMap<BiomeType, Handle<StandardMaterial>>,
}
fn initialize_assets_map_biomes(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let medium_type_materials = HashMap::from([
        (BiomeType::Stone, materials.add(Color::srgb(0.5, 0.5, 0.5))),
        (BiomeType::Sand, materials.add(Color::srgb(0.9, 0.9, 0.2))),
        (BiomeType::Dirt, materials.add(Color::srgb(0.8, 0.5, 0.2))),
        (BiomeType::Grass, materials.add(Color::srgb(0.4, 0.9, 0.4))),
        (BiomeType::Water, materials.add(Color::srgb(0.2, 0.4, 0.9))),
    ]);

    cmd.insert_resource(AssetsMapBiomes {
        medium_type_materials,
    });
}
fn update_tile_color_for_biome(
    mut tiles: Query<(&mut MeshMaterial3d<StandardMaterial>, &BiomeType)>,
    assets_map: Res<AssetsMapBiomes>,
) {
    for (mut mesh_material, medium_type) in tiles.iter_mut() {
        mesh_material.0 = assets_map
            .medium_type_materials
            .get(medium_type)
            .unwrap()
            .clone();
    }
}
