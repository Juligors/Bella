pub mod mobile;

use crate::bella::{
    config::SimConfig,
    organism::{plant::PlantMarker, LifeState},
    terrain::{thermal_conductor::ThermalConductor, TileMap},
    ui::layer::SpriteLayer,
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};
use rand::{self, Rng};
use std::time::Duration;

use self::mobile::{move_mobile, Mobile};

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
            ),
        )
        .add_systems(
            Update,
            (update_animal_destination).run_if(on_timer(Duration::from_secs(3))),
        )
        .register_type::<Mobile>()
        .register_type::<HungerLevel>()
        .register_type::<SightRange>();
    }
}

#[derive(Component)]
struct AnimalMarker;

#[derive(Component, Reflect)]
enum HungerLevel {
    Satiated(u32),
    Hungry(u32),
    Starving,
}

#[derive(Component, Reflect, Deref, DerefMut)]
struct SightRange(f32);

#[derive(Resource)]
struct AnimalAssets {
    alive: Vec<Handle<ColorMaterial>>,
    dead: Handle<ColorMaterial>,
}

fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let animal_assets = AnimalAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(i as f32 / 100., 0.3, i as f32 / 100.)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
    };

    cmd.insert_resource(animal_assets);
}

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

// TODO: add this to the sceduler. How often should it run? probably every time event like "HourPassed" or "TimeTickPassed" is emited.
fn decrease_satiation(mut hunger_levels: Query<(&mut HungerLevel, &mut LifeState)>) {
    for (mut hunger_level, mut life_state) in hunger_levels.iter_mut() {
        if let LifeState::Alive { hp } = life_state.as_mut() {
            *hunger_level = match *hunger_level {
                HungerLevel::Satiated(level) => {
                    if level > 1 {
                        HungerLevel::Satiated(level - 1)
                    } else {
                        HungerLevel::Hungry(100)
                    }
                }
                HungerLevel::Hungry(level) => {
                    if level > 1 {
                        HungerLevel::Hungry(level - 1)
                    } else {
                        HungerLevel::Starving
                    }
                }
                HungerLevel::Starving => {
                    *hp -= 1.;
                    if *hp <= 0. {
                        *life_state = LifeState::Dead;
                    }
                    HungerLevel::Starving // TODO: can something be starving and dead at the same time? should probably change this enum
                }
            }
        }
    }
}

fn update_animal_destination(
    mut creatures: Query<
        (
            &mut Mobile,
            &LifeState,
            &Transform,
            &HungerLevel,
            &SightRange,
        ),
        With<AnimalMarker>,
    >,
    plants: Query<&Transform, With<PlantMarker>>,
) {
    let mut rng = rand::thread_rng();

    for (mut moving, life_state, transform, hunger_level, sight_range) in creatures.iter_mut() {
        if let LifeState::Dead = life_state {
            moving.speed = 0.;
            continue;
        }

        match hunger_level {
            HungerLevel::Satiated(_) => continue,
            HungerLevel::Hungry(_) | HungerLevel::Starving => {
                let nearest_plant = plants
                    .iter()
                    .map(|&p_trans| {
                        let plant = Vec2::new(p_trans.translation.x, p_trans.translation.y);
                        let creature = Vec2::new(transform.translation.x, transform.translation.y);
                        (plant, creature.distance(plant))
                    })
                    .filter(|(_, distance)| distance < sight_range)
                    .min_by(|a, b| a.1.total_cmp(&b.1));

                moving.dest = match nearest_plant {
                    Some((plant_pos, _)) => Some(plant_pos),
                    None => Some(Vec2::new(
                        // TODO: hardcoded values
                        transform.translation.x + rng.gen_range(-1000.0..1000.0),
                        transform.translation.y + rng.gen_range(-1000.0..1000.0),
                    )),
                }
            }
        }
    }
}

fn update_animal_color(
    mut query: Query<(&mut Handle<ColorMaterial>, &mut LifeState), With<AnimalMarker>>,
    assets: Res<AnimalAssets>,
) {
    for (mut handle, mut life_state) in query.iter_mut() {
        // TODO: remove this
        if let LifeState::Alive { hp } = life_state.as_mut() {
            *hp -= 0.01;
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
