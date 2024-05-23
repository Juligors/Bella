use bevy::prelude::*;

use crate::bella::organism::LifeState;

use super::AnimalMarker;

#[derive(Resource)]
pub struct AnimalAssets {
    pub alive: Vec<Handle<ColorMaterial>>,
    pub dead: Handle<ColorMaterial>,
}

pub fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let animal_assets = AnimalAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(i as f32 / 100., 0.3, i as f32 / 100.)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
    };

    cmd.insert_resource(animal_assets);
}

pub fn update_animal_color(
    mut query: Query<(&mut Handle<ColorMaterial>, &mut LifeState), With<AnimalMarker>>,
    assets: Res<AnimalAssets>,
) {
    for (mut handle, mut life_state) in query.iter_mut() {
        match life_state.as_mut() {
            LifeState::Alive { hp } => *handle = assets.alive[*hp as usize].clone(),
            LifeState::Dead => *handle = assets.dead.clone(),
        }
    }
}