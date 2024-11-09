use bevy::prelude::*;

use crate::bella::organism::plant::PlantMarker;

use super::{
    mobile::{Destination, Mobile},
    AnimalMarker, Diet, SightRange,
};

pub struct AnimalGizmosPlugin;

impl Plugin for AnimalGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_gizmo_to_animal_destination,
                draw_gizmo_of_animal_sight_range,
            ),
        );
    }
}

fn draw_gizmo_to_animal_destination(
    mut gizmos: Gizmos,
    mobiles: Query<(&Transform, &Mobile, &Diet)>,
    organisms: Query<&Transform, Or<(With<AnimalMarker>, With<PlantMarker>)>>,
) {
    for (transform, mobile, diet) in mobiles.iter() {
        if mobile.destination.is_none() {
            continue;
        }

        let start = transform.translation;
        let end = match mobile.destination.as_ref().unwrap() {
            Destination::Place { position } => position.extend(start.z),
            Destination::Organism { entity } => match organisms.get(*entity) {
                Ok(transform) => transform.translation,
                Err(_) => continue,
            },
        };
        let color = match diet {
            Diet::Carnivorous(_) => bevy::color::palettes::css::RED,
            Diet::Herbivorous(_) => bevy::color::palettes::css::GREEN,
        };

        gizmos.line(start, end, color);
    }
}

fn draw_gizmo_of_animal_sight_range(
    mut gizmos: Gizmos,
    animals: Query<(&Transform, &SightRange, &Diet), With<AnimalMarker>>,
) {
    for (transform, sight_range, diet) in animals.iter(){
        let position = transform.translation;
        let normal = Dir3::Z;
        let radius = **sight_range;
        let color = match diet {
            Diet::Carnivorous(_) => bevy::color::palettes::css::RED,
            Diet::Herbivorous(_) => bevy::color::palettes::css::GREEN,
        };

        gizmos.circle(position, normal, radius, color);
    }
}
