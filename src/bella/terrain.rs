pub mod biome;
pub mod terrain_overlay_state;
pub mod thermal_conductor;

use self::{
    biome::{BiomePlugin, BiomeType},
    thermal_conductor::{
        init_thermal_overlay_update_timer, update_temperatures, ThermalConductor,
        ThermalConductorPlugin,
    },
};
use crate::bella::config::SimConfig;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::hashbrown::HashMap,
};
use hexx::{shapes, Hex, HexLayout};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    HybridMulti, Perlin,
};
use rand::Rng;
use terrain_overlay_state::TerrainOverlayStatePlugin;

use super::{pause::PauseState, restart::SimState, time::HourPassedEvent};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ThermalConductorPlugin,
            BiomePlugin,
            TerrainOverlayStatePlugin,
        ))
        .add_systems(
            OnEnter(SimState::PreSimulation),
            init_thermal_overlay_update_timer, // TODO: do we still need it? Probably just use events
        )
        .add_systems(OnEnter(SimState::TerrainGeneration), generate_terrain)
        .add_systems(OnExit(SimState::Simulation), despawn_terrain);
        // .add_systems(
        //     Update,
        //     update_temperatures
        //         // .run_if(in_state(PauseState::Running))
        //         // .run_if(in_state(SimState::Simulation)),
        //         .run_if(on_event::<HourPassedEvent>),
        // );
    }
}

#[derive(Component)]
pub struct TerrainMarker;

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TerrainPosition {
    pub hex_pos: Hex,
}

#[derive(Resource, Debug)]
pub struct TileMap {
    pub layout: HexLayout,
    pub entities: HashMap<Hex, Entity>,
}

impl TileMap {
    pub fn world_pos_to_entity(&self, position: Vec2) -> Option<Entity> {
        let hex = self
            .layout
            .world_pos_to_hex(hexx::Vec2::new(position.x, position.y));

        self.entities.get(&hex).copied()
    }

    pub fn world_pos_in_entities(&self, position: Vec2) -> bool {
        let hex = self
            .layout
            .world_pos_to_hex(hexx::Vec2::new(position.x, position.y));

        self.entities.contains_key(&hex)
    }
}
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

    let mut rng = rand::thread_rng();
    let noise_map = PlaneMapBuilder::new(HybridMulti::<Perlin>::new(current_time as u32))
        .set_size(
            (config.terrain.map_radius * 2 + 1) as usize,
            (config.terrain.map_radius * 2 + 1) as usize,
        )
        .build();

    // noise_map.write_to_file(std::path::Path::new("test.png"));

    let hex_layout = HexLayout {
        hex_size: hexx::Vec2::splat(config.terrain.hex_size),
        orientation: hexx::HexOrientation::Pointy,
        ..default()
    };

    // let mesh = hexagonal_plane(&hex_layout);
    let mesh = helpers::generate_hex_mesh();
    let mesh_handle = meshes.add(mesh);

    let entities = shapes::hexagon(Hex::ZERO, config.terrain.map_radius)
        // let entities = shapes::flat_rectangle([
        //     -(config.terrain.map_radius as i32),
        //     (config.terrain.map_radius as i32),
        //     -(config.terrain.map_radius as i32),
        //     (config.terrain.map_radius as i32),
        // ])
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

            let mut transform = Transform::from_xyz(pos.x, pos.y, 0.)
                .with_scale(Vec3::splat(config.terrain.hex_size));
            transform.rotate_x(90.0_f32.to_radians());

            let terrain_tile = cmd
                .spawn((
                    TerrainMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(materials.add(Color::linear_rgb(0.9, 0.3, 0.3))),
                    transform,
                    terrain_position,
                    biome,
                    thermal_conductor,
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

fn despawn_terrain(mut cmd: Commands, terrain: Query<Entity, With<TerrainMarker>>) {
    for terrain_entity in terrain.iter() {
        cmd.entity(terrain_entity).despawn_recursive();
    }
}

mod helpers {
    use crate::bella::hex::{
        bevel_hexagon_indices, bevel_hexagon_normals, bevel_hexagon_points, HexCoord,
    };

    use super::*;

    pub fn generate_hex_mesh() -> Mesh {
        // return Cylinder::new(0.7, 0.1).into();

        let mut pts: Vec<[f32; 3]> = vec![];
        let c = HexCoord::new(0, 0);
        bevel_hexagon_points(&mut pts, 1.0, 0.9, &c);

        let mut normals: Vec<[f32; 3]> = vec![];
        bevel_hexagon_normals(&mut normals);

        let mut uvs: Vec<[f32; 2]> = vec![];
        for _ in 0..pts.len() {
            uvs.push([0., 0.]);
        }

        let mut indices = vec![];
        bevel_hexagon_indices(&mut indices);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_indices(Indices::U32(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pts);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh

        // let mut mesh = Mesh::new(
        //     PrimitiveTopology::TriangleList,
        //     RenderAssetUsages::RENDER_WORLD,
        // );

        // let vertexes: Vec<[f32; 3]> = vec![[1.5, 0., 0.], [0., 0., 1.], [0., 0., 0.]];

        // let indices = vec![2, 1, 0];

        // let normals: Vec<[f32; 3]> = [[0., 1., 0.]].repeat(3);

        // let uvs: Vec<[f32; 2]> = (0..vertexes.len()).map(|_| [0., 0.]).collect();

        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertexes);
        // mesh.insert_indices(Indices::U32(indices));
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        // mesh
    }
}
