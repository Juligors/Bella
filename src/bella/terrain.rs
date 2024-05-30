pub mod biome;
pub mod thermal_conductor;

use self::{
    biome::{BiomePlugin, BiomeType},
    thermal_conductor::{
        init_thermal_overlay_update_timer, update_temperatures, ThermalConductor,
        ThermalConductorPlugin,
    },
};
use crate::bella::{config::SimConfig, system_set::InitializationSet, ui::layer::SpriteLayer};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::hashbrown::HashMap,
};
use hexx::{shapes, *};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    HybridMulti, Perlin,
};
use rand::Rng;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ThermalConductorPlugin)
            .add_plugins(BiomePlugin)
            .add_systems(
                Startup,
                (
                    init_thermal_overlay_update_timer.in_set(InitializationSet::ConfigLoad),
                    generate_terrain.in_set(InitializationSet::TerrainGeneration),
                ),
            )
            .add_systems(
                Update,
                update_temperatures,
                // .run_if(on_timer(Duration::from_secs(1))), // TODO: this should be timer
            )
            .register_type::<BiomeType>();
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct TerrainPosition {
    pub hex_pos: Hex,
}

#[derive(Resource)]
pub struct TileMap {
    pub layout: HexLayout,
    pub entities: HashMap<Hex, Entity>,
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
fn generate_terrain(mut cmd: Commands, mut meshes: ResMut<Assets<Mesh>>, config: Res<SimConfig>) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Can't read system time.")
        .as_secs();

    let mut rng = rand::thread_rng();
    let noise_map = PlaneMapBuilder::new(HybridMulti::<Perlin>::new(current_time as u32))
        .set_size(
            (config.terrain.map_radius * 2 + 1) as usize,
            (config.terrain.map_radius * 2 + 1) as usize,
        )
        .build();

    // noise_map.write_to_file(std::path::Path::new("test.png"));

    let hex_layout = HexLayout {
        hex_size: Vec2::splat(config.terrain.hex_size),
        ..default()
    };

    let mesh = hexagonal_plane(&hex_layout);
    let mesh_handle = meshes.add(mesh);

    // let entities = shapes::hexagon(Hex::ZERO, config.map_radius)
    let entities = shapes::flat_rectangle([
        -(config.terrain.map_radius as i32),
        (config.terrain.map_radius as i32),
        -(config.terrain.map_radius as i32),
        (config.terrain.map_radius as i32),
    ])
    .map(|hex| {
        let pos = hex_layout.hex_to_world_pos(hex);
        let terrain_position = TerrainPosition { hex_pos: hex };

        let noise_value = noise_map.get_value(
            (hex.x + config.terrain.map_radius as i32) as usize,
            (hex.y + config.terrain.map_radius as i32) as usize,
        );
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

        let terrain_tile = cmd
            .spawn((
                ColorMesh2dBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.),
                    mesh: mesh_handle.clone().into(),
                    ..default()
                },
                terrain_position,
                biome,
                thermal_conductor,
                SpriteLayer::Terrain,
            ))
            .id();

        (hex, terrain_tile)
    })
    .collect();

    cmd.insert_resource(TileMap {
        layout: hex_layout,
        entities,
    });
}

// --------------------------------------- helpers ---------------------------------------

/// idk what it does, it's probably needed tho.
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
