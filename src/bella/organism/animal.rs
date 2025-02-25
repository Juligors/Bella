pub mod gizmos;
pub mod mobile;

use self::mobile::Mobile;
use super::{
    gene::FloatGene, plant::PlantMatterMarker, Age, EnergyDatav3, HungerLevel,
    OrganismBundle, OrganismEnergyEfficiency, ReadyToReproduceMarker, SexualMaturity,
};
use crate::bella::{
    config::SimulationConfig,
    inspector::choose_entity_observer,
    organism::Health,
    pause::PauseState,
    restart::SimulationState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::TimeUnitPassedEvent,
};
use bevy::prelude::*;
use gizmos::AnimalGizmosPlugin;
use itertools::multiunzip;
use mobile::{Destination, MobilePlugin};

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MobilePlugin, AnimalGizmosPlugin))
            .register_type::<Diet>()
            .register_type::<SightRange>()
            .register_type::<Attack>()
            .add_event::<ReproduceAnimalsEvent>()
            .add_systems(OnEnter(SimulationState::LoadAssets), prepare_animal_assets)
            .add_systems(OnEnter(SimulationState::OrganismGeneration), spawn_animals)
            .add_systems(OnExit(SimulationState::Simulation), despawn_all_animals)
            .add_systems(
                Update,
                (
                    choose_new_destination,
                    send_reproduce_events_if_possible_and_reset_cooldowns_and_consume_energy,
                    reproduce,
                    // connect_animal_with_medium_its_on,
                )
                    .chain()
                    .run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct AnimalMarker;

#[derive(Component)]
pub struct AnimalMatterMarker;

#[derive(Bundle)]
pub struct AnimalBundle {
    organism_bundle: OrganismBundle,
    marker: AnimalMarker,
    matter_marker: AnimalMatterMarker,
    animal_energy_efficiency: AnimalEnergyEfficiency,
    reproduction_range: ReproductionRange,
    mobile: Mobile,
    attack: Attack,
    sight_range: SightRange,
    diet: Diet,
}

#[derive(Event)]
pub struct ReproduceAnimalsEvent {
    pub parent1: Entity,
    pub parent2: Entity,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct AnimalEnergyEfficiency {
    // pub energy_consumption_to_survive_per_mass_unit_gene: FloatGene,
}

impl AnimalEnergyEfficiency {
    pub fn new() -> Self {
        Self {
            // production_from_solar_gene: production_from_solar_gene.into(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub enum Diet {
    Carnivorous,
    Herbivorous,
    Omnivore,
}

#[derive(Component, Reflect, Debug, Clone, Deref, DerefMut)]
pub struct SightRange(f32);

#[derive(Component, Reflect, Debug, Clone)]
pub struct Attack {
    pub range: FloatGene,
    pub damage: FloatGene,
}

#[derive(Resource)]
pub struct AnimalAssets {
    pub carnivorous: Handle<StandardMaterial>,
    pub herbivorous: Handle<StandardMaterial>,
    pub omnivore: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct ReproductionRange {
    gene: FloatGene,
}

impl ReproductionRange {
    pub fn new(gene: impl Into<FloatGene>) -> Self {
        Self { gene: gene.into() }
    }
}

pub fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let animal_assets = AnimalAssets {
        carnivorous: materials.add(Color::srgb(1.0, 0.3, 0.3)),
        herbivorous: materials.add(Color::srgb(0.3, 1.0, 0.7)),
        omnivore: materials.add(Color::srgb(0.3, 0.3, 1.0)),
    };

    cmd.insert_resource(animal_assets);
}

fn spawn_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimulationConfig>,
    tiles: Query<(&BiomeType, &Tile)>,
    tile_layout: Res<TileLayout>,
) {
    let mesh_handle = meshes.add(Sphere::new(1.0));
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile) in tiles.iter() {
        if *biome_type != BiomeType::Sand {
            continue;
        }

        if !config.animal.group_spawn_on_sand_chance.happened() {
            continue;
        }

        let animal_count = config.animal.group_size_dist.sample();

        for _ in 0..animal_count {
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
                    .animal
                    .energy_to_survive_per_mass_unit_gene_config
                    .into(),
                config.organism.reproduction_energy_cost_gene_config.into(),
            );

            let diet = match config.animal.diet_dist.sample() {
                0 => Diet::Herbivorous,
                1 => Diet::Carnivorous,
                _ => Diet::Omnivore,
            };
            let animal_energy_efficiency = AnimalEnergyEfficiency::new();
            let reproduction_range =
                ReproductionRange::new(config.plant.pollination_range_gene_config);
            let mobile = Mobile {
                speed: config.animal.speed_dist.sample(),
                destination: None,
                next_step_destination: None,
            };
            let sight_range = SightRange(config.animal.sight_range_dist.sample());
            let attack = Attack {
                range: config.animal.attack_range_dist.sample(),
                damage: config.animal.attack_damage_dist.sample(),
            };

            let size = energy_data.get_size();
            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn(AnimalBundle {
                    organism_bundle: OrganismBundle {
                        mesh: Mesh3d(mesh_handle.clone()),
                        material: MeshMaterial3d(get_animal_asset(&animal_assets, &diet)),
                        transform: Transform::from_translation(position.extend(size / 2.0))
                            .with_scale(Vec3::splat(size)),
                        health,
                        age,
                        sexual_maturity,
                        energy_data,
                        organism_energy_efficiency,
                    },
                    marker: AnimalMarker,
                    matter_marker: AnimalMatterMarker,
                    animal_energy_efficiency,
                    reproduction_range,
                    mobile,
                    attack,
                    sight_range,
                    diet,
                })
                .id();

            choose_entity_observer.watch_entity(entity);
        }
    }

    commands.spawn(choose_entity_observer);
}

fn despawn_all_animals(mut commands: Commands, animals: Query<Entity, With<AnimalMarker>>) {
    for animal_entity in animals.iter() {
        commands.entity(animal_entity).despawn_recursive();
    }
}

pub fn choose_new_destination(
    mut animals: Query<
        (
            Entity,
            &mut Mobile,
            &Transform,
            &EnergyDatav3,
            &SightRange,
            &Diet,
        ),
        With<AnimalMarker>,
    >,
    meat: Query<(Entity, &Transform), With<AnimalMatterMarker>>,
    plant_matter: Query<(Entity, &Transform), With<PlantMatterMarker>>,
    tile_layout: Res<TileLayout>,
) {
    let meat: Vec<_> = meat.iter().collect();
    let plant_matter: Vec<_> = plant_matter.iter().collect();

    let (entities, mut mobiles, transforms, energy_datas, sight_ranges, diets): (
        Vec<Entity>,
        Vec<Mut<Mobile>>,
        Vec<&Transform>,
        Vec<&EnergyDatav3>,
        Vec<&SightRange>,
        Vec<&Diet>,
    ) = multiunzip(animals.iter_mut());

    for i in 0..mobiles.len() {
        if mobiles[i].destination.is_some() {
            continue;
        }

        let mut should_wander_around = false;

        match energy_datas[i].get_hunger_level() {
            HungerLevel::Satiated => should_wander_around = true,
            HungerLevel::Hungry => {
                let distances = match diets[i] {
                    Diet::Carnivorous => {
                        utils::find_closest_meat(entities[i], transforms[i], &meat)
                    }
                    Diet::Herbivorous => {
                        utils::find_closest_plant_matter(transforms[i], &plant_matter)
                    }
                    Diet::Omnivore => {
                        let mut distances_meat =
                            utils::find_closest_meat(entities[i], transforms[i], &meat);
                        let mut distances_plant_matter =
                            utils::find_closest_plant_matter(transforms[i], &plant_matter);

                        distances_meat.append(&mut distances_plant_matter);

                        distances_meat
                    }
                };

                let destination = utils::get_closest_visible(distances, sight_ranges[i]);
                if destination.is_some() {
                    mobiles[i].destination = destination;
                } else {
                    should_wander_around = true;
                }
            }
        }

        if should_wander_around {
            let new_destination_position = tile_layout.get_random_position_in_range(
                transforms[i].translation.truncate(),
                **sight_ranges[i] / 20.0, // TODO: magic number for showcase
            );

            mobiles[i].destination = Some(Destination::Place {
                position: new_destination_position,
            });

            mobiles[i].next_step_destination = Some(new_destination_position);
        }
    }
}

// TODO: it's almost exactly the same as in plants, merge them into organism? (With<ReadyToReproduceMarker>, Or<With<PlantMarker>, With<AnimalMarker>>) or something like that
fn send_reproduce_events_if_possible_and_reset_cooldowns_and_consume_energy(
    mut reproduction_ew: EventWriter<ReproduceAnimalsEvent>,
    mut commands: Commands,
    mut animals_ready_to_reproduce: Query<
        (
            Entity,
            &Transform,
            &ReproductionRange,
            &mut SexualMaturity,
            // &mut EnergyDatav3,
            // &OrganismEnergyEfficiency,
        ),
        (With<ReadyToReproduceMarker>, With<AnimalMarker>),
    >,
) {
    let mut combinations = animals_ready_to_reproduce.iter_combinations_mut();

    while let Some(
        [(
            entity1,
            transform1,
            range1,
            mut sexual_maturity1,
            // mut energy_data1,
            // organism_energy_efficiency1,
        ), (
            entity2,
            transform2,
            range2,
            mut sexual_maturity2,
            // mut energy_data2,
            // organism_energy_efficiency2,
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
            reproduction_ew.send(ReproduceAnimalsEvent {
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
            // let energy_needed1 = organism_energy_efficiency1
            //     .reproduction_energy_cost_gene
            //     .phenotype();
            // if energy_data1.try_to_consume_energy(energy_needed1).is_err() {
            //     kill_organism_ew.send(KillOrganismEvent { entity: entity1 });
            // }
            // println!("Energy consumed for reproduction: {}", energy_needed1);

            // let energy_needed2 = organism_energy_efficiency2
            //     .reproduction_energy_cost_gene
            //     .phenotype();
            // if energy_data2.try_to_consume_energy(energy_needed2).is_err() {
            //     kill_organism_ew.send(KillOrganismEvent { entity: entity2 });
            // }
            // println!("Energy consumed for reproduction: {}", energy_needed2);
        }
    }
}

fn reproduce(
    mut commands: Commands,
    mut event_reader: EventReader<ReproduceAnimalsEvent>,
    animal_assets: Res<AnimalAssets>,
    tile_layout: Res<TileLayout>,
    config: Res<SimulationConfig>,
    query: Query<(
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &Transform,
        &Health,
        &SexualMaturity,
        &EnergyDatav3,
        &OrganismEnergyEfficiency,
        &AnimalEnergyEfficiency,
        &ReproductionRange,
        &Age,
        &Mobile,
        &Attack,
        &SightRange,
        &Diet,
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
        let starting_age = 0;
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
        let animal_energy_efficiency = AnimalEnergyEfficiency::new();
        let reproduction_range = ReproductionRange::new(parent1.8.gene.mixed_with(&parent2.8.gene));
        // TODO: All to genes!
        let mobile = Mobile {
            speed: parent1.10.speed,
            destination: None,
            next_step_destination: None,
        };
        let attack = Attack {
            range: parent1.11.range,
            damage: parent1.11.damage,
        };
        let sight_range = SightRange(parent1.12 .0);
        let diet = parent1.13.clone();

        // other setup
        let middle = (parent1.2.translation + parent2.2.translation) / 2.0;
        let new_plant_position = tile_layout
            .get_random_position_in_ring(
                middle.truncate(),
                config.organism.offspring_spawn_range,
                config.organism.offspring_spawn_range / 2.0,
            )
            .extend(energy_data.get_size() / 2.0);

        let new_size = energy_data.get_size();
        let mut transform =
            Transform::from_translation(new_plant_position).with_scale(Vec3::splat(new_size));
        transform.translation.z = new_size / 2.0;

        let entity = commands
            .spawn(AnimalBundle {
                organism_bundle: OrganismBundle {
                    mesh: parent1.0.clone(),
                    material: MeshMaterial3d(get_animal_asset(&animal_assets, &diet)),
                    transform,

                    health,
                    age,
                    sexual_maturity,
                    energy_data,
                    organism_energy_efficiency,
                },
                marker: AnimalMarker,
                matter_marker: AnimalMatterMarker,
                animal_energy_efficiency,
                reproduction_range,
                mobile,
                attack,
                sight_range,
                diet,
            })
            .id();

        choose_entity_observer.watch_entity(entity);
    }

    commands.spawn(choose_entity_observer);
}

// fn consume_energy_to_reproduce(
//     mut commands: Commands,
//     mut query: Query<
//         (
//             &mut ReproductionState,
//             &mut EnergyData,
//             &mut Health,
//             &Transform,
//             &Diet,
//             &Mesh3d,
//             &MeshMaterial3d<StandardMaterial>,
//             &Size,
//             &Attack,
//             &SightRange,
//             &Mobile,
//         ),
//         With<AnimalMarker>,
//     >,
//     config: Res<SimConfig>,
//     tile_layout: Res<TileLayout>,
// ) {
//     let mut choose_entity_observer = Observer::new(choose_entity_observer);

//     for (
//         mut parent_life_cycle_state,
//         mut parent_energy_data,
//         mut parent_health,
//         parent_transform,
//         parent_diet,
//         parent_mesh,
//         parent_material,
//         parent_size,
//         parent_attack,
//         parent_sight_range,
//         parent_mobile,
//     ) in query.iter_mut()
//     {
//         // TODO: this should happen somewhere else and emit ReproduceEvent with Entity being parent
//         match *parent_life_cycle_state {
//             ReproductionState::Developing(_) => continue,
//             ReproductionState::WaitingToReproduce(cooldown) => {
//                 *parent_life_cycle_state = ReproductionState::WaitingToReproduce(cooldown - 1);
//             }
//             ReproductionState::ReadyToReproduce => {
//                 *parent_life_cycle_state = ReproductionState::WaitingToReproduce(
//                     config.plant.waiting_for_reproduction_time,
//                 );

//                 let size = Size {
//                     size: parent_size.size,
//                 };
//                 let health = Health {
//                     max_hp: parent_health.max_hp,
//                     hp: parent_health.max_hp / 2.0,
//                 };
//                 let mobile = Mobile {
//                     speed: parent_mobile.speed,
//                     destination: None,
//                     next_step_destination: None,
//                 };

//                 let energy_data = EnergyData {
//                     energy: 1000.,
//                     production_efficiency: 0.01,
//                     energy_needed_for_survival_per_mass_unit: 5.,
//                     energy_needed_for_growth_per_mass_unit: 5.,
//                     grow_by: 0.2,
//                 };
//                 let attack = Attack {
//                     damage: parent_attack.damage,
//                     range: parent_attack.range,
//                 };
//                 let sight_range = SightRange(parent_sight_range.0);
//                 let diet = (parent_diet).clone();

//                 let position = tile_layout.get_random_position_in_range(
//                     parent_transform.translation.truncate(),
//                     config.animal.reproduction_range,
//                 );

//                 let entity = commands
//                     .spawn((
//                         AnimalMarker,
//                         Transform::from_translation(position.extend(size.size))
//                             .with_scale(Vec3::splat(size.size)),
//                         HungerLevel::Hungry(100), // FIXME: magic numbetruct
//                         ReproductionState::Developing(config.animal.development_time), // TODO: probably need to fix that?
//                         Meat {
//                             stored_energy: energy_data.energy, // TODO: this shouldn't be duplicated, it's stored enerngy!
//                         },
//                         parent_mesh.clone(),
//                         parent_material.clone(),
//                         attack,
//                         sight_range,
//                         diet,
//                         health,
//                         mobile,
//                         size,
//                         energy_data,
//                     ))
//                     .id();

//                 choose_entity_observer.watch_entity(entity);
//             }
//         }
//     }

//     commands.spawn(choose_entity_observer);
// }

mod utils {
    use super::*;

    // pub fn find_closest_meat(
    //     animal_entities: &[Entity],
    //     animal_transforms: &[&Transform],
    //     i: usize,
    // ) -> Vec<(Entity, f32)> {
    //     animal_entities
    //         .iter()
    //         .zip(animal_transforms)
    //         .enumerate()
    //         .filter(|(j, _)| i != *j)
    //         .map(|(_, (&other_animal_entity, &other_animal_transform))| {
    //             let other_aminal_pos = other_animal_transform.translation.truncate();
    //             let animal_pos = animal_transforms[i].translation.truncate();

    //             (other_animal_entity, animal_pos.distance(other_aminal_pos))
    //         })
    //         .collect()
    // }

    pub fn find_closest_meat(
        animal_entity: Entity,
        animal_transform: &Transform,
        meats: &[(Entity, &Transform)],
    ) -> Vec<(Entity, f32)> {
        meats
            .iter()
            .filter(|(meat_entity, _)| *meat_entity != animal_entity)
            .map(|(meat_entity, &meat_transform)| {
                let meat_pos = meat_transform.translation.truncate();
                let animal_pos = animal_transform.translation.truncate();

                (*meat_entity, animal_pos.distance(meat_pos))
            })
            .collect()
    }

    pub fn find_closest_plant_matter(
        animal_transform: &Transform,
        plant_matters: &[(Entity, &Transform)],
    ) -> Vec<(Entity, f32)> {
        plant_matters
            .iter()
            .map(|(plant_entity, &plant_transform)| {
                let plant_pos = plant_transform.translation.truncate();
                let animal_pos = animal_transform.translation.truncate();

                (*plant_entity, animal_pos.distance(plant_pos))
            })
            .collect()
    }

    pub fn get_closest_visible(
        positions_and_distances: Vec<(Entity, f32)>,
        sight_range: &SightRange,
    ) -> Option<Destination> {
        positions_and_distances
            .iter()
            .filter(|(_, distance)| *distance < **sight_range)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| Destination::Organism { entity: *entity })
    }
}

fn get_animal_asset(assets: &AnimalAssets, diet: &Diet) -> Handle<StandardMaterial> {
    match &diet {
        Diet::Carnivorous => assets.carnivorous.clone(),
        Diet::Herbivorous => assets.herbivorous.clone(),
        Diet::Omnivore => assets.omnivore.clone(),
    }
}

// TODO: this doesn't do much, but this logic should be used later on
// fn connect_animal_with_medium_its_on(
//     creature_transforms: Query<&Transform, With<AnimalMarker>>,
//     tiles: Query<(Entity, &ThermalConductor)>,
//     tile_layout: Res<TileLayout>,
// ) {
//     for creature_transform in creature_transforms.iter() {
//         let creature_pos = creature_transform.translation.truncate();
//         let entity = tile_layout.get_entity_for_position(creature_pos);

//         match entity {
//             Some(e) => {
//                 // for (tile_entity, _medium) in tiles.iter() {
//                 //     if tile_entity != e {
//                 //         continue;
//                 //     }
//                 // }
//                 match tiles.get(e) {
//                     Ok(_) => continue,
//                     Err(_) => println!("No entity {}, despite getting it from tile_layout", e),
//                 }
//             }
//             None => {
//                 println!("No tile under this creature, pos: {}", creature_pos);
//             }
//         }
//     }
// }
