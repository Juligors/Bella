use bevy::prelude::*;

use crate::bella::{
    organism::plant::PlantMarker,
    restart::SimulationState,
    ui_facade::{ChosenEntity, EguiFocusState},
};

use super::{
    animal_ai::{Destination, Mobile},
    ActionRange, AnimalMarker, AttackDmg, Diet, SightRange,
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
                    println!("Entity {} doesn't exist despite Destination pointing to it (should we do something about it?)", entity);
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
                    println!("Entity {} doesn't exist despite Destination pointing to it (should we do something about it?)", entity);
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
        let radius = sight_range.gene.phenotype();
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
        let radius = sight_range.gene.phenotype();
        let color = get_color_for_diet(diet);

        gizmos.circle(isometry, radius, color).resolution(32);
    }
}

fn draw_gizmo_of_animal_attack_range_for_chosen_animal(
    mut gizmos: Gizmos,
    animals: Query<(&Transform, &ActionRange, &Diet), With<AnimalMarker>>,
    chosen_entity: Res<ChosenEntity>,
) {
    if chosen_entity.entity.is_none() {
        return;
    }

    if let Ok((transform, action_range, diet)) = animals.get(chosen_entity.entity.unwrap()) {
        let isometry = Isometry3d::from_translation(transform.translation);
        let radius = action_range.gene.phenotype();
        let color = get_color_for_diet(diet);

        gizmos.circle(isometry, radius, color).resolution(32);
    }
}

fn get_color_for_diet(diet: &Diet) -> Srgba {
    match diet {
        Diet::Carnivore => bevy::color::palettes::css::RED,
        Diet::Herbivore => bevy::color::palettes::css::GREEN,
        Diet::Omnivore => bevy::color::palettes::css::BLUE,
    }
}
