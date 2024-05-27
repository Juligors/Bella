use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

use crate::bella::{
    config::SimConfig,
    organism::LifeState,
    system_set::InitializationSet,
    terrain::{biome::BiomeType, TerrainPosition, TileMap},
    ui::layer::SpriteLayer,
};

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                prepare_plant_assets,
                spawn_plants.in_set(InitializationSet::PlantSpawning),
            ),
        )
        .add_systems(Update, update_plant_color);
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Resource)]
struct PlantAssets {
    alive: Vec<Handle<ColorMaterial>>,
    dead: Handle<ColorMaterial>,
}

fn prepare_plant_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let plant_assets = PlantAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(0.3, i as f32 / 100., 0.3)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
    };

    cmd.insert_resource(plant_assets);
}

fn spawn_plants(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    plant_assets: Res<PlantAssets>,
    config: Res<SimConfig>,
    tiles: Query<(&BiomeType, &TerrainPosition)>,
    tile_map: Res<TileMap>,
) {
    let mut rng = rand::thread_rng();
    let mesh_handle = Mesh2dHandle(meshes.add(Rectangle::new(3., 3.)));

    for (biome_type, terrain_position) in tiles.iter() {
        if !rng.gen_bool(config.plant_group_spawn_chance_grass as f64) {
            continue;
        }

        if *biome_type != BiomeType::Grass {
            continue;
        }

        let group_middle_pos = tile_map.layout.hex_to_world_pos(terrain_position.hex_pos);
        let plant_count = rng.gen_range(config.plant_group_size_min..config.plant_group_size_max);

        for _ in 0..plant_count {
            let hp = 100.;

            // algorithm taken from: https://stackoverflow.com/questions/3239611/generating-random-points-within-a-hexagon-for-procedural-game-content
            let sqrt3 = 3.0f32.sqrt();
            let vectors = [(-1., 0.), (0.5, sqrt3 / 2.), (0.5, -sqrt3 / 2.)];

            let index = rng.gen_range(0..=2);
            let vector_x = vectors[index];
            let vector_y = vectors[(index + 1) % 3];

            let (base_x, base_y) = rng.gen::<(f32, f32)>();
            let x_offset = base_x * vector_x.0 + base_y * vector_y.0;
            let y_offset = base_x * vector_x.1 + base_y * vector_y.1;

            let x = group_middle_pos.x + x_offset * config.hex_size;
            let y = group_middle_pos.y + y_offset * config.hex_size;

            cmd.spawn((
                PlantMarker,
                MaterialMesh2dBundle {
                    mesh: mesh_handle.clone(),
                    material: plant_assets.alive[hp as usize].clone(),
                    transform: Transform::from_xyz(x, y, 1.),
                    ..default()
                },
                LifeState::Alive { hp },
                SpriteLayer::Plant,
            ));
        }
    }
}

fn update_plant_color(
    mut plants: Query<(&mut Handle<ColorMaterial>, &mut LifeState), With<PlantMarker>>,
    assets: Res<PlantAssets>,
) {
    for (mut handle, mut life_state) in plants.iter_mut() {
        // TODO: remove this
        if let LifeState::Alive { hp } = life_state.as_mut() {
            *hp -= 0.3;
            if *hp <= 0. {
                *life_state = LifeState::Dead;
            }
        }

        match life_state.as_mut() {
            LifeState::Alive { hp } => *handle = assets.alive[*hp as usize].clone(),
            LifeState::Dead => *handle = assets.dead.clone(),
        }
    }
}
