use crate::bella::{
    config::SimConfig,
    environment::Sun,
    inspector::choose_entity_observer,
    organism::{EnergyData, Health},
    pause::PauseState,
    restart::SimState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::{DayPassedEvent, HourPassedEvent},
};
use bevy::prelude::*;

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
                    // produce_energy_from_solar,
                    give_plant_energy_from_thermal_conductor_its_on,
                    consume_energy_to_survive,
                    consume_energy_to_grow,
                    consume_energy_to_reproduce,
                )
                    .chain()
                    .run_if(on_event::<HourPassedEvent>),
                // .run_if(in_state(SimState::Simulation).and(in_state(PauseState::Running))),
            )
            .add_systems(
                Update,
                (data_collection::save_plant_data,).run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Resource)]
pub struct PlantAssets {
    alive: Handle<StandardMaterial>,
    dead: Handle<StandardMaterial>,
}

fn prepare_plant_assets(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let plant_assets = PlantAssets {
        alive: materials.add(Color::srgb(0.0, 1.0, 0.0)),
        dead: materials.add(Color::srgb(0.0, 0.0, 0.0)),
    };

    commands.insert_resource(plant_assets);
}

fn spawn_plants(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    plant_assets: Res<PlantAssets>,
    config: Res<SimConfig>,
    tiles: Query<(&BiomeType, &Tile)>,
    tile_layout: Res<TileLayout>,
) {
    let mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile) in tiles.iter() {
        if *biome_type != BiomeType::Dirt {
            continue;
        }

        if !config.plant.group_spawn_on_grass_chance.happened() {
            continue;
        }

        let size = config.plant.size_dist.sample();
        let plant_count = config.plant.group_size_dist.sample();

        for _ in 0..plant_count {
            let max_hp = config.plant.max_health_dist.sample();
            let health = Health {
                max: max_hp,
                hp: max_hp / 2.0,
            };

            let energy_data = EnergyData {
                energy: 1000.,
                production_efficiency: 0.01,
                energy_needed_for_survival_per_mass_unit: 5.,
                energy_needed_for_growth_per_mass_unit: 5.,
                grow_by: 0.2,
            };

            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn((
                    PlantMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(plant_assets.alive.clone()),
                    Transform::from_translation(position.extend(size))
                        .with_scale(Vec3::splat(size)),
                    Size { size },
                    ReproductionState::Developing(config.plant.development_time), // TODO: probably need to fix that?
                    energy_data,
                    health,
                ))
                .id();

            choose_entity_observer.watch_entity(entity);
        }
    }

    commands.spawn(choose_entity_observer);
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
        let surface_percentage = size.real_surface() / config.terrain.tile_size.powi(2);
        let energy_from_sun = sun.get_energy_part(surface_percentage);
        let produced_energy = energy_from_sun * energy_data.production_efficiency;

        energy_data.energy += produced_energy;
    }
}

fn consume_energy_to_survive(mut query: Query<(&mut EnergyData, &Size), With<PlantMarker>>) {
    for (mut energy_data, size) in query.iter_mut() {
        let energy_consumed_to_survive =
            energy_data.energy_needed_for_survival_per_mass_unit * size.real_volume();

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
                    energy_data.energy_needed_for_growth_per_mass_unit * size.real_volume();

                if energy_data.energy < energy_consumed_to_grow {
                    continue;
                }

                *reproduction_state = ReproductionState::Developing(time - 1);
                energy_data.energy -= energy_consumed_to_grow;
                size.size += energy_data.grow_by;

                *transform = transform.with_scale(Vec3::splat(size.size));
            }
        }
    }
}

fn consume_energy_to_reproduce(
    mut commands: Commands,
    mut query: Query<
        (
            &mut ReproductionState,
            &mut EnergyData,
            &mut Health,
            &Transform,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Size,
        ),
        With<PlantMarker>,
    >,
    tile_layout: Res<TileLayout>,
    config: Res<SimConfig>,
) {
    // TODO: more data should be inherited from parent/parents
    for (
        mut parent_life_cycle_state,
        mut parent_energy_data,
        mut parent_health,
        parent_transform,
        parent_mesh,
        parent_material,
        parent_size,
    ) in query.iter_mut()
    {
        // TODO: this should happen somewhere else and emit ReproduceEvent with Entity being parent
        match *parent_life_cycle_state {
            ReproductionState::Developing(_) => continue,
            ReproductionState::WaitingToReproduce(cooldown) => {
                *parent_life_cycle_state = ReproductionState::WaitingToReproduce(cooldown - 1);
            }
            ReproductionState::ReadyToReproduce => {
                *parent_life_cycle_state = ReproductionState::WaitingToReproduce(
                    config.plant.waiting_for_reproduction_time,
                );

                let size = Size {
                    size: parent_size.size,
                };
                let health = Health {
                    max: parent_health.max,
                    hp: parent_health.max / 2.0,
                };

                let energy_data = EnergyData {
                    energy: 1000.,
                    production_efficiency: 0.01,
                    energy_needed_for_survival_per_mass_unit: 5.,
                    energy_needed_for_growth_per_mass_unit: 5.,
                    grow_by: 0.2,
                };

                let position = tile_layout.get_random_position_in_range(
                    parent_transform.translation.truncate(),
                    config.plant.reproduction_range,
                );

                commands
                    .spawn((
                        PlantMarker,
                        Transform::from_translation(position.extend(size.size))
                            .with_scale(Vec3::splat(size.size)),
                        ReproductionState::Developing(config.plant.development_time), // TODO: probably need to fix that?
                        parent_mesh.clone(),
                        parent_material.clone(),
                        health,
                        size,
                        energy_data,
                    ))
                    .observe(choose_entity_observer);
            }
        }
    }
}

fn give_plant_energy_from_thermal_conductor_its_on(
    mut plants: Query<(&mut EnergyData, &Transform), With<PlantMarker>>,
    mut tiles: Query<(Entity, &mut ThermalConductor)>,
    tile_layout: Res<TileLayout>,
) {
    for (mut energy_data, plant_transform) in plants.iter_mut() {
        let entity = tile_layout.get_entity_for_position(plant_transform.translation.truncate());

        match entity {
            Some(e) => {
                for (tile_entity, mut thermal_conductor) in tiles.iter_mut() {
                    if tile_entity != e {
                        continue;
                    }

                    let energy_taken = 1_000_000. * energy_data.production_efficiency;
                    if energy_taken < thermal_conductor.heat {
                        thermal_conductor.heat -= energy_taken;
                        energy_data.energy += energy_taken;
                    }
                }
            }
            None => {
                error!("No tile under this plant :(");
            }
        }
    }
}

mod data_collection {
    use super::*;
    use crate::bella::{data_collection::DataCollectionDirectory, time::SimTime};

    #[derive(Debug, serde::Serialize)]
    pub struct Plant {
        pub id: u64,
        pub hour: u32,
        pub day: u32,

        pub health: f32,
        pub size: f32,

        pub energy: f32,
        pub production_efficiency: f32,
        pub energy_needed_for_survival_per_mass_unit: f32,
        pub energy_needed_for_growth_per_mass_unit: f32,
        pub grow_by: f32,
    }

    pub fn save_plant_data(
        plants: Query<(Entity, &Health, &Size, &EnergyData), With<PlantMarker>>,
        path: Res<DataCollectionDirectory>,
        time: Res<SimTime>,
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
                hour: time.hours,
                day: time.days,

                health: x.1.hp,
                size: x.2.size,
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
