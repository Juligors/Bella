use super::{ActionRange, AnimalMarker, AttackDmg, Diet, ReproduceAnimalsEvent, SightRange};
use crate::bella::{
    config::SimulationConfig,
    organism::{
        carcass::Carcass, gene::FloatGene, plant::PlantMarker, EnergyData, Health, HungerLevel,
        SexualMaturity,
    },
    pause::PauseState,
    restart::SimulationState,
    terrain::{tile::TileLayout, BiomeType, ObjectsInTile},
    time::TimeUnitPassedEvent,
};
use bevy::prelude::*;

pub struct AnimalAiPlugin;

impl Plugin for AnimalAiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Mobile>()
            .register_type::<Destination>()
            .register_type::<Action>()
            .add_event::<MakeDecisionEvent>()
            .add_systems(
                Update,
                (
                    (find_next_step_destination, make_step).chain(),
                    // attack,
                    // eat_matter,
                )
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(
                Update,
                send_make_decision_event
                    .run_if(on_event::<TimeUnitPassedEvent>)
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(
                Update,
                (apply_deferred, discover_animal_state_and_set_action)
                    .chain()
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(Update, handle_action);
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub enum Action {
    /// NOTE: or call it Resting? Sleeping? It should probably cost less energy
    DoingNothing {
        for_hours: u32,
    },
    GoingTo {
        position: Vec2,
    },
    Eating {
        food: Entity,
    },
    Attacking {
        enemy: Entity,
    },
    Mating {
        with: Entity,
    },
}

#[derive(Event)]
pub struct MakeDecisionEvent {
    animal_entity: Entity,
}

fn send_make_decision_event(
    mut event_writer: EventWriter<MakeDecisionEvent>,
    mut animals_query: Query<(Entity, &mut Action)>,
) {
    for (entity, mut action) in animals_query.iter_mut() {
        if let Action::DoingNothing { ref mut for_hours } = action.as_mut() {
            if *for_hours > 0 {
                *for_hours -= 1
            } else {
                event_writer.send(MakeDecisionEvent {
                    animal_entity: entity,
                });
            }
        };
    }
}

pub fn discover_animal_state_and_set_action(
    mut event_reader: EventReader<MakeDecisionEvent>,
    mut animals_query: Query<
        (
            Entity,
            &Transform,
            &mut Action,
            &EnergyData,
            &SexualMaturity,
            &Diet,
            &SightRange,
        ),
        With<AnimalMarker>,
    >,
    tile_layout: Res<TileLayout>,
    objects_in_tile_query: Query<&ObjectsInTile>,
    transforms_query: Query<&Transform>,
    potential_partners_query: Query<(&Transform, &SexualMaturity), With<AnimalMarker>>,
) {
    'main_loop: for (event, _) in event_reader.par_read() {
        let Ok((
            entity,
            animal_transform,
            mut animal_action,
            energy_data,
            sexual_maturity,
            animal_diet,
            sight_range,
        )) = animals_query.get_mut(event.animal_entity)
        else {
            continue;
        };

        // TODO: here we should have something like CharacterComponent, that allows us to value each of those states in range [0; 1] and then we choose the most important one for that animal (like aggressive animals, horny etc.)

        let is_hungry = matches!(energy_data.get_hunger_level(), HungerLevel::Hungry);

        'horny: {
            if !sexual_maturity.is_ready_to_reproduce() {
                break 'horny;
            }

            let chosen_partner_entity = tile_layout
                .get_tile_entities_in_range(
                    animal_transform.translation.truncate(),
                    sight_range.gene.phenotype(),
                )
                .iter()
                // NOTE: we take all animals, maybe we should filter by diet (i.e. species once they are introduced)
                .flat_map(|tile_entity| {
                    objects_in_tile_query
                        .get(*tile_entity)
                        .expect("Failed to get tile")
                        .animals
                        .clone()
                })
                .filter_map(|entity| match potential_partners_query.get(entity) {
                    Ok((transform, sexual_maturity)) => {
                        if sexual_maturity.is_ready_to_reproduce() {
                            let distance =
                                animal_transform.translation.distance(transform.translation);

                            Some((entity, distance))
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
                .max_by(|(_, distance1), (_, distance2)| distance1.total_cmp(distance2))
                .map(|(entity, _)| entity);

            match chosen_partner_entity {
                Some(partner_entity) => {
                    *animal_action = Action::Mating {
                        with: partner_entity,
                    };
                    continue 'main_loop;
                }
                None => break 'horny,
            };
        }
        'hungry_non_agressive: {
            if !is_hungry {
                break 'hungry_non_agressive;
            }

            let chosen_food_entity = tile_layout
                .get_tile_entities_in_range(
                    animal_transform.translation.truncate(),
                    sight_range.gene.phenotype(),
                )
                .iter()
                .map(|tile_entity| objects_in_tile_query.get(*tile_entity).expect("Failed to get tile"))
                .flat_map(|objects_in_tile| match animal_diet {
                    Diet::Carnivore => objects_in_tile.animal_carcasses.clone(),
                    Diet::Herbivore => [objects_in_tile.plant_carcasses.clone()].concat(),
                    Diet::Omnivore => [objects_in_tile.plant_carcasses.clone(), objects_in_tile.animal_carcasses.clone()].concat(),
                })
                .map(|entity| {
                    let transform = transforms_query
                        .get(entity)
                        .expect("Failed to get transform for entity despite that entity being in ObjectsInTile");
                    let distance = animal_transform.translation.distance(transform.translation);

                    (entity, distance)
                })
                .max_by(|(_, distance1), (_, distance2)| distance1.total_cmp(distance2))
                .map(|(entity, _)| entity);

            match chosen_food_entity {
                Some(food_entity) => {
                    *animal_action = Action::Eating { food: food_entity };
                    continue 'main_loop;
                }
                None => {
                    break 'hungry_non_agressive;
                }
            };
        }
        'hungry_agressive: {
            if !is_hungry {
                break 'hungry_agressive;
            }

            let chosen_prey_entity = tile_layout
                .get_tile_entities_in_range(
                    animal_transform.translation.truncate(),
                    sight_range.gene.phenotype(),
                )
                .iter()
                .map(|tile_entity| {
                    objects_in_tile_query
                        .get(*tile_entity)
                        .expect("Failed to get tile")
                })
                .flat_map(|objects_in_tile| match animal_diet {
                    Diet::Carnivore => objects_in_tile.animals.clone(),
                    Diet::Herbivore => objects_in_tile.plants.clone(),
                    Diet::Omnivore => [
                        objects_in_tile.plants.clone(),
                        objects_in_tile.animals.clone(),
                    ]
                    .concat(),
                })
                // HACK: sometimes this entity isn't in transform query for some reason, so we just ignore it
                .flat_map(|entity| match transforms_query.get(entity) {
                    Ok(transform) => {
                        let distance = animal_transform.translation.distance(transform.translation);

                        Some((entity, distance))
                    }
                    Err(_) => None,
                })
                .max_by(|(_, distance1), (_, distance2)| distance1.total_cmp(distance2))
                .map(|(entity, _)| entity);

            match chosen_prey_entity {
                Some(prey_entity) => {
                    *animal_action = Action::Attacking { enemy: prey_entity };
                    continue 'main_loop;
                }
                None => break 'hungry_agressive,
            };
        }
        'bored: {
            let wander_around_to = tile_layout.get_random_position_in_range(
                animal_transform.translation.truncate(),
                sight_range.gene.phenotype(),
            );

            *animal_action = Action::GoingTo {
                position: wander_around_to,
            };
        }
    }
}

fn handle_action(
    config: Res<SimulationConfig>,
    mut animals_query: Query<(
        Entity,
        &mut Action,
        &mut Mobile,
        &ActionRange,
        &Transform,
        &AttackDmg,
        &mut EnergyData,
        &mut SexualMaturity,
    )>,
    mut matter_query: Query<(&mut Carcass, &Transform)>,
    mut other_organism_query: Query<
        (&mut Health, &Transform),
        (
            Without<Carcass>,
            Or<(With<PlantMarker>, With<AnimalMarker>)>,
        ),
    >,
    mut reproduction_ew: EventWriter<ReproduceAnimalsEvent>,
    mut other_animal_partner_query: Query<&Transform, With<AnimalMarker>>,
) {
    let mut animals_that_will_reproduce = Vec::new();

    for (
        animal_entity,
        mut action,
        mut mobile,
        action_range,
        transform,
        attack,
        mut energy_data,
        _,
    ) in animals_query.iter_mut()
    {
        match *action {
            Action::DoingNothing { for_hours: _ } => (),
            Action::GoingTo { position } => {
                if position == transform.translation.truncate() {
                    *action = Action::DoingNothing { for_hours: 0 };
                } else {
                    mobile.destination = Some(Destination::Place { position })
                }
            }
            Action::Eating { food: food_entity } => {
                // NOTE: carcass entity could have already disappeared, just ignore it
                let Ok((mut carcass, carcass_transform)) = matter_query.get_mut(food_entity) else {
                    *action = Action::DoingNothing { for_hours: 0 };
                    continue;
                };

                if carcass_transform
                    .translation
                    .distance(transform.translation)
                    < action_range.gene.phenotype()
                {
                    let mut eaten_mass = attack.gene.phenotype();
                    if eaten_mass > carcass.mass {
                        eaten_mass = carcass.mass;
                    }
                    carcass.mass -= eaten_mass;
                    energy_data.store_energy(eaten_mass * carcass.energy_per_mass_unit);
                } else {
                    mobile.destination = Some(Destination::Organism {
                        entity: food_entity,
                    });
                }
            }
            Action::Attacking {
                enemy: enemy_entity,
            } => {
                // NOTE: entity could have already became something else like carcass, just ignore it
                let Ok((mut health, other_transform)) = other_organism_query.get_mut(enemy_entity)
                else {
                    *action = Action::DoingNothing { for_hours: 0 };
                    continue;
                };

                if other_transform.translation.distance(transform.translation)
                    < action_range.gene.phenotype()
                {
                    health.hp -= attack.gene.phenotype();
                } else {
                    mobile.destination = Some(Destination::Organism {
                        entity: enemy_entity,
                    });
                }
            }
            Action::Mating {
                with: partner_entity,
            } => {
                // NOTE: entity could have already became something else like carcass, just ignore it
                let Ok(other_transform) = other_animal_partner_query.get_mut(partner_entity) else {
                    *action = Action::DoingNothing { for_hours: 0 };
                    continue;
                };

                if other_transform.translation.distance(transform.translation)
                    < action_range.gene.phenotype()
                {
                    reproduction_ew.send(ReproduceAnimalsEvent {
                        parent1: animal_entity,
                        parent2: partner_entity,
                    });
                    animals_that_will_reproduce.push(animal_entity);
                    animals_that_will_reproduce.push(partner_entity);
                } else {
                    mobile.destination = Some(Destination::Organism {
                        entity: partner_entity,
                    });
                }
            }
        }
    }

    animals_that_will_reproduce
        .into_iter()
        .for_each(|animal_entity| {
            if let Ok(mut animal) = animals_query.get_mut(animal_entity) {
                animal.7.reset_reproduction_cooldown();
            }
        });
}

#[derive(Component, Reflect, Debug)]
pub struct Mobile {
    pub speed: FloatGene,
    pub destination: Option<Destination>,
    pub next_step_destination: Option<Vec2>,
}

#[derive(Reflect, Debug)]
pub enum Destination {
    Place { position: Vec2 },
    Organism { entity: Entity },
}

pub fn find_next_step_destination(
    query_organisms: Query<&Transform>,
    mut query_mobiles: Query<&mut Mobile>,
) {
    for mut mobile in query_mobiles.iter_mut() {
        if let Some(destination) = &mobile.destination {
            match *destination {
                Destination::Place { position } => mobile.next_step_destination = position.into(),
                Destination::Organism { entity } => match query_organisms.get(entity) {
                    Ok(other) => mobile.next_step_destination = other.translation.truncate().into(),
                    Err(_) => mobile.destination = None,
                },
            };
        }
    }
}

pub fn make_step(
    mut query: Query<(Entity, &mut Mobile, &mut Transform)>,
    tile_layout: Res<TileLayout>,
    mut objects_in_tiles: Query<&mut ObjectsInTile>,
    biome_types: Query<&BiomeType>,
) {
    for (entity, mut mobile, mut transform) in query.iter_mut() {
        let Some(dest_position) = mobile.next_step_destination else {
            continue;
        };
        let prev_position = transform.translation.truncate();
        let position_diff = dest_position - prev_position;

        let move_by = if position_diff.length() <= mobile.speed.phenotype() {
            mobile.destination = None;
            position_diff
        } else {
            position_diff.normalize() * mobile.speed.phenotype()
        };

        let new_position = prev_position + move_by;
        let prev_tile_entity = tile_layout.get_tile_entity_for_position(prev_position);
        let next_tile_entity = tile_layout.get_tile_entity_for_position(new_position);

        let animal_can_live_here = biome_types
            .get(next_tile_entity)
            .expect("Failed to get next tile biome")
            .animals_can_live_here();

        if tile_layout.is_position_in_bounds(new_position) && animal_can_live_here {
            transform.translation.x = new_position.x;
            transform.translation.y = new_position.y;
            mobile.next_step_destination = None;

            if prev_tile_entity != next_tile_entity {
                objects_in_tiles
                    .get_mut(prev_tile_entity)
                    .expect("Failed to get previous tile objects")
                    .remove_animal_entity(entity);

                objects_in_tiles
                    .get_mut(next_tile_entity)
                    .expect("Failed to get next tile objects")
                    .add_animal_entity(entity);
            }
        } else {
            mobile.destination = None;
        }
    }
}
