use super::{
    gene::{Allele, AlleleType, Gene, UnsignedFloatGene},
    Age, KillOrganismEvent, OrganismBundle, OrganismEnergyEfficiency, ReadyToReproduceMarker,
    ReproduceEvent, SexualMaturity,
};
use crate::bella::{
    config::SimConfig,
    environment::Sun,
    inspector::choose_entity_observer,
    organism::{EnergyDatav3, Health},
    restart::SimState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::HourPassedEvent,
};
use bevy::prelude::*;

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlantEnergyEfficiency>()
            .register_type::<PollinationRange>()
            .add_systems(OnEnter(SimState::LoadAssets), prepare_plant_assets)
            .add_systems(OnEnter(SimState::OrganismGeneration), spawn_plants)
            .add_systems(OnExit(SimState::Simulation), despawn_plants)
            .add_systems(
                Update,
                (
                    produce_energy_from_solar,
                    send_reproduce_events_if_possible_and_reset_cooldowns_and_consume_energy,
                    reproduce,
                    // give_plant_energy_from_thermal_conductor_its_on,
                )
                    .chain()
                    .run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Bundle)]
pub struct PlantBundle {
    organism_bundle: OrganismBundle,
    marker: PlantMarker,
    plant_energy_efficiency: PlantEnergyEfficiency,
    pollination_range: PollinationRange,
}

#[derive(Resource)]
pub struct PlantAssets {
    alive: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PlantEnergyEfficiency {
    pub production_from_solar_gene: UnsignedFloatGene,
}

impl PlantEnergyEfficiency {
    pub fn new(production_from_solar_gene: impl Into<UnsignedFloatGene>) -> Self {
        Self {
            production_from_solar_gene: production_from_solar_gene.into(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PollinationRange {
    gene: UnsignedFloatGene,
}

impl PollinationRange {
    pub fn new(gene: impl Into<UnsignedFloatGene>) -> Self {
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
            let health = Health::new(config.organism.max_health_gene_config.into());
            let starting_age = config.organism.starting_age_dist.sample();
            let age = Age::new(starting_age, config.organism.age_penalty_gene_config.into());
            let sexual_maturity = SexualMaturity::new(
                config.organism.maturity_age_gene_config.into(),
                config.organism.reproduction_cooldown_gene_config.into(),
                starting_age,
            );
            let energy_data = EnergyDatav3::new(
                config.organism.max_active_energy_gene_config.into(),
                config.organism.max_active_energy_gene_config.into(),
                config.organism.starting_mass_dist.sample(),
            );
            let organism_energy_efficiency = OrganismEnergyEfficiency::new(
                config
                    .organism
                    .energy_to_survive_per_mass_unit_gene_config
                    .into(),
                config.organism.reproduction_energy_cost_gene_config.into(),
            );

            let plant_energy_efficiency = PlantEnergyEfficiency::new(
                config
                    .plant
                    .energy_production_from_solar_efficiency_gene_config,
            );
            let pollination_range =
                PollinationRange::new(config.plant.pollination_range_gene_config);

            // let reproduction_cooldown = ReproductionCooldown(config.plant.development_time);

            let size = energy_data.get_size();
            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn(PlantBundle {
                    organism_bundle: OrganismBundle {
                        mesh: Mesh3d(mesh_handle.clone()),
                        material: MeshMaterial3d(plant_assets.alive.clone()),
                        transform: Transform::from_translation(position.extend(size / 2.0))
                            .with_scale(Vec3::splat(size)),
                        health,
                        age,
                        sexual_maturity,
                        energy_data,
                        organism_energy_efficiency,
                    },
                    marker: PlantMarker,
                    plant_energy_efficiency,
                    pollination_range,
                })
                .id();

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
    mut query: Query<(&mut EnergyDatav3, &PlantEnergyEfficiency), With<PlantMarker>>,
    sun: Res<Sun>,
) {
    for (mut energy_data, energy_efficiency) in query.iter_mut() {
        let produced_energy =
            sun.get_energy_for_plant() * energy_efficiency.production_from_solar_gene.phenotype();
        energy_data.store_energy(produced_energy);
        trace!(
            "Produced {} energy from solar, current energy (after taking active energy): {}",
            produced_energy,
            energy_data.get_stored_energy()
        );
    }
}

fn send_reproduce_events_if_possible_and_reset_cooldowns_and_consume_energy(
    mut reproduction_ew: EventWriter<ReproduceEvent>,
    mut kill_organism_ew: EventWriter<KillOrganismEvent>,
    mut commands: Commands,
    mut plants_ready_to_reproduce: Query<
        (
            Entity,
            &Transform,
            &PollinationRange,
            &mut SexualMaturity,
            &mut EnergyDatav3,
            &OrganismEnergyEfficiency,
        ),
        With<ReadyToReproduceMarker>,
    >,
) {
    let mut combinations = plants_ready_to_reproduce.iter_combinations_mut();

    while let Some(
        [(
            entity1,
            transform1,
            range1,
            mut sexual_maturity1,
            mut energy_data1,
            organism_energy_efficiency1,
        ), (
            entity2,
            transform2,
            range2,
            mut sexual_maturity2,
            mut energy_data2,
            organism_energy_efficiency2,
        )],
    ) = combinations.fetch_next()
    {
        let distance = transform1.translation.distance(transform2.translation);
        if distance <= range1.gene.phenotype() && distance <= range2.gene.phenotype() {
            // TODO: it's a bug fix to ensure one plant get's to have multiple children at the same time
            // (this loop iterates over one plant multiple times, it doesn't know that marker gets removed)
            // honestly at this point marker could be removed, but it helps with O(n^2) complexity
            // could get rid of it and in this function just manually filter only plants ready to reproduce and then iter over their combinations
            if !sexual_maturity1.is_ready_to_reproduce()
                || !sexual_maturity2.is_ready_to_reproduce()
            {
                continue;
            }
            reproduction_ew.send(ReproduceEvent {
                parent1: entity1,
                parent2: entity2,
            });

            // remove marker
            commands.entity(entity1).remove::<ReadyToReproduceMarker>();
            commands.entity(entity2).remove::<ReadyToReproduceMarker>();

            // reset reproduction cooldowns
            sexual_maturity1.reset_reproduction_cooldown();
            sexual_maturity2.reset_reproduction_cooldown();

            // consume energy for reproduction and  kill organisms if they didn't have enough energy
            let energy_needed1 = organism_energy_efficiency1
                .reproduction_energy_cost_gene
                .phenotype();
            if energy_data1.try_to_consume_energy(energy_needed1).is_err() {
                kill_organism_ew.send(KillOrganismEvent { entity: entity1 });
            }
            trace!("Energy consumed for reproduction: {}", energy_needed1);

            let energy_needed2 = organism_energy_efficiency2
                .reproduction_energy_cost_gene
                .phenotype();
            if energy_data2.try_to_consume_energy(energy_needed2).is_err() {
                kill_organism_ew.send(KillOrganismEvent { entity: entity2 });
            }
            trace!("Energy consumed for reproduction: {}", energy_needed2);
        }
    }
}

fn reproduce(
    mut commands: Commands,
    mut event_reader: EventReader<ReproduceEvent>,
    plant_assets: Res<PlantAssets>,
    tile_layout: Res<TileLayout>,
    config: Res<SimConfig>,
    query: Query<(
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &Transform,
        &Health,
        &SexualMaturity,
        &EnergyDatav3,
        &OrganismEnergyEfficiency,
        &PlantEnergyEfficiency,
        &PollinationRange,
        &Age,
    )>,
) {
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for event in event_reader.read() {
        let parent1 = query
            .get(event.parent1)
            .expect("Failed to fetch parent from query during reproduction");
        let parent2 = query
            .get(event.parent2)
            .expect("Failed to fetch parent from query during reproduction");

        // crossing parent organism genes

        let health = Health::new(parent1.3.max_hp_gene.mixed_with(&parent2.3.max_hp_gene));
        let starting_age = config.organism.starting_age_dist.sample();
        let age = Age::new(
            starting_age,
            parent1
                .9
                .age_penalty_gene
                .mixed_with(&parent2.9.age_penalty_gene),
        );
        let sexual_maturity = SexualMaturity::new(
            parent1
                .4
                .maturity_age_gene
                .mixed_with(&parent2.4.maturity_age_gene),
            parent1
                .4
                .reproduction_cooldown_gene
                .mixed_with(&parent2.4.reproduction_cooldown_gene),
            starting_age,
        );
        let energy_data = EnergyDatav3::new(
            parent1
                .5
                .max_active_energy_gene
                .mixed_with(&parent2.5.max_active_energy_gene),
            parent1
                .5
                .energy_per_mass_unit_gene
                .mixed_with(&parent2.5.energy_per_mass_unit_gene),
            config.organism.starting_mass_dist.sample(),
        );
        let organism_energy_efficiency = OrganismEnergyEfficiency::new(
            parent1
                .6
                .energy_consumption_to_survive_per_mass_unit_gene
                .mixed_with(&parent2.6.energy_consumption_to_survive_per_mass_unit_gene),
            parent1
                .6
                .reproduction_energy_cost_gene
                .mixed_with(&parent2.6.reproduction_energy_cost_gene),
        );

        // crossing parent plant genes
        let plant_energy_efficiency = PlantEnergyEfficiency::new(
            parent1
                .7
                .production_from_solar_gene
                .mixed_with(&parent2.7.production_from_solar_gene),
        );
        let pollination_range = PollinationRange::new(parent1.8.gene.mixed_with(&parent2.8.gene));

        // other setup
        let new_plant_position = tile_layout
            .get_random_position_in_ring(
                parent1.2.translation.truncate(),
                config.organism.offspring_spawn_range,
                config.organism.offspring_spawn_range / 2.0,
            )
            .extend(energy_data.get_size() / 2.0);

        let new_size = energy_data.get_size();
        let mut transform =
            Transform::from_translation(new_plant_position).with_scale(Vec3::splat(new_size));
        transform.translation.z = new_size / 2.0;

        let entity = commands
            .spawn(PlantBundle {
                organism_bundle: OrganismBundle {
                    mesh: parent1.0.clone(),
                    material: MeshMaterial3d(plant_assets.alive.clone()),
                    transform,

                    health,
                    age,
                    sexual_maturity,
                    energy_data,
                    organism_energy_efficiency,
                },
                marker: PlantMarker,
                plant_energy_efficiency,
                pollination_range,
            })
            .id();

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
//                 Err(_) => error!("No entity {}, despite getting it from tile_layout", e),
//             },
//             None => {
//                 error!("No tile under this plant :(");
//             }
//         }
//     }
// }
