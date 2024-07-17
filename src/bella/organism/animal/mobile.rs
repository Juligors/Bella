use bevy::prelude::*;

use crate::bella::terrain::TileMap;

pub struct MobilePlugin;

impl Plugin for MobilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_mobile);
    }
}

#[derive(Component)]
pub struct Mobile {
    pub dest: Option<Vec2>,
    pub speed: f32,
}

pub fn move_mobile(mut creatures: Query<(&mut Mobile, &mut Transform)>, map: Res<TileMap>) {
    for (mut mobile, mut transform) in creatures.iter_mut() {
        if let Some(destination) = mobile.dest {
            let old_position = transform.translation.truncate();
            let position_diff = destination - old_position;

            let move_by = if position_diff.length() <= mobile.speed {
                mobile.dest = None;
                position_diff
            } else {
                position_diff.normalize() * mobile.speed
            };

            let new_position = old_position + move_by;

            if !map.world_pos_in_entities(new_position) {
                mobile.dest = None;
                continue;
            }

            transform.translation.x = new_position.x;
            transform.translation.y = new_position.y;
        }
    }
}
