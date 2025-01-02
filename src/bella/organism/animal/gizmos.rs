use bevy::prelude::*;

use crate::bella::{inspector::EguiFocusState, organism::plant::PlantMarker, restart::SimState};

use super::{
    mobile::{Destination, Mobile},
    AnimalMarker, Diet, SightRange,
};

pub struct AnimalGizmosPlugin;

impl Plugin for AnimalGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AnimalGizmosOverlayState>()
            .add_systems(
                Update,
                change_overlay_state_based_on_keyboard_input
                    .run_if(in_state(EguiFocusState::IsNotFocused)),
            )
            .add_systems(
                Update,
                (
                    draw_gizmo_to_animal_destination,
                    draw_gizmo_of_animal_sight_range,
                )
                    .run_if(in_state(SimState::Simulation))
                    .run_if(in_state(AnimalGizmosOverlayState::Visible)),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AnimalGizmosOverlayState {
    Visible,
    #[default]
    Hidden,
}

fn change_overlay_state_based_on_keyboard_input(
    current_state: Res<State<AnimalGizmosOverlayState>>,
    mut next_state: ResMut<NextState<AnimalGizmosOverlayState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        next_state.set(match **current_state {
            AnimalGizmosOverlayState::Visible => AnimalGizmosOverlayState::Hidden,
            AnimalGizmosOverlayState::Hidden => AnimalGizmosOverlayState::Visible,
        });
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
        let color = get_color_for_diet(diet);

        gizmos.line(start, end, color);
    }
}

fn draw_gizmo_of_animal_sight_range(
    mut gizmos: Gizmos,
    animals: Query<(&Transform, &SightRange, &Diet), With<AnimalMarker>>,
) {
    for (transform, sight_range, diet) in animals.iter() {
        let isometry = Isometry3d::from_translation(transform.translation);
        let radius = **sight_range;
        let color = get_color_for_diet(diet);

        gizmos.circle(isometry, radius, color).resolution(64);
    }
}

fn get_color_for_diet(diet: &Diet) -> Srgba {
    match diet {
        Diet::Carnivorous => bevy::color::palettes::css::RED,
        Diet::Herbivorous => bevy::color::palettes::css::GREEN,
        Diet::Omnivore => bevy::color::palettes::css::BLUE,
    }
}
