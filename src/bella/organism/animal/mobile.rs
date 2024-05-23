use bevy::prelude::*;
use rand::{self, Rng};

use crate::bella::{
    organism::{plant::PlantMarker, LifeState},
    terrain::TileMap,
};

use super::{AnimalMarker, HungerLevel, SightRange};

#[derive(Component, Reflect)]
pub struct Mobile {
    pub dest: Option<Vec2>,
    pub speed: f32,
}

pub fn move_mobile(mut creatures: Query<(&mut Mobile, &mut Transform)>, map: Res<TileMap>) {
    for (mobile, mut transform) in creatures.iter_mut() {
        if let Some(destination) = mobile.dest {
            let old_position = Vec2::new(transform.translation.x, transform.translation.y);
            let position_diff = destination - old_position;

            let move_by = if position_diff.length() <= mobile.speed {
                position_diff
            } else {
                position_diff.normalize() * mobile.speed
            };

            let new_position = old_position + move_by;
            if map.world_pos_in_entities(new_position) {
                transform.translation.x = new_position.x;
                transform.translation.y = new_position.y;
            }
        }
    }
}

pub fn choose_new_animal_destination(
    mut creatures: Query<
        (
            &mut Mobile,
            &LifeState,
            &Transform,
            &HungerLevel,
            &SightRange,
        ),
        With<AnimalMarker>,
    >,
    plants: Query<&Transform, With<PlantMarker>>,
) {
    let mut rng = rand::thread_rng();

    for (mut moving, life_state, transform, hunger_level, sight_range) in creatures.iter_mut() {
        if let LifeState::Dead = life_state {
            moving.speed = 0.;
            continue;
        }

        match hunger_level {
            HungerLevel::Satiated(_) => continue,
            HungerLevel::Hungry(_) | HungerLevel::Starving => {
                let nearest_plant = plants
                    .iter()
                    .map(|&p_trans| {
                        let plant = Vec2::new(p_trans.translation.x, p_trans.translation.y);
                        let creature = Vec2::new(transform.translation.x, transform.translation.y);
                        (plant, creature.distance(plant))
                    })
                    .filter(|(_, distance)| distance < sight_range)
                    .min_by(|a, b| a.1.total_cmp(&b.1));

                moving.dest = match nearest_plant {
                    Some((plant_pos, _)) => Some(plant_pos),
                    None => Some(Vec2::new(
                        // TODO: hardcoded values
                        transform.translation.x + rng.gen_range(-1000.0..1000.0),
                        transform.translation.y + rng.gen_range(-1000.0..1000.0),
                    )),
                }
            }
        }
    }
}
