pub mod gizmos;
pub mod mobile;

use std::cell::RefCell;

use self::mobile::Mobile;
use super::{
    gene::{FloatGene, IntGene},
    plant::PlantMatterMarker,
    Age, EnergyDatav3, HungerLevel, OrganismBundle, OrganismEnergyEfficiency,
    ReadyToReproduceMarker, SexualMaturity,
};
use crate::bella::{
    config::SimulationConfig,
    organism::Health,
    pause::PauseState,
    restart::SimulationState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::TimeUnitPassedEvent, ui_facade::choose_entity_observer,
};
use bevy::prelude::*;
use gizmos::AnimalGizmosPlugin;
use itertools::multiunzip;
use mobile::{Destination, MobilePlugin};
use rand::{rngs::ThreadRng, thread_rng, Rng};

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MobilePlugin))
            .register_type::<Diet>()
            .register_type::<SightRange>()
            .register_type::<AttackDmg>()
            .add_event::<ReproduceAnimalsEvent>()
            .add_systems(OnEnter(SimulationState::LoadAssets), prepare_animal_assets)
            .add_systems(OnEnter(SimulationState::OrganismGeneration), spawn_animals)
            .add_systems(OnExit(SimulationState::Simulation), despawn_all_animals)
            .add_systems(
                Update,
                (
                    // choose_new_destination,
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
    action_range: ActionRange,
    mobile: Mobile,
    attack: AttackDmg,
    sight_range: SightRange,
    diet: Diet,
}

pub enum Action {
    /// NOTE: or call it Resting? Sleeping? It should probably cost less energy
    DoingNothing,
    GoingTo { position: Vec2 },
    Eating { food: Entity },
    Attacking { enemy: Entity },
    Mating { with: Entity },
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
    Carnivore,
    Herbivore,
    Omnivore,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct SightRange {
    gene: FloatGene,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct ActionRange {
    pub gene: FloatGene,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct AttackDmg {
    pub gene: FloatGene,
}

#[derive(Resource)]
pub struct AnimalAssets {
    pub carnivore: Handle<StandardMaterial>,
    pub herbivore: Handle<StandardMaterial>,
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
        carnivore: materials.add(Color::srgb(1.0, 0.3, 0.3)),
        herbivore: materials.add(Color::srgb(0.3, 1.0, 0.7)),
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
        if !biome_type.animals_can_live_here() {
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
                0 => Diet::Herbivore,
                1 => Diet::Carnivore,
                _ => Diet::Omnivore,
            };
            let animal_energy_efficiency = AnimalEnergyEfficiency::new();
            let mobile = Mobile {
                speed: config.animal.speed_gene_config.into(),
                destination: None,
                next_step_destination: None,
            };
            let sight_range = SightRange {
                gene: config.animal.sight_range_gene_config.into(),
            };
            let action_range = ActionRange {
                gene: config.animal.action_range_gene_config.into(),
            };
            let attack = AttackDmg {
                gene: config.animal.attack_damage_gene_config.into(),
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
                    action_range,
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
                    Diet::Carnivore => utils::find_closest_meat(entities[i], transforms[i], &meat),
                    Diet::Herbivore => {
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
                sight_ranges[i].gene.phenotype(),
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
        &ActionRange,
        &SightRange,
        &AttackDmg,
        &Diet,
    )>,
    biomes: Query<&BiomeType>,
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

        // crossing parent animal genes
        let animal_energy_efficiency = AnimalEnergyEfficiency::new();
        let reproduction_range = ReproductionRange::new(parent1.8.gene.mixed_with(&parent2.8.gene));
        let mobile = Mobile {
            speed: parent1.10.speed.mixed_with(&parent2.10.speed),
            destination: None,
            next_step_destination: None,
        };
        let action_range = ActionRange {
            gene: parent1.11.gene.mixed_with(&parent2.11.gene),
        };
        let sight_range = SightRange {
            gene: parent1.12.gene.mixed_with(&parent2.12.gene),
        };
        let attack = AttackDmg {
            gene: parent1.13.gene.mixed_with(&parent2.13.gene),
        };
        // TODO: diet should also be a gene
        let diet = parent1.14.clone();

        // other setup
        let point = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            if rng.gen_bool(0.5) {
                parent1.2.translation.truncate()
            } else {
                parent2.2.translation.truncate()
            }
        });
        let mut new_animal_position = tile_layout.get_random_position_in_ring(
            point,
            config.organism.offspring_spawn_range,
            4.0, // NOTE: magic number to make sure plants don't spawn on top of each other
        );
        loop {
            let entity_of_tile_under = tile_layout
                .get_entity_for_position(new_animal_position)
                .expect("Failed to get tile for new animal position");
            let biome_under_new_animal = biomes
                .get(entity_of_tile_under)
                .expect("Failed to get biome of tile for new animal position");
            if *biome_under_new_animal == BiomeType::Water {
                new_animal_position = tile_layout.get_random_position_in_ring(
                    point,
                    config.organism.offspring_spawn_range,
                    4.0, // NOTE: magic number to make sure animals don't spawn on top of each other
                );
            } else {
                break;
            }
        }

        let new_size = energy_data.get_size();
        let mut transform =
            Transform::from_translation(new_animal_position.extend(energy_data.get_size() / 2.0))
                .with_scale(Vec3::splat(new_size));
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
                mobile,
                action_range,
                sight_range,
                attack,
                diet,
            })
            .id();

        choose_entity_observer.watch_entity(entity);
    }

    commands.spawn(choose_entity_observer);
}

/// TODO: this should be called every hour and when previous action ends
fn choose_next_action(query: Query<()>){
    // Order:
    // Fear > Hunger (carcass/plant) > Hunger (attack) > LookingForMate > Bored (wander around) > Bored (wait?)
    
    // if is_hungry() -> 

}


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
            .filter(|(_, distance)| *distance < sight_range.gene.phenotype())
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| Destination::Organism { entity: *entity })
    }
}

fn get_animal_asset(assets: &AnimalAssets, diet: &Diet) -> Handle<StandardMaterial> {
    match &diet {
        Diet::Carnivore => assets.carnivore.clone(),
        Diet::Herbivore => assets.herbivore.clone(),
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
