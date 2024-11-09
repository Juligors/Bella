use bevy::prelude::*;

use super::Attack;
use crate::bella::{organism::Health, pause::PauseState, terrain::TileMap};

pub struct MobilePlugin;

impl Plugin for MobilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((find_next_step_destination, make_step).chain(), attack)
                .run_if(in_state(PauseState::Running)),
        );
    }
}

#[derive(Component)]
pub struct Mobile {
    pub speed: f32,
    pub destination: Option<Destination>,
    pub next_step_destination: Option<Vec2>,
}

pub enum Destination {
    Place { position: Vec2 },
    Organism { entity: Entity },
}

pub fn find_next_step_destination(
    query_organisms: Query<&Transform>,
    mut query_mobiles: Query<&mut Mobile>,
) {
    for mut mobile in query_mobiles.iter_mut() {
        match &mobile.destination {
            Some(destination) => {
                mobile.next_step_destination = match &destination {
                    Destination::Place { position } => Some(*position),
                    Destination::Organism { entity } => match query_organisms.get(*entity) {
                        Ok(other) => Some(other.translation.truncate()),
                        Err(_) => None,
                    },
                }
            }
            None => continue,
        }
    }
}

pub fn make_step(mut query: Query<(&mut Mobile, &mut Transform)>, map: Res<TileMap>) {
    for (mut mobile, mut transform) in query.iter_mut() {
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

        if !map.world_pos_in_entities(new_position) {
            mobile.destination = None;
            continue;
        }

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;

        mobile.next_step_destination = None;
    }
}

fn attack(
    mut query: Query<(&Attack, &mut Mobile, &Transform)>,
    mut query_organisms: Query<(&mut Health, &Transform)>,
) {
    for (attack, mut mobile, transform) in query.iter_mut() {
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
                }
                Err(_) => mobile.destination = None,
            },
        }
    }
}
