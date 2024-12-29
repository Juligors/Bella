use bevy::prelude::*;
use rand::Rng;

use super::{Attack, HungerLevel, SightRange};
use crate::bella::{
    organism::{EnergyData, Health},
    pause::PauseState,
    restart::SimState,
    terrain::tile::TileLayout,
};

pub struct MobilePlugin;

impl Plugin for MobilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((find_next_step_destination, make_step).chain(), attack)
                .run_if(in_state(SimState::Simulation))
                .run_if(in_state(PauseState::Running)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Mobile {
    pub speed: f32,
    pub destination: Option<Destination>,
    pub next_step_destination: Option<Vec2>,
}

#[derive(Debug)]
pub enum Destination {
    Place { position: Vec2 },
    Organism { entity: Entity },
}

pub fn find_next_step_destination(
    query_organisms: Query<&Transform>,
    mut query_mobiles: Query<(&mut Mobile, &Transform, &SightRange)>,
) {
    let mut rng = rand::thread_rng();

    for (mut mobile, transform, sight_range) in query_mobiles.iter_mut() {
        match &mobile.destination {
            // we already have a destination, make it move there
            Some(destination) => {
                mobile.next_step_destination = match &destination {
                    Destination::Place { position } => Some(*position),
                    Destination::Organism { entity } => match query_organisms.get(*entity) {
                        Ok(other) => Some(other.translation.truncate()),
                        Err(_) => None,
                    },
                }
            }
            // no destination, let's choose a random one
            None => {
                // TODO: this can go out of bounds
                // TODO: magic number
                let r = **sight_range;
                let new_destination = Vec2::new(
                    transform.translation.x + rng.gen_range(-r..r),
                    transform.translation.y + rng.gen_range(-r..r),
                );

                mobile.destination = Some(Destination::Place {
                    position: new_destination,
                });

                mobile.next_step_destination = Some(new_destination);
            }
        }
    }
}

pub fn make_step(mut query: Query<(&mut Mobile, &mut Transform)>, tile_layout: Res<TileLayout>) {
    for (mut mobile, mut transform) in query.iter_mut() {
        // TODO: this field probably shouldn't exist, it should just be a local variable
        if mobile.next_step_destination.is_none() {
            continue;
        }

        let dest_position = mobile.next_step_destination.as_mut().unwrap();
        let old_position = transform.translation.truncate();
        let position_diff = *dest_position - old_position;

        let move_by = if position_diff.length() <= mobile.speed {
            mobile.destination = None;
            position_diff
        } else {
            position_diff.normalize() * mobile.speed
        };

        let new_position = old_position + move_by;

        if !tile_layout.is_position_in_bounds(new_position) {
            mobile.destination = None;
            continue;
        }

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;

        mobile.next_step_destination = None;
    }
}

fn attack(
    mut query: Query<(
        &Attack,
        &mut Mobile,
        &mut HungerLevel,
        &mut EnergyData,
        &Transform,
    )>,
    mut query_organisms: Query<(&mut Health, &Transform)>,
) {
    for (attack, mut mobile, mut hunger_level, mut energy_data, transform) in query.iter_mut() {
        if mobile.destination.is_none() {
            continue;
        }

        match mobile.destination.as_ref().unwrap() {
            Destination::Place { position: _ } => continue,
            Destination::Organism { entity: target } => match query_organisms.get_mut(*target) {
                Ok((mut health, other_transform)) => {
                    let distance = other_transform.translation.distance(transform.translation);
                    if distance > attack.range {
                        continue;
                    }

                    // TODO: this should also give energy/hunger to the animal, probably with event
                    health.hp -= attack.damage;
                    energy_data.energy += 100.;
                    *hunger_level = match *hunger_level {
                        HungerLevel::Satiated(v) => HungerLevel::Hungry((v + 1).clamp(0, 100)),
                        HungerLevel::Hungry(v) => HungerLevel::Hungry((v + 1).clamp(0, 100)),
                        HungerLevel::Starving => HungerLevel::Hungry(1),
                    }
                }
                Err(_) => mobile.destination = None,
            },
        }
    }
}
