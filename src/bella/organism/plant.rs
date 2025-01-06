use crate::bella::{
    config::SimConfig,
    environment::Sun,
    inspector::choose_entity_observer,
    organism::{EnergyDatav2, Health},
    restart::SimState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::HourPassedEvent,
};
use bevy::prelude::*;

use super::{ReproductionState, Tissue};

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
                    // give_plant_energy_from_thermal_conductor_its_on,
                    // consume_energy_to_survive,
                    // consume_energy_to_grow,
                    // consume_energy_to_reproduce,
                )
                    .chain()
                    .run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Resource)]
pub struct PlantAssets {
    alive: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Debug)]
pub struct PlantEnergyEfficiency {
    pub production_from_solar: f32,
}

fn prepare_plant_assets(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let plant_assets = PlantAssets {
        alive: materials.add(Color::srgb(0.0, 1.0, 0.0)),
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

        let plant_count = config.plant.group_size_dist.sample();

        for _ in 0..plant_count {
            let max_hp = config.plant.max_health_dist.sample();
            let health = Health {
                hp: max_hp / 2.0,
                max_hp,
            };

            let max_active_energy = config.organism.max_active_energy_dist.sample();
            let energy_data = EnergyDatav2 {
                active_energy: max_active_energy / 2.0,
                max_active_energy,
            };

            let tissue = Tissue {
                mass: config.organism.starting_mass_dist.sample(),
                energy_per_mass_unit: config.organism.energy_per_mass_unit_dist.sample(),
                energy_consumption_per_mass_unit: config
                    .organism
                    .energy_consumption_per_mass_unit_dist
                    .sample(),
            };

            let size = tissue.get_size();

            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn((
                    PlantMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(plant_assets.alive.clone()),
                    Transform::from_translation(position.extend(size))
                        .with_scale(Vec3::splat(size)),
                    ReproductionState::Developing(config.plant.development_time), // TODO: probably need to fix that?
                    health,
                    energy_data,
                    tissue,
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
    mut query: Query<(&mut EnergyDatav2, &PlantEnergyEfficiency), With<PlantMarker>>,
    sun: Res<Sun>,
) {
    for (mut energy_data, energy_efficiency) in query.iter_mut() {
        let produced_energy = sun.get_energy_for_plant() * energy_efficiency.production_from_solar;
        energy_data.active_energy += produced_energy;
        // TODO: should clamp this? or convert overflow directly to fat? Probably...
    }
}

fn consume_energy_to_survive(
    mut query: Query<(&mut EnergyDatav2, &mut Tissue), With<PlantMarker>>,
) {
    for (mut energy_data, mut tissue) in query.iter_mut() {
        let energy_to_survive = tissue.get_energy_consumption();
        energy_data.consume_energy_with_tissue_if_needed(&mut tissue, energy_to_survive);
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
            &mut EnergyDatav2,
            &mut Health,
            &Transform,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Size,
            &PlantMatter,
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
        parent_plant_matter,
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
                    max_hp: parent_health.max_hp,
                    hp: parent_health.max_hp / 2.0,
                };

                let energy_data = EnergyDatav2 {
                    max_active_energy: parent_energy_data.max_active_energy,
                    active_energy: parent_energy_data.max_active_energy / 2.0,
                };

                let plant_matter = PlantMatter {
                    stored_energy: parent_plant_matter.stored_energy,
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
                        plant_matter,
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

// fn give_plant_energy_from_thermal_conductor_its_on(
//     mut plants: Query<(&mut EnergyData, &Transform), With<PlantMarker>>,
//     mut tiles: Query<(Entity, &mut ThermalConductor)>,
//     tile_layout: Res<TileLayout>,
// ) {
//     for (mut energy_data, plant_transform) in plants.iter_mut() {
//         let entity = tile_layout.get_entity_for_position(plant_transform.translation.truncate());

//         match entity {
//             Some(e) => match tiles.get_mut(e) {
//                 Ok((_, mut thermal_conductor)) => {
//                     let energy_taken = 1_000_000. * energy_data.production_efficiency;
//                     if energy_taken < thermal_conductor.heat {
//                         thermal_conductor.heat -= energy_taken;
//                         energy_data.energy += energy_taken;
//                     };
//                 }
//                 Err(_) => error!("No entity {}, despite getting it from tile_layout", e),
//             },
//             None => {
//                 error!("No tile under this plant :(");
//             }
//         }
//     }
// }
