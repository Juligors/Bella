use super::{AnimalMarker, Attack};
use crate::bella::{
    organism::{carcass::Carcass, plant::PlantMarker, EnergyDatav3, Health},
    pause::PauseState,
    restart::SimulationState,
    terrain::tile::TileLayout,
};
use bevy::prelude::*;

pub struct MobilePlugin;

impl Plugin for MobilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Mobile>()
            .register_type::<Destination>()
            .add_systems(
                Update,
                (
                    (find_next_step_destination, make_step).chain(),
                    attack,
                    eat_matter,
                )
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(PauseState::Running)),
            );
    }
}

// TODO: to powinna być część zwierząt i tyle
#[derive(Component, Reflect, Debug)]
pub struct Mobile {
    pub speed: f32,
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
            mobile.next_step_destination = match &destination {
                Destination::Place { position } => Some(*position),
                Destination::Organism { entity } => match query_organisms.get(*entity) {
                    Ok(other) => Some(other.translation.truncate()),
                    Err(_) => {
                        mobile.destination = None;
                        continue;
                    }
                },
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
    mut query_self: Query<(&Attack, &Mobile, &Transform)>,
    mut query_other_organisms: Query<
        (&mut Health, &Transform),
        Or<(With<PlantMarker>, With<AnimalMarker>)>,
    >,
) {
    for (attack, mobile, transform) in query_self.iter_mut() {
        if mobile.destination.is_none() {
            continue;
        }

        match mobile.destination.as_ref().unwrap() {
            Destination::Place { position: _ } => continue,
            Destination::Organism { entity: target } => {
                match query_other_organisms.get_mut(*target) {
                    Ok((mut health, other_transform)) => {
                        let distance = other_transform.translation.distance(transform.translation);
                        if distance > attack.range {
                            continue;
                        }

                        health.hp -= attack.damage;
                    }
                    Err(_) => {
                        // This means that destination organism isn't Plant/Animal, so just ignore it
                    }
                }
            }
        }
    }
}

fn eat_matter(
    mut query: Query<(&Attack, &Mobile, &Transform, &mut EnergyDatav3)>,
    mut query_matter: Query<(&mut Carcass, &Transform)>,
) {
    for (attack, mobile, transform, mut energy_data) in query.iter_mut() {
        if mobile.destination.is_none() {
            continue;
        }

        match mobile.destination.as_ref().unwrap() {
            Destination::Place { position: _ } => continue,
            Destination::Organism { entity: target } => match query_matter.get_mut(*target) {
                Ok((mut carcass, other_transform)) => {
                    let distance = other_transform.translation.distance(transform.translation);
                    if distance > attack.range {
                        continue;
                    }

                    let mut eaten_mass = attack.damage / 5.0;
                    if eaten_mass > carcass.mass {
                        eaten_mass = carcass.mass;
                    }
                    carcass.mass -= eaten_mass;
                    energy_data.store_energy(eaten_mass * carcass.energy_per_mass_unit);
                }
                Err(_) => {
                    // This means that destination organism isn't PlantMatter/AnimalMatter, so just ignore it
                }
            },
        }
    }
}
