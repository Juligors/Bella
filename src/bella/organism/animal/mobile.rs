use bevy::prelude::*;

use crate::bella::terrain::TileMap;

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
