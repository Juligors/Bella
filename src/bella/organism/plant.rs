use bevy::prelude::*;
use rand::Rng;

use crate::bella::{
    config::SimConfig,
    environment::Sun,
    organism::{EnergyData, Health},
    pause::PauseState,
    restart::SimState,
    terrain::{biome::BiomeType, TerrainPosition, TileMap},
    time::{DayPassedEvent, HourPassedEvent},
};

use super::{ReproductionState, Size};

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimState::LoadAssets), prepare_plant_assets)
            .add_systems(OnEnter(SimState::OrganismGeneration), spawn_plants)
            .add_systems(OnExit(SimState::Simulation), despawn_plants)
            .add_systems(
                Update,
                (
                    produce_energy_from_solar,
                    consume_energy_to_survive,
                    consume_energy_to_grow,
                    consume_energy_to_reproduce,
                    update_plant_color,
                )
                    .chain()
                    .run_if(on_event::<HourPassedEvent>),
                // .run_if(in_state(SimState::Simulation).and(in_state(PauseState::Running))),
            )
            .add_systems(
                Update,
                data_collection::save_plant_data.run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Resource)]
struct PlantAssets {
    alive: Vec<Handle<StandardMaterial>>,
    dead: Handle<StandardMaterial>,
}

fn prepare_plant_assets(mut cmd: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let plant_assets = PlantAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::srgb(0.3, i as f32 / 100., 0.3)))
            .collect(),
        dead: materials.add(Color::srgb(0., 0., 0.)),
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

    let base_size = 3.;
    let mesh_handle = meshes.add(Cuboid::new(base_size, base_size, base_size));

    for (biome_type, terrain_position) in tiles.iter() {
        if !rng.gen_bool(config.plant.group_spawn_chance_grass as f64) {
            continue;
        }

        if *biome_type != BiomeType::Dirt {
            continue;
        }

        let group_middle_pos = tile_map.layout.hex_to_world_pos(terrain_position.hex_pos);
        let plant_count = rng.gen_range(config.plant.group_size_min..=config.plant.group_size_max);

        for _ in 0..plant_count {
            let hp = 100.;
            let size = Size {
                base_size,
                ratio: rng.gen_range(0.5..2.0),
            };
            let energy_data = EnergyData {
                energy: 1000.,
                production_efficiency: 0.01,
                energy_needed_for_survival_per_mass_unit: 5.,
                energy_needed_for_growth_per_mass_unit: 5.,
                grow_by: 0.2,
            };

            // algorithm taken from: https://stackoverflow.com/questions/3239611/generating-random-points-within-a-hexagon-for-procedural-game-content
            let sqrt3 = 3.0f32.sqrt();
            let vectors = [(-1., 0.), (0.5, sqrt3 / 2.), (0.5, -sqrt3 / 2.)];

            let index = rng.gen_range(0..=2);
            let vector_x = vectors[index];
            let vector_y = vectors[(index + 1) % 3];

            let (base_x, base_y) = rng.gen::<(f32, f32)>();
            let offset_x = base_x * vector_x.0 + base_y * vector_y.0;
            let offset_y = base_x * vector_x.1 + base_y * vector_y.1;

            let x = group_middle_pos.x + offset_x * config.terrain.hex_size;
            let y = group_middle_pos.y + offset_y * config.terrain.hex_size;

            cmd.spawn((
                PlantMarker,
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(plant_assets.alive[hp as usize].clone()),
                Transform::from_xyz(x, y, base_size).with_scale(Vec3::splat(size.ratio)),
                Health { hp },
                size,
                energy_data,
                ReproductionState::Developing(config.plant.development_time),
            ));
        }
    }
}

fn despawn_plants(mut cmd: Commands, plants: Query<Entity, With<PlantMarker>>) {
    for plant_entity in plants.iter() {
        cmd.entity(plant_entity).despawn_recursive();
    }
}

fn produce_energy_from_solar(
    mut query: Query<(&mut EnergyData, &Size), With<PlantMarker>>,
    sun: Res<Sun>,
    config: Res<SimConfig>,
) {
    for (mut energy_data, size) in query.iter_mut() {
        let surface_percentage = size.real_surface() / config.terrain.hex_surface();
        let energy_from_sun = sun.get_energy_part(surface_percentage);
        let produced_energy = energy_from_sun * energy_data.production_efficiency;

        energy_data.energy += produced_energy;
    }
}

fn consume_energy_to_survive(mut query: Query<(&mut EnergyData, &Size), With<PlantMarker>>) {
    for (mut energy_data, size) in query.iter_mut() {
        let energy_consumed_to_survive =
            energy_data.energy_needed_for_survival_per_mass_unit * size.real_mass();

        energy_data.energy -= energy_consumed_to_survive;
    }
}

fn consume_energy_to_grow(
    mut query: Query<
        (
            &mut EnergyData,
            &mut Size,
            &mut Transform,
            &mut ReproductionState,
        ),
        With<PlantMarker>,
    >,
) {
    for (mut energy_data, mut size, mut transform, mut reproduction_state) in query.iter_mut() {
        match *reproduction_state {
            ReproductionState::ReadyToReproduce => continue,
            ReproductionState::WaitingToReproduce(_) => continue,
            ReproductionState::Developing(time) => {
                let energy_consumed_to_grow =
                    energy_data.energy_needed_for_growth_per_mass_unit * size.real_mass();

                if energy_data.energy < energy_consumed_to_grow {
                    continue;
                }

                *reproduction_state = ReproductionState::Developing(time - 1);
                energy_data.energy -= energy_consumed_to_grow;
                size.ratio += energy_data.grow_by;

                *transform = transform.with_scale(Vec3::splat(size.ratio));
            }
        }
    }
}

fn consume_energy_to_reproduce(
    mut cmd: Commands,
    mut query: Query<
        (
            &mut ReproductionState,
            &mut EnergyData,
            &mut Health,
            &Transform,
        ),
        With<PlantMarker>,
    >,
    _tile_map: Res<TileMap>,
    config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    plant_assets: Res<PlantAssets>,
) {
    let mut rng = rand::thread_rng();

    let base_size = 3.;
    let mesh_handle = meshes.add(Cuboid::new(base_size, base_size, base_size));

    for (mut life_cycle_state, mut energy_data, mut health, transform) in query.iter_mut() {
        match *life_cycle_state {
            ReproductionState::Developing(_) => continue,
            ReproductionState::WaitingToReproduce(cooldown) => {
                *life_cycle_state = ReproductionState::WaitingToReproduce(cooldown - 1);
            }
            ReproductionState::ReadyToReproduce => {
                *life_cycle_state = ReproductionState::WaitingToReproduce(
                    config.plant.waiting_for_reproduction_time,
                );

                // TODO: for now to make plants smaller and die (why are they not get smaller?)
                let by = 2.0;
                // energy_data.energy /= by;
                // size.ratio /= by;
                // transform.scale /= by;

                health.hp /= by;
                health.hp -= 1.;

                // TODO: this function should work like this:
                // iterate over neighbouring tiles and check if they are suitable for plant
                // get list of them (including current tile)
                // pick 1 of the tiles at random
                // pawn new plant there

                let old_plant_x = transform.translation.x;
                let old_plant_y = transform.translation.y;

                let range = -50.0..50.0;
                let offset_x = rng.gen_range(range.clone());
                let offset_y = rng.gen_range(range);

                let new_plant_x = old_plant_x + offset_x;
                let new_plant_y = old_plant_y + offset_y;

                // TODO: this is copied from base spawn function, should create separate function for creating default plant (and organism as well)
                let hp = 100.;
                let size = Size {
                    base_size,
                    ratio: rng.gen_range(0.5..2.0),
                };
                let energy_data = EnergyData {
                    energy: 1000.,
                    production_efficiency: 0.01,
                    energy_needed_for_survival_per_mass_unit: 5.,
                    energy_needed_for_growth_per_mass_unit: 1.,
                    grow_by: 0.2,
                };

                cmd.spawn((
                    PlantMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(plant_assets.alive[hp as usize].clone()),
                    Transform::from_xyz(new_plant_x, new_plant_y, 1.)
                        .with_scale(Vec3::splat(size.ratio)),
                    Health { hp },
                    size,
                    energy_data,
                    ReproductionState::Developing(config.plant.development_time),
                ));
            }
        }
    }
}

fn update_plant_color(
    mut plants: Query<(&mut MeshMaterial3d<StandardMaterial>, &Health), With<PlantMarker>>,
    assets: Res<PlantAssets>,
) {
    for (mut mesh_material, health) in plants.iter_mut() {
        mesh_material.0 = if health.hp > 0. {
            assets.alive[health.hp as usize].clone()
        } else {
            assets.dead.clone()
        };
    }
}

mod data_collection {
    use super::*;
    use crate::bella::data_collection::DataCollectionDirectory;

    #[derive(Debug, serde::Serialize)]
    pub struct Plant {
        pub id: u64,
        pub health: f32,
        pub base_size: f32,
        pub ratio: f32,

        pub energy: f32,
        pub production_efficiency: f32,
        pub energy_needed_for_survival_per_mass_unit: f32,
        pub energy_needed_for_growth_per_mass_unit: f32,
        pub grow_by: f32,
    }

    pub fn save_plant_data(
        plants: Query<(Entity, &Health, &Size, &EnergyData), With<PlantMarker>>,
        path: Res<DataCollectionDirectory>,
    ) {
        let path = path.0.join("plants.csv");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(path.exists())
            // .truncate(true)
            .create(true)
            .open(path)
            .unwrap();

        let needs_headers = std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0)).unwrap() == 0;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'|')
            // .quote_style(csv::QuoteStyle::Always)
            .has_headers(needs_headers)
            .from_writer(file);

        for x in plants.iter() {
            let plant_record = Plant {
                id: x.0.to_bits(),
                health: x.1.hp,
                base_size: x.2.base_size,
                ratio: x.2.ratio,
                energy: x.3.energy,
                production_efficiency: x.3.production_efficiency,
                energy_needed_for_survival_per_mass_unit: x
                    .3
                    .energy_needed_for_survival_per_mass_unit,
                energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
                grow_by: x.3.grow_by,
            };

            writer
                .serialize(&plant_record)
                .unwrap_or_else(|_| panic!("Couldn't serialize object {:?}", plant_record));
        }

        writer
            .flush()
            .expect("Couldn't save new plant data to a file");
    }
}
