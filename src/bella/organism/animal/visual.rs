use bevy::prelude::*;

use super::{AnimalMarker, Diet};

#[derive(Resource)]
pub struct AnimalAssets {
    pub alive: Vec<Handle<ColorMaterial>>,
    pub dead: Handle<ColorMaterial>,
    pub carnivorous: Handle<ColorMaterial>,
    pub herbivorous: Handle<ColorMaterial>,
}

pub fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let animal_assets = AnimalAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(i as f32 / 100., 0.3, i as f32 / 100.)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
        carnivorous: materials.add(Color::rgb(1., 0.3, 0.3)),
        herbivorous: materials.add(Color::rgb(0.3, 1., 0.7)),
    };

    cmd.insert_resource(animal_assets);
}

pub fn update_animal_color(
    mut query: Query<(&mut Handle<ColorMaterial>, &Diet), With<AnimalMarker>>,
    assets: Res<AnimalAssets>,
) {
    for (mut handle, diet) in query.iter_mut() {
        *handle = match &diet {
            Diet::Carnivorous(_) => assets.carnivorous.clone(),
            Diet::Herbivorous(_) => assets.herbivorous.clone(),
        }
    }
}
