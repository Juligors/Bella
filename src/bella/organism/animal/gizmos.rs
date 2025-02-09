use bevy::prelude::*;

use crate::bella::{
    inspector::{ChosenEntity, EguiFocusState},
    organism::plant::PlantMarker,
    restart::SimulationState,
};

use super::{
    mobile::{Destination, Mobile},
    AnimalMarker, Attack, Diet, SightRange,
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
                    .run_if(in_state(SimulationState::Simulation))
                    .run_if(in_state(AnimalGizmosOverlayState::Visible)),
            )
            .add_systems(
                Update,
                (
                    draw_gizmo_to_animal_destination_for_chosen_animal,
                    draw_gizmo_of_animal_sight_range_for_chosen_animal,
                    draw_gizmo_of_animal_attack_range_for_chosen_animal,
                ),
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
    mut mobiles: Query<(&Transform, &mut Mobile, &Diet)>,
    organisms: Query<&Transform, Or<(With<AnimalMarker>, With<PlantMarker>)>>, // TODO: this should be Edible component
) {
    for (transform, mut mobile, diet) in mobiles.iter_mut() {
        if mobile.destination.is_none() {
            continue;
        }

        let start = transform.translation;
        let end = match mobile.destination.as_ref().unwrap() {
            Destination::Place { position } => position.extend(start.z),
            Destination::Organism { entity } => match organisms.get(*entity) {
                Ok(transform) => transform.translation,
                Err(_) => {
                    debug!("Entity {} doesn't exist despite Destination pointing to it (should we do something about it?)", entity);
                    mobile.destination = None;
                    continue;
                }
            },
        };
        let color = get_color_for_diet(diet);

        gizmos.line(start, end, color);
    }
}
fn draw_gizmo_to_animal_destination_for_chosen_animal(
    mut gizmos: Gizmos,
    mut mobiles: Query<(&Transform, &mut Mobile, &Diet)>,
    organisms: Query<&Transform, Or<(With<AnimalMarker>, With<PlantMarker>)>>, // TODO: this should be Edible component
    chosen_entity: Res<ChosenEntity>,
) {
    if chosen_entity.entity.is_none() {
        return;
    }

    if let Ok((transform, mut mobile, diet)) = mobiles.get_mut(chosen_entity.entity.unwrap()) {
        if mobile.destination.is_none() {
            return;
        }

        let start = transform.translation;
        let end = match mobile.destination.as_ref().unwrap() {
            Destination::Place { position } => position.extend(start.z),
            Destination::Organism { entity } => match organisms.get(*entity) {
                Ok(transform) => transform.translation,
                Err(_) => {
                    debug!("Entity {} doesn't exist despite Destination pointing to it (should we do something about it?)", entity);
                    mobile.destination = None;
                    return;
                }
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

        gizmos.circle(isometry, radius, color).resolution(32);
    }
}

fn draw_gizmo_of_animal_sight_range_for_chosen_animal(
    mut gizmos: Gizmos,
    animals: Query<(&Transform, &SightRange, &Diet), With<AnimalMarker>>,
    chosen_entity: Res<ChosenEntity>,
) {
    if chosen_entity.entity.is_none() {
        return;
    }

    if let Ok((transform, sight_range, diet)) = animals.get(chosen_entity.entity.unwrap()) {
        let isometry = Isometry3d::from_translation(transform.translation);
        let radius = **sight_range;
        let color = get_color_for_diet(diet);

        gizmos.circle(isometry, radius, color).resolution(32);
    }
}

fn draw_gizmo_of_animal_attack_range_for_chosen_animal(
    mut gizmos: Gizmos,
    animals: Query<(&Transform, &Attack, &Diet), With<AnimalMarker>>,
    chosen_entity: Res<ChosenEntity>,
) {
    if chosen_entity.entity.is_none() {
        return;
    }

    if let Ok((transform, attack, diet)) = animals.get(chosen_entity.entity.unwrap()) {
        let isometry = Isometry3d::from_translation(transform.translation);
        let radius = attack.range;
        let color = get_color_for_diet(diet);

        gizmos.circle(isometry, radius, color).resolution(32);
    }
}

fn get_color_for_diet(diet: &Diet) -> Srgba {
    match diet {
        Diet::Carnivorous => bevy::color::palettes::css::RED,
        Diet::Herbivorous => bevy::color::palettes::css::GREEN,
        Diet::Omnivore => bevy::color::palettes::css::BLUE,
    }
}
