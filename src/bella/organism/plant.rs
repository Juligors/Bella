use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::bella::{config::SimConfig, organism::LifeState, ui::layer::SpriteLayer};

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                prepare_plant_assets,
                spawn_plants.after(prepare_plant_assets),
            ),
        )
        .add_systems(Update, update_plant_color);
    }
}

#[derive(Component)]
pub struct PlantMarker;

#[derive(Resource)]
struct PlantAssets {
    alive: Vec<Handle<ColorMaterial>>,
    dead: Handle<ColorMaterial>,
}

fn prepare_plant_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let plant_assets = PlantAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(0.3, i as f32 / 100., 0.3)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
    };

    cmd.insert_resource(plant_assets);
}

fn spawn_plants(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    plant_assets: Res<PlantAssets>,
    config: Res<SimConfig>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(Rectangle::new(3., 3.)));

    for x in 0..config.plant_spawn_x {
        for y in 0..config.plant_spawn_y {
            let hp = 100.;
            cmd.spawn((
                PlantMarker,
                MaterialMesh2dBundle {
                    mesh: mesh_handle.clone(),
                    material: plant_assets.alive[hp as usize].clone(),
                    transform: Transform::from_xyz(x as f32 * 25., y as f32 * 20., 1.),
                    ..default()
                },
                LifeState::Alive { hp },
                SpriteLayer::Plant,
            ));
        }
    }
}

fn update_plant_color(
    mut plants: Query<(&mut Handle<ColorMaterial>, &mut LifeState), With<PlantMarker>>,
    assets: Res<PlantAssets>,
) {
    for (mut handle, mut life_state) in plants.iter_mut() {
        // TODO: remove this
        if let LifeState::Alive { hp } = life_state.as_mut() {
            *hp -= 0.3;
            if *hp <= 0. {
                *life_state = LifeState::Dead;
            }
        }

        match life_state.as_mut() {
            LifeState::Alive { hp } => *handle = assets.alive[*hp as usize].clone(),
            LifeState::Dead => *handle = assets.dead.clone(),
        }
    }
}
