use std::cell::RefCell;

use super::{
    gene::FloatGene, Age, BasicBundle, OrganismBundle, OrganismEnergyEfficiency, SexualMaturity,
};
use crate::bella::{
    config::SimulationConfig,
    environment::Sun,
    organism::{EnergyData, Health},
    restart::SimulationState,
    terrain::{
        tile::{Tile, TileLayout},
        BiomeType, Humidity, Nutrients, ObjectsInTile,
    },
    time::TimeUnitPassedEvent,
    ui_facade::choose_entity_observer,
};
use bevy::prelude::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlantEnergyEfficiency>()
            .register_type::<PollinationRange>()
            .add_event::<ReproducePlantsEvent>()
            .add_systems(OnEnter(SimulationState::LoadAssets), prepare_plant_assets)
            .add_systems(OnEnter(SimulationState::OrganismGeneration), spawn_plants)
            .add_systems(OnExit(SimulationState::Simulation), despawn_plants)
            .add_systems(
                Update,
                (
                    produce_energy_from_solar,
                    send_reproduce_events_if_possible,
                    reproduce,
                    // give_plant_energy_from_thermal_conductor_its_on,
                )
                    .chain()
                    .run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Component)]
pub struct PlantMatterMarker;

#[derive(Bundle)]
pub struct PlantBundle {
    organism_bundle: OrganismBundle,
    marker: PlantMarker,
    matter_marker: PlantMatterMarker,
    plant_energy_efficiency: PlantEnergyEfficiency,
    pollination_range: PollinationRange,
}

#[derive(Event)]
pub struct ReproducePlantsEvent {
    pub parent1: Entity,
    pub parent2: Entity,
}

#[derive(Resource)]
pub struct PlantAssets {
    alive: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PlantEnergyEfficiency {
    pub production_from_solar_gene: FloatGene,
    pub nutrient_consumption: FloatGene,
}

impl PlantEnergyEfficiency {
    pub fn new(
        production_from_solar_gene: impl Into<FloatGene>,
        nutrient_consumption_gene: impl Into<FloatGene>,
    ) -> Self {
        Self {
            production_from_solar_gene: production_from_solar_gene.into(),
            nutrient_consumption: nutrient_consumption_gene.into(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PollinationRange {
    gene: FloatGene,
}

impl PollinationRange {
    pub fn new(gene: impl Into<FloatGene>) -> Self {
        Self { gene: gene.into() }
    }
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
    config: Res<SimulationConfig>,
    mut tiles: Query<(&BiomeType, &Tile, &mut ObjectsInTile)>,
    tile_layout: Res<TileLayout>,
) {
    let mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile, mut objects_in_tile) in tiles.iter_mut() {
        if !biome_type.plants_can_live_here() {
            continue;
        }

        if !config.plant.group_spawn_on_grass_chance.happened() {
            continue;
        }

        let plant_count = config.plant.group_size_dist.sample();

        for _ in 0..plant_count {
            let health = Health::new(config.organism.max_health_gene_config.into());
            let starting_age = config.organism.starting_age_dist.sample();
            let age = Age::new(starting_age, config.organism.age_penalty_gene_config.into());
            let sexual_maturity = SexualMaturity::new(
                config.plant.maturity_age_gene_config.into(),
                config.plant.reproduction_cooldown_gene_config.into(),
                starting_age,
            );
            let energy_data = EnergyData::new(
                config.organism.max_active_energy_gene_config.into(),
                config.organism.max_active_energy_gene_config.into(),
                config.organism.starting_mass_dist.sample(),
            );
            let organism_energy_efficiency = OrganismEnergyEfficiency::new(
                config
                    .plant
                    .energy_to_survive_per_mass_unit_gene_config
                    .into(),
                config.organism.reproduction_energy_cost_gene_config.into(),
            );

            let plant_energy_efficiency = PlantEnergyEfficiency::new(
                config
                    .plant
                    .energy_production_from_solar_efficiency_gene_config,
                config.plant.nutrient_consumption_gene_config,
            );
            let pollination_range =
                PollinationRange::new(config.plant.pollination_range_gene_config);

            let size = energy_data.get_size();
            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn((
                    BasicBundle {
                        mesh: Mesh3d(mesh_handle.clone()),
                        material: MeshMaterial3d(plant_assets.alive.clone()),
                        transform: Transform::from_translation(position.extend(size / 2.0))
                            .with_scale(Vec3::splat(size)),
                    },
                    PlantBundle {
                        organism_bundle: OrganismBundle {
                            health,
                            age,
                            sexual_maturity,
                            energy_data,
                            organism_energy_efficiency,
                        },
                        marker: PlantMarker,
                        matter_marker: PlantMatterMarker,
                        plant_energy_efficiency,
                        pollination_range,
                    },
                ))
                .id();

            objects_in_tile.add_plant_entity(entity);
            choose_entity_observer.watch_entity(entity);
        }
    }

    commands.spawn(choose_entity_observer);
}

fn despawn_plants(mut commands: Commands, plants: Query<Entity, With<PlantMarker>>) {
    for plant_entity in plants.iter() {
        commands.entity(plant_entity).despawn_recursive();
    }
}

fn produce_energy_from_solar(
    mut query: Query<(&mut EnergyData, &PlantEnergyEfficiency, &Transform), With<PlantMarker>>,
    mut nutrients_query: Query<&mut Nutrients>,
    humidity_query: Query<&Humidity>,
    tile_layout: Res<TileLayout>,
    sun: Res<Sun>,
) {
    for (mut energy_data, energy_efficiency, transform) in query.iter_mut() {
        let tile_entity = tile_layout.get_tile_entity_for_transform(transform);

        let mut tile_nutrients = nutrients_query
            .get_mut(tile_entity)
            .expect("Failed to get tile's nutrients from query!");
        let nutrients_value = tile_nutrients
            .take_part_of_nutrients(energy_efficiency.nutrient_consumption.phenotype());

        let tile_humidity = humidity_query
            .get(tile_entity)
            .expect("Failed to get tile's humidity from query!");
        let humidity_value = tile_humidity.value;

        let produced_energy = sun.get_energy_for_plant()
            * energy_efficiency.production_from_solar_gene.phenotype()
            * nutrients_value
            * humidity_value;

        energy_data.store_energy(produced_energy);
    }
}

fn send_reproduce_events_if_possible(
    mut reproduction_ew: EventWriter<ReproducePlantsEvent>,
    objects_in_tile_query: Query<&ObjectsInTile>,
    tile_layout: Res<TileLayout>,
    mut plants_query: Query<
        (Entity, &Transform, &PollinationRange, &mut SexualMaturity),
        With<PlantMarker>,
    >,
) {
    let mut plants_that_will_reproduce = Vec::new();

    for (plant_entity, plant_transform, pollination_range, sexual_maturity) in
        plants_query.iter()
    {
        if !sexual_maturity.is_ready_to_reproduce() {
            continue;
        }

        let chosen_partner_entity = tile_layout
            .get_tile_entities_in_range(
                plant_transform.translation.truncate(),
                pollination_range.gene.phenotype(),
            )
            .iter()
            .flat_map(|tile_entity| {
                objects_in_tile_query
                    .get(*tile_entity)
                    .expect("Failed to get tile")
                    .plants
                    .clone()
            })
            .filter_map(|entity| {
                let (entity, transform, _, sexual_maturity) = plants_query.get(entity).expect(
                    "Failed to get plant entity despite that entity being in ObjectsInTile",
                );
                if sexual_maturity.is_ready_to_reproduce() {
                    let distance = plant_transform.translation.distance(transform.translation);

                    Some((entity, distance))
                } else {
                    None
                }
            })
            .max_by(|(_, distance1), (_, distance2)| distance1.total_cmp(distance2))
            .map(|(entity, _)| entity);

        if let Some(partner_entity) = chosen_partner_entity {
            reproduction_ew.send(ReproducePlantsEvent {
                parent1: plant_entity,
                parent2: partner_entity,
            });
            plants_that_will_reproduce.push(plant_entity);
            plants_that_will_reproduce.push(partner_entity);
        };
    }

    plants_that_will_reproduce
        .into_iter()
        .for_each(|plant_entity| {
            if let Ok((_, _, _, mut sexual_maturity)) = plants_query.get_mut(plant_entity) {
                sexual_maturity.reset_reproduction_cooldown();
            }
        });
}

fn reproduce(
    mut commands: Commands,
    mut event_reader: EventReader<ReproducePlantsEvent>,
    plant_assets: Res<PlantAssets>,
    tile_layout: Res<TileLayout>,
    config: Res<SimulationConfig>,
    query: Query<(
        &Mesh3d,
        &Transform,
        &Health,
        &SexualMaturity,
        &EnergyData,
        &OrganismEnergyEfficiency,
        &PlantEnergyEfficiency,
        &PollinationRange,
        &Age,
    )>,
    mut tiles: Query<(&BiomeType, &mut ObjectsInTile)>,
) {
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for event in event_reader.read() {
        let Ok((
            mesh1,
            transform1,
            health1,
            sexual_maturity1,
            energy_data1,
            organism_energy_efficiency1,
            plant_energy_efficiency1,
            pollination_range1,
            age1,
        )) = query.get(event.parent1)
        else {
            continue;
        };
        let Ok((
            _,
            transform2,
            health2,
            sexual_maturity2,
            energy_data2,
            organism_energy_efficiency2,
            plant_energy_efficiency2,
            pollination_range2,
            age2,
        )) = query.get(event.parent2)
        else {
            continue;
        };

        // crossing parent organism genes
        let health = Health::new(health1.max_hp_gene.mixed_with(&health2.max_hp_gene));
        let starting_age = config.organism.starting_age_dist.sample();
        let age = Age::new(
            starting_age,
            age1.age_penalty_gene.mixed_with(&age2.age_penalty_gene),
        );
        let sexual_maturity = SexualMaturity::new(
            sexual_maturity1
                .maturity_age_gene
                .mixed_with(&sexual_maturity2.maturity_age_gene),
            sexual_maturity1
                .reproduction_cooldown_gene
                .mixed_with(&sexual_maturity2.reproduction_cooldown_gene),
            starting_age,
        );
        let energy_data = EnergyData::new(
            energy_data1
                .max_active_energy_gene
                .mixed_with(&energy_data2.max_active_energy_gene),
            energy_data1
                .energy_per_mass_unit_gene
                .mixed_with(&energy_data2.energy_per_mass_unit_gene),
            config.organism.starting_mass_dist.sample(),
        );
        let organism_energy_efficiency = OrganismEnergyEfficiency::new(
            organism_energy_efficiency1
                .energy_consumption_to_survive_per_mass_unit_gene
                .mixed_with(
                    &organism_energy_efficiency2.energy_consumption_to_survive_per_mass_unit_gene,
                ),
            organism_energy_efficiency1
                .reproduction_energy_cost_gene
                .mixed_with(&organism_energy_efficiency2.reproduction_energy_cost_gene),
        );

        // crossing parent plant genes
        let plant_energy_efficiency = PlantEnergyEfficiency::new(
            plant_energy_efficiency1
                .production_from_solar_gene
                .mixed_with(&plant_energy_efficiency2.production_from_solar_gene),
            plant_energy_efficiency1
                .nutrient_consumption
                .mixed_with(&plant_energy_efficiency2.nutrient_consumption),
        );
        let pollination_range =
            PollinationRange::new(pollination_range1.gene.mixed_with(&pollination_range2.gene));

        // other setup
        let point = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            if rng.gen_bool(0.5) {
                transform1.translation.truncate()
            } else {
                transform2.translation.truncate()
            }
        });

        let (new_plant_position, mut objects_in_tile) = loop {
            let new_plant_position = tile_layout.get_random_position_in_ring(
                point,
                config.organism.offspring_spawn_range,
                config.organism.offspring_spawn_range / 2.0,
            );

            let entity_of_tile_under = tile_layout.get_tile_entity_for_position(new_plant_position);

            let (biome_under_new_plant, objects_in_tile) = tiles
                .get_mut(entity_of_tile_under)
                .expect("Failed to get tile components of tile for new plant position");

            if biome_under_new_plant.plants_can_live_here() {
                break (new_plant_position, objects_in_tile);
            }
        };

        let new_size = energy_data.get_size();
        let mut transform =
            Transform::from_translation(new_plant_position.extend(energy_data.get_size() / 2.0))
                .with_scale(Vec3::splat(new_size));
        transform.translation.z = new_size / 2.0;

        let entity = commands
            .spawn((
                BasicBundle {
                    mesh: mesh1.clone(),
                    material: MeshMaterial3d(plant_assets.alive.clone()),
                    transform,
                },
                PlantBundle {
                    organism_bundle: OrganismBundle {
                        health,
                        age,
                        sexual_maturity,
                        energy_data,
                        organism_energy_efficiency,
                    },
                    marker: PlantMarker,
                    matter_marker: PlantMatterMarker,
                    plant_energy_efficiency,
                    pollination_range,
                },
            ))
            .id();
        objects_in_tile.add_plant_entity(entity);
        choose_entity_observer.watch_entity(entity);
    }

    commands.spawn(choose_entity_observer);
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
//                 Err(_) => println!("No entity {}, despite getting it from tile_layout", e),
//             },
//             None => {
//                 println!("No tile under this plant :(");
//             }
//         }
//     }
// }
