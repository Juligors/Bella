pub mod mobile;
pub mod visual;

use crate::bella::{
    config::SimConfig,
    organism::LifeState,
    terrain::{thermal_conductor::ThermalConductor, TileMap},
    time::HourPassedEvent,
    ui::layer::SpriteLayer,
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};
use rand::{self, Rng};
use std::time::Duration;

use self::{
    mobile::{choose_new_animal_destination, move_mobile, Mobile},
    visual::{prepare_animal_assets, update_animal_color, AnimalAssets},
};

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                prepare_animal_assets,
                spawn_animals.after(prepare_animal_assets),
            ),
        )
        .add_systems(
            Update,
            (
                move_mobile,
                update_animal_color,
                connect_animal_with_medium_its_on,
                decrease_satiation,
                (choose_new_animal_destination).run_if(on_timer(Duration::from_secs(3))),
            ),
        )
        .register_type::<Mobile>()
        .register_type::<HungerLevel>()
        .register_type::<SightRange>();
    }
}

#[derive(Component)]
pub struct AnimalMarker;

#[derive(Component, Reflect)]
pub enum HungerLevel {
    Satiated(u32),
    Hungry(u32),
    Starving,
}

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct SightRange(f32);

fn spawn_animals(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimConfig>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(Circle::new(3.)));
    let mut rng = rand::thread_rng();

    for x in 0..config.creature_spawn_x {
        for y in 0..config.creature_spawn_y {
            let hp = 100.;
            cmd.spawn((
                AnimalMarker,
                MaterialMesh2dBundle {
                    mesh: mesh_handle.clone(),
                    material: animal_assets.alive[hp as usize].clone(),
                    transform: Transform::from_xyz(x as f32 * -10. - 50., y as f32 * -5. - 50., 0.),
                    ..default()
                },
                LifeState::Alive { hp },
                Mobile {
                    dest: None,
                    speed: rng.gen_range(0.2..0.3),
                },
                HungerLevel::Satiated(100),
                SightRange(500.),
                SpriteLayer::Creature,
            ));
        }
    }
}

fn decrease_satiation(
    mut hunger_levels: Query<(&mut HungerLevel, &mut LifeState)>,
    mut er_hour_passed: EventReader<HourPassedEvent>,
) {
    if er_hour_passed.is_empty() {
        return;
    }

    er_hour_passed.clear();

    for (mut hunger_level, mut life_state) in hunger_levels.iter_mut() {
        if let LifeState::Alive { hp } = life_state.as_mut() {
            *hunger_level = match *hunger_level {
                HungerLevel::Satiated(level) => {
                    if level > 1 {
                        HungerLevel::Satiated(level - 10)
                    } else {
                        HungerLevel::Hungry(100)
                    }
                }
                HungerLevel::Hungry(level) => {
                    if level > 1 {
                        HungerLevel::Hungry(level - 20)
                    } else {
                        HungerLevel::Starving
                    }
                }
                HungerLevel::Starving => {
                    *hp -= 10.;
                    HungerLevel::Starving
                }
            }
        }
    }
}

// TODO: this doesn't do much, but this logic should be used later on
fn connect_animal_with_medium_its_on(
    creature_transforms: Query<&Transform, With<AnimalMarker>>,
    tiles: Query<(Entity, &ThermalConductor)>,
    map: Res<TileMap>,
) {
    for creature_transform in creature_transforms.iter() {
        let entity = map.world_pos_to_entity(Vec2 {
            x: creature_transform.translation.x,
            y: creature_transform.translation.y,
        });

        match entity {
            Some(e) => {
                for (tile_entity, _medium) in tiles.iter() {
                    if tile_entity != e {
                        continue;
                    }

                    // println!(
                    //     "Right now I'm over tile with temperature {}",
                    //     medium.get_temp()
                    // );
                }
            }
            None => println!("No tile under this creature :("),
        }
    }
}
