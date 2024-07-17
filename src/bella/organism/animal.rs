pub mod mobile;
pub mod visual;

use crate::bella::{
    config::SimConfig,
    organism::Health,
    system_set::InitializationSet,
    terrain::{biome::BiomeType, thermal_conductor::ThermalConductor, TerrainPosition, TileMap},
    time::HourPassedEvent,
    ui::layer::SpriteLayer,
};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use mobile::MobilePlugin;
use rand::{self, Rng};

use self::{
    mobile::Mobile,
    visual::{prepare_animal_assets, update_animal_color, AnimalAssets},
};
use itertools::multiunzip;

use super::{plant::PlantMarker, EnergyData, ReproductionState, Size};

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MobilePlugin)
            .add_systems(
                Startup,
                (
                    prepare_animal_assets,
                    spawn_animals.in_set(InitializationSet::OrganismSpawning),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    update_animal_color,
                    connect_animal_with_medium_its_on,
                    decrease_satiation.run_if(on_event::<HourPassedEvent>()),
                    choose_new_destination,
                ),
            );
    }
}

#[derive(Component)]
pub struct AnimalMarker;

#[derive(Component)]
pub enum HungerLevel {
    Satiated(u32),
    Hungry(u32),
    Starving,
}

#[derive(Component)]
pub enum Diet {
    Carnivorous(f32),
    Herbivorous(f32),
}

#[derive(Component, Deref, DerefMut)]
pub struct SightRange(f32);

fn spawn_animals(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimConfig>,
    tiles: Query<(&BiomeType, &TerrainPosition)>,
    tile_map: Res<TileMap>,
) {
    let mut rng = rand::thread_rng();

    let base_size = 3.;
    let mesh_handle = Mesh2dHandle(meshes.add(Circle::new(3.)));

    for (biome_type, terrain_position) in tiles.iter() {
        if !rng.gen_bool(config.animal.group_spawn_chance_sand as f64) {
            // TODO: where should animal spawn?
            continue;
        }

        if *biome_type != BiomeType::Sand {
            continue;
        }

        let group_middle_pos = tile_map.layout.hex_to_world_pos(terrain_position.hex_pos);
        let plant_count =
            rng.gen_range(config.animal.group_size_min..=config.animal.group_size_max);

        for _ in 0..plant_count {
            let hp = 100.;
            let size = Size {
                base_size,
                ratio: rng.gen_range(0.5..2.0),
            };
            let energy_data = EnergyData {
                energy: 1000.,
                production_efficiency: 0.01,
                energy_needed_for_survival_per_mass_unit: 5.,
                energy_needed_for_growth_per_mass_unit: 50.,
                grow_by: 0.2,
            };

            // algorithm taken from: https://stackoverflow.com/questions/3239611/generating-random-points-within-a-hexagon-for-procedural-game-content
            let sqrt3 = 3.0f32.sqrt();
            let vectors = [(-1., 0.), (0.5, sqrt3 / 2.), (0.5, -sqrt3 / 2.)];

            let index = rng.gen_range(0..=2);
            let vector_x = vectors[index];
            let vector_y = vectors[(index + 1) % 3];

            let (base_x, base_y) = rng.gen::<(f32, f32)>();
            let x_offset = base_x * vector_x.0 + base_y * vector_y.0;
            let y_offset = base_x * vector_x.1 + base_y * vector_y.1;

            let x = group_middle_pos.x + x_offset * config.terrain.hex_size;
            let y = group_middle_pos.y + y_offset * config.terrain.hex_size;

            let mut new_animal = cmd.spawn((
                AnimalMarker,
                SpriteLayer::Creature,
                MaterialMesh2dBundle {
                    mesh: mesh_handle.clone(),
                    material: animal_assets.alive[hp as usize].clone(),
                    transform: Transform::from_xyz(x, y, 1.).with_scale(Vec3::splat(size.ratio)),
                    ..default()
                },
                Health { hp },
                Mobile {
                    dest: None,
                    speed: rng.gen_range(0.2..0.3),
                },
                HungerLevel::Hungry(100),
                SightRange(500.),
                size,
                energy_data,
                ReproductionState::Developing(config.animal.development_time),
            ));

            new_animal.insert(
                if rng.gen::<f32>() < config.animal.carnivores_to_herbivores_ratio {
                    Diet::Carnivorous(1.)
                } else {
                    Diet::Herbivorous(1.)
                },
            );
        }
    }
}

fn decrease_satiation(mut hunger_levels: Query<(&mut HungerLevel, &mut Health)>) {
    for (mut hunger_level, mut health) in hunger_levels.iter_mut() {
        *hunger_level = match *hunger_level {
            HungerLevel::Satiated(level) => {
                if level > 1 {
                    HungerLevel::Satiated(level - 10) // FIXME: magic number
                } else {
                    HungerLevel::Hungry(100) // FIXME: magic number
                }
            }
            HungerLevel::Hungry(level) => {
                if level > 1 {
                    HungerLevel::Hungry(level - 20) // FIXME: magic number
                } else {
                    HungerLevel::Starving
                }
            }
            HungerLevel::Starving => {
                health.hp -= 10.; // FIXME: magic number
                HungerLevel::Starving
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
                }
            }
            None => error!("No tile under this creature :("),
        }
    }
}

pub fn choose_new_destination(
    mut animals: Query<
        (&mut Mobile, &Transform, &HungerLevel, &SightRange, &Diet),
        With<AnimalMarker>,
    >,
    plants: Query<&Transform, With<PlantMarker>>,
) {
    let plants: Vec<&Transform> = plants.iter().collect();
    let (mut mobiles, transforms, hunger_levels, sight_ranges, diets): (
        Vec<Mut<Mobile>>,
        Vec<&Transform>,
        Vec<&HungerLevel>,
        Vec<&SightRange>,
        Vec<&Diet>,
    ) = multiunzip(animals.iter_mut());

    // TODO: This has to be changed into IDestination, can't just use positions (or update super often)

    for i in 0..mobiles.len() {
        match hunger_levels[i] {
            HungerLevel::Satiated(_) => continue,
            HungerLevel::Hungry(_) | HungerLevel::Starving => {
                let new_destination = match diets[i] {
                    Diet::Carnivorous(_) => {
                        helpers::find_closest_animal(&transforms, sight_ranges[i], i)
                    }
                    Diet::Herbivorous(_) => {
                        helpers::find_closest_plant(transforms[i], sight_ranges[i], &plants)
                    }
                };

                mobiles[i].dest = new_destination.map(|(position, _)| position)
            }
        }
    }
}

mod helpers {
    use super::*;

    pub fn find_closest_animal(
        animal_transforms: &[&Transform],
        sight_range: &SightRange,
        i: usize,
    ) -> Option<(Vec2, f32)> {
        let distances_to_other_animals = animal_transforms
            .iter()
            .enumerate()
            .filter(|(j, _)| i != *j)
            .map(|(_, &other_animal_transform)| {
                let other_aminal_pos = other_animal_transform.translation.truncate();
                let animal_pos = animal_transforms[i].translation.truncate();

                (other_aminal_pos, animal_pos.distance(other_aminal_pos))
            });

        get_closest_visible(distances_to_other_animals, sight_range)
    }

    pub fn find_closest_plant(
        animal_transform: &Transform,
        sight_range: &SightRange,
        plants: &[&Transform],
    ) -> Option<(Vec2, f32)> {
        let distances_to_plants = plants.iter().map(|&plant_transform| {
            let plant_pos = plant_transform.translation.truncate();
            let animal_pos = animal_transform.translation.truncate();

            (plant_pos, animal_pos.distance(plant_pos))
        });

        get_closest_visible(distances_to_plants, sight_range)
    }

    fn get_closest_visible<I>(
        positions_and_distances: I,
        sight_range: &SightRange,
    ) -> Option<(Vec2, f32)>
    where
        I: Iterator<Item = (Vec2, f32)>,
    {
        positions_and_distances
            .filter(|(_, distance)| *distance < **sight_range)
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}
