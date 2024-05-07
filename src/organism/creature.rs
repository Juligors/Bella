use crate::layer::SpriteLayer;
use crate::terrain::{thermal::ThermalConductor, TileMap};
use crate::{
    config::SimConfig,
    organism::{plant::PlantMarker, LifeState},
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};
use rand::{self, Rng};
use std::time::Duration;

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                prepare_creature_assets,
                spawn_creatures.after(prepare_creature_assets),
            ),
        )
        .add_systems(
            Update,
            (
                move_creatures,
                update_creature_color,
                connect_creature_with_medium_its_on,
                decrease_satiation,
            ),
        )
        .add_systems(
            Update,
            (update_creature_destination).run_if(on_timer(Duration::from_secs(3))),
        );
    }
}

#[derive(Component)]
struct CreatureMarker;

#[derive(Component)]
struct Moving {
    dest: Option<Vec2>,
    speed: f32,
}

#[derive(Component)]
enum HungerLevel {
    Satiated(u32),
    Hungry(u32),
    Starving,
}

#[derive(Component, Deref, DerefMut)]
struct SightRange(f32);

#[derive(Resource)]
struct CreatureAssets {
    alive: Vec<Handle<ColorMaterial>>,
    dead: Handle<ColorMaterial>,
}

fn prepare_creature_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let creature_assets = CreatureAssets {
        alive: (0..=100)
            .map(|i| materials.add(Color::rgb(i as f32 / 100., 0.3, i as f32 / 100.)))
            .collect(),
        dead: materials.add(Color::rgb(0., 0., 0.)),
    };

    cmd.insert_resource(creature_assets);
}

fn spawn_creatures(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    creature_assets: Res<CreatureAssets>,
    config: Res<SimConfig>,
) {
    let mesh_handle = Mesh2dHandle(meshes.add(Circle::new(3.)));
    let mut rng = rand::thread_rng();

    for x in 0..config.creature_spawn_x {
        for y in 0..config.creature_spawn_y {
            let hp = 100.;
            cmd.spawn((
                CreatureMarker,
                MaterialMesh2dBundle {
                    mesh: mesh_handle.clone(),
                    material: creature_assets.alive[hp as usize].clone(),
                    transform: Transform::from_xyz(x as f32 * -10. - 50., y as f32 * -5. - 50., 0.),
                    ..default()
                },
                LifeState::Alive { hp },
                Moving {
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

fn update_creature_destination(
    mut creatures: Query<
        (
            &mut Moving,
            &LifeState,
            &Transform,
            &HungerLevel,
            &SightRange,
        ),
        With<CreatureMarker>,
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

// TODO: move this to movingThing, it's not just for creatures
fn move_creatures(mut creatures: Query<(&mut Moving, &mut Transform)>, map: Res<TileMap>) {
    for (moving, mut transform) in creatures.iter_mut() {
        if let Some(destination) = moving.dest {
            let old_position = Vec2::new(transform.translation.x, transform.translation.y);
            let position_diff = destination - old_position;

            let move_by = if position_diff.length() <= moving.speed {
                position_diff
            } else {
                position_diff.normalize() * moving.speed
            };

            let new_position = old_position + move_by;
            if map.world_pos_in_entities(new_position) {
                transform.translation.x = new_position.x;
                transform.translation.y = new_position.y;
            }
        }
    }
}


fn update_creature_color(
    mut creatures: Query<(&mut Handle<ColorMaterial>, &mut LifeState), With<CreatureMarker>>,
    assets: Res<CreatureAssets>,
) {
    for (mut handle, mut life_state) in creatures.iter_mut() {
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

fn connect_creature_with_medium_its_on(
    creature_transforms: Query<&Transform, With<CreatureMarker>>,
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
