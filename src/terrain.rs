pub mod biome;
pub mod thermal;
pub mod visualization;

use self::{
    biome::TerrainType,
    thermal::{update_temperatures, ThermalConductor},
};
use crate::{config::SimConfig, layer::SpriteLayer, system_set::InitializationSet};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::hashbrown::HashMap,
};
use hexx::{shapes, *};
use rand::Rng;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app //.add_systems(schedule, systems)
            .add_systems(
                Startup,
                (
                    init_thermal_overlay_update_timer.in_set(InitializationSet::ConfigLoad),
                    setup_grid.in_set(InitializationSet::TerrainGeneration),
                ),
            )
            .add_systems(
                Update,
                update_temperatures,
                // .run_if(on_timer(Duration::from_secs(1))), // TODO: this should be timer
            );
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ThermalOverlayUpdateTimer(Timer);

fn init_thermal_overlay_update_timer(mut cmd: Commands, config: Res<SimConfig>) {
    cmd.insert_resource(ThermalOverlayUpdateTimer(Timer::from_seconds(
        config.thermal_overlay_update_cooldown,
        TimerMode::Repeating,
    )));
}

#[derive(Resource)]
pub struct TileMap {
    layout: HexLayout,
    entities: HashMap<Hex, Entity>,
}

impl TileMap {
    pub fn world_pos_to_entity(&self, position: Vec2) -> Option<Entity> {
        let hex = self.layout.world_pos_to_hex(position);
        self.entities.get(&hex).copied()
    }

    pub fn world_pos_in_entities(&self, position: Vec2) -> bool {
        let hex = self.layout.world_pos_to_hex(position);
        self.entities.contains_key(&hex)
    }
}

fn setup_grid(mut cmd: Commands, mut meshes: ResMut<Assets<Mesh>>, config: Res<SimConfig>) {
    let mut rng = rand::thread_rng();

    let layout = HexLayout {
        hex_size: Vec2::splat(config.hex_size),
        ..default()
    };

    let mesh = hexagonal_plane(&layout);
    let mesh_handle = meshes.add(mesh);

    let entities = shapes::hexagon(Hex::ZERO, config.map_radius)
        // let entities = shapes::flat_rectangle([-5, 5, -5, 4])
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let heat_capacity = ThermalConductor::default_heat_capacity();
            let min_heat = heat_capacity * ThermalConductor::min_temperature();
            let max_heat = heat_capacity * ThermalConductor::max_temperature();
            let heat = rng.gen_range(min_heat..max_heat);
            let k = ThermalConductor::default_thermal_conductivity();

            let medium = ThermalConductor {
                hex_pos: hex,
                heat,
                heat_capacity,
                thermal_conductivity: k,
            };

            let medium_type = match rng.gen_range(1..=4) {
                1 => TerrainType::Stone,
                2 => TerrainType::Dirt,
                3 => TerrainType::Grass,
                _ => TerrainType::Water,
            };

            let entity = cmd
                .spawn((
                    ColorMesh2dBundle {
                        transform: Transform::from_xyz(pos.x, pos.y, 0.),
                        mesh: mesh_handle.clone().into(),
                        ..default()
                    },
                    medium,
                    medium_type,
                    SpriteLayer::Terrain,
                ))
                .id();

            (hex, entity)
        })
        .collect();

    cmd.insert_resource(TileMap { layout, entities });
}

// --------------------------------------- helpers ---------------------------------------

/// idk what it does, it's probably needed tho
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(1.))
        .center_aligned()
        .build();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}
