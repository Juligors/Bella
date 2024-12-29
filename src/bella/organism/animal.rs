pub mod gizmos;
pub mod mobile;
pub mod visual;

use crate::bella::{
    config::SimConfig,
    inspector::choose_entity_observer,
    organism::Health,
    pause::PauseState,
    restart::SimState,
    terrain::{
        thermal_conductor::ThermalConductor,
        tile::{Tile, TileLayout},
        BiomeType,
    },
    time::{DayPassedEvent, HourPassedEvent},
};
use bevy::prelude::*;
use gizmos::AnimalGizmosPlugin;
use mobile::{Destination, MobilePlugin};
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
        app.add_plugins((MobilePlugin, AnimalGizmosPlugin))
            .register_type::<HungerLevel>()
            .register_type::<Diet>()
            .register_type::<SightRange>()
            .register_type::<Attack>()
            .add_systems(OnEnter(SimState::LoadAssets), prepare_animal_assets)
            .add_systems(OnEnter(SimState::OrganismGeneration), spawn_animals)
            .add_systems(OnExit(SimState::Simulation), despawn_animals)
            .add_systems(
                Update,
                (
                    update_animal_color,
                    connect_animal_with_medium_its_on,
                    choose_new_destination,
                    decrease_satiation.run_if(on_event::<HourPassedEvent>),
                    consume_energy_to_grow.run_if(on_event::<HourPassedEvent>),
                    consume_energy_to_reproduce.run_if(on_event::<HourPassedEvent>),
                )
                    .run_if(in_state(SimState::Simulation))
                    .run_if(in_state(PauseState::Running)),
            )
            .add_systems(
                Update,
                data_collection::save_animal_data.run_if(on_event::<HourPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct AnimalMarker;

#[derive(Component, Reflect, Debug)]
pub enum HungerLevel {
    Satiated(u32),
    Hungry(u32),
    Starving,
}

#[derive(Component, Reflect, Debug, Clone)]
pub enum Diet {
    Carnivorous(f32),
    Herbivorous(f32),
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
pub struct SightRange(f32);

#[derive(Component, Reflect, Debug)]
pub struct Attack {
    pub range: f32,
    pub damage: f32,
}

fn spawn_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimConfig>,
    tiles: Query<(&BiomeType, &Tile)>,
    tile_layout: Res<TileLayout>,
) {
    let mut rng = rand::thread_rng();

    let base_size = 2.; // FIXME: magic number
    let mesh_handle = meshes.add(Sphere::new(base_size)); // FIXME: magic number

    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile) in tiles.iter() {
        if !rng.gen_bool(config.animal.group_spawn_chance_sand as f64) {
            // TODO: where should animal spawn?
            continue;
        }

        if *biome_type != BiomeType::Sand {
            continue;
        }

        let (pos_min, pos_max) = tile_layout.get_tile_bounds(tile);
        let animal_count =
            rng.gen_range(config.animal.group_size_min..=config.animal.group_size_max);

        for _ in 0..animal_count {
            let hp = 100.; // FIXME: magic number
            let size = Size {
                base_size,
                ratio: rng.gen_range(0.5..2.0), // FIXME: magic number
            };
            let diet = if rng.gen::<f32>() < config.animal.carnivores_to_herbivores_ratio {
                Diet::Carnivorous(1.)
            } else {
                Diet::Herbivorous(1.)
            };
            let mut energy_data = EnergyData {
                energy: 10_000.,                               // FIXME: magic number
                production_efficiency: 0.01,                   // FIXME: magic number
                energy_needed_for_survival_per_mass_unit: 0.1, // FIXME: magic number
                energy_needed_for_growth_per_mass_unit: 5.,    // FIXME: magic number
                grow_by: 0.2,                                  // FIXME: magic number
            };

            // TODO: temp
            if let Diet::Carnivorous(_) = &diet {
                energy_data.production_efficiency /= 10.;
            }

            let x = rng.gen_range(pos_min.x..pos_max.x);
            let y = rng.gen_range(pos_min.y..pos_max.y);

            let entity = commands
                .spawn((
                    AnimalMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(animal_assets.alive[hp as usize].clone()),
                    Transform::from_xyz(x, y, base_size).with_scale(Vec3::splat(size.ratio)),
                    Health { hp },
                    Mobile {
                        speed: rng.gen_range(0.2..0.3), // FIXME: magic number
                        destination: None,
                        next_step_destination: None,
                    },
                    HungerLevel::Hungry(100), // FIXME: magic number
                    SightRange(300.),         // FIXME: magic number
                    Attack {
                        range: 2.,  // FIXME: magic number
                        damage: 3., // FIXME: magic number
                    },
                    size,
                    energy_data,
                    ReproductionState::Developing(rng.gen_range(
                        config.animal.development_time..(config.animal.development_time * 2),
                    )),
                    diet,
                ))
                .id();

            choose_entity_observer.watch_entity(entity);
        }
    }

    commands.spawn(choose_entity_observer);
}

fn despawn_animals(mut commands: Commands, animals: Query<Entity, With<AnimalMarker>>) {
    for animal_entity in animals.iter() {
        commands.entity(animal_entity).despawn_recursive();
    }
}

fn decrease_satiation(mut hunger_levels: Query<(&mut HungerLevel, &mut Health)>) {
    for (mut hunger_level, mut health) in hunger_levels.iter_mut() {
        *hunger_level = match *hunger_level {
            HungerLevel::Satiated(level) => {
                if level > 1 {
                    HungerLevel::Satiated(level - 1) // FIXME: magic number
                } else {
                    HungerLevel::Hungry(100) // FIXME: magic number
                }
            }
            HungerLevel::Hungry(level) => {
                if level > 1 {
                    HungerLevel::Hungry(level - 1) // FIXME: magic number
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
    tile_layout: Res<TileLayout>,
) {
    for creature_transform in creature_transforms.iter() {
        let creature_pos = creature_transform.translation.truncate();
        let entity = tile_layout.get_entity_for_position(creature_pos);

        match entity {
            Some(e) => {
                for (tile_entity, _medium) in tiles.iter() {
                    if tile_entity != e {
                        continue;
                    }
                }
            }
            None => {
                error!("No tile under this creature, pos: {}", creature_pos);
            }
        }
    }
}

pub fn choose_new_destination(
    mut animals: Query<
        (
            Entity,
            &mut Mobile,
            &Transform,
            &HungerLevel,
            &SightRange,
            &Diet,
        ),
        With<AnimalMarker>,
    >,
    plants: Query<(Entity, &Transform), With<PlantMarker>>,
) {
    let plants: Vec<_> = plants.iter().collect();
    let (entities, mut mobiles, transforms, hunger_levels, sight_ranges, diets): (
        Vec<Entity>,
        Vec<Mut<Mobile>>,
        Vec<&Transform>,
        Vec<&HungerLevel>,
        Vec<&SightRange>,
        Vec<&Diet>,
    ) = multiunzip(animals.iter_mut());

    for i in 0..mobiles.len() {
        if mobiles[i].destination.is_some() {
            continue;
        }

        match hunger_levels[i] {
            HungerLevel::Satiated(_) => continue,
            HungerLevel::Hungry(_) | HungerLevel::Starving => {
                mobiles[i].destination = match diets[i] {
                    Diet::Carnivorous(_) => {
                        utils::find_closest_animal(&entities, &transforms, sight_ranges[i], i)
                    }
                    Diet::Herbivorous(_) => {
                        utils::find_closest_plant(transforms[i], sight_ranges[i], &plants)
                    }
                };
            }
        }
    }
}

fn consume_energy_to_grow(
    mut query: Query<
        (
            &mut EnergyData,
            &mut Size,
            &mut Transform,
            &mut ReproductionState,
        ),
        With<AnimalMarker>,
    >,
) {
    for (mut energy_data, mut size, mut transform, mut reproduction_state) in query.iter_mut() {
        match *reproduction_state {
            ReproductionState::ReadyToReproduce => continue,
            ReproductionState::WaitingToReproduce(_) => continue,
            ReproductionState::Developing(time) => {
                let energy_consumed_to_grow =
                    energy_data.energy_needed_for_growth_per_mass_unit * size.real_mass();

                if energy_data.energy < energy_consumed_to_grow {
                    continue;
                }

                *reproduction_state = ReproductionState::Developing(time - 1);
                energy_data.energy -= energy_consumed_to_grow;
                size.ratio += energy_data.grow_by;

                *transform = transform.with_scale(Vec3::splat(size.ratio));
            }
        }
    }
}

fn consume_energy_to_reproduce(
    mut commands: Commands,
    mut query: Query<
        (
            &mut ReproductionState,
            &mut EnergyData,
            &mut Health,
            &Transform,
            &Diet,
        ),
        With<AnimalMarker>,
    >,
    _tile_map: Res<TileLayout>,
    config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
) {
    let mut rng = rand::thread_rng();

    let base_size = 2.; // FIXME: magic number
    let mesh_handle = meshes.add(Sphere::new(base_size)); // FIXME: magic number

    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (mut life_cycle_state, mut energy_data, mut health, transform, diet) in query.iter_mut() {
        match *life_cycle_state {
            ReproductionState::Developing(_) => continue,
            ReproductionState::WaitingToReproduce(cooldown) => {
                *life_cycle_state = ReproductionState::WaitingToReproduce(cooldown - 1);
            }
            ReproductionState::ReadyToReproduce => {
                *life_cycle_state = ReproductionState::WaitingToReproduce(
                    config.plant.waiting_for_reproduction_time,
                );

                // TODO: for now to make plants smaller and die (why are they not get smaller?)
                let by = 2.0;
                energy_data.energy /= by;

                health.hp /= by;
                health.hp -= 1.;

                // TODO: this function should work like this:
                // iterate over neighbouring tiles and check if they are suitable for plant
                // get list of them (including current tile)
                // pick 1 of the tiles at random
                // pawn new plant there

                let old_x = transform.translation.x;
                let old_y = transform.translation.y;

                let range = -50.0..50.0;
                let offset_x = rng.gen_range(range.clone());
                let offset_y = rng.gen_range(range);

                let new_x = old_x + offset_x;
                let new_y = old_y + offset_y;

                // TODO: this is copied from base spawn function, should create separate function for creating default plant (and organism as well)
                let hp = 100.;
                let size = Size {
                    base_size,
                    ratio: rng.gen_range(0.5..2.0),
                };
                let energy_data = EnergyData {
                    energy: 1000.,
                    production_efficiency: 0.01,
                    energy_needed_for_survival_per_mass_unit: 5.,
                    energy_needed_for_growth_per_mass_unit: 1.,
                    grow_by: 0.2,
                };

                // TODO: copied from setup spawning (BUT CHANGED!!), maybe can be avoided a little with RequiredComponents and default values generated with default rng?
                let entity = commands
                    .spawn((
                        AnimalMarker,
                        Mesh3d(mesh_handle.clone()),
                        MeshMaterial3d(animal_assets.alive[hp as usize].clone()),
                        Transform::from_xyz(new_x, new_y, base_size)
                            .with_scale(Vec3::splat(size.ratio)),
                        Health { hp },
                        Mobile {
                            speed: rng.gen_range(0.2..0.3), // FIXME: magic number
                            destination: None,
                            next_step_destination: None,
                        },
                        HungerLevel::Hungry(100), // FIXME: magic number
                        SightRange(300.),         // FIXME: magic number
                        Attack {
                            range: 2.,  // FIXME: magic number
                            damage: 3., // FIXME: magic number
                        },
                        size,
                        energy_data,
                        ReproductionState::Developing(rng.gen_range(
                            config.animal.development_time..(config.animal.development_time * 2),
                        )),
                        diet.clone(),
                    ))
                    .id();

                choose_entity_observer.watch_entity(entity);
            }
        }
    }

    commands.spawn(choose_entity_observer);
}

mod utils {
    use super::*;

    pub fn find_closest_animal(
        animal_entities: &[Entity],
        animal_transforms: &[&Transform],
        sight_range: &SightRange,
        i: usize,
    ) -> Option<Destination> {
        let distances_to_other_animals = animal_entities
            .iter()
            .zip(animal_transforms)
            .enumerate()
            .filter(|(j, _)| i != *j)
            .map(|(_, (&other_animal_entity, &other_animal_transform))| {
                let other_aminal_pos = other_animal_transform.translation.truncate();
                let animal_pos = animal_transforms[i].translation.truncate();

                (other_animal_entity, animal_pos.distance(other_aminal_pos))
            });

        get_closest_visible(distances_to_other_animals, sight_range)
    }

    pub fn find_closest_plant(
        animal_transform: &Transform,
        sight_range: &SightRange,
        plants: &[(Entity, &Transform)],
    ) -> Option<Destination> {
        let distances_to_plants = plants.iter().map(|(plant_entity, &plant_transform)| {
            let plant_pos = plant_transform.translation.truncate();
            let animal_pos = animal_transform.translation.truncate();

            (*plant_entity, animal_pos.distance(plant_pos))
        });

        get_closest_visible(distances_to_plants, sight_range)
    }

    fn get_closest_visible<I>(
        positions_and_distances: I,
        sight_range: &SightRange,
    ) -> Option<Destination>
    where
        I: Iterator<Item = (Entity, f32)>,
    {
        positions_and_distances
            .filter(|(_, distance)| *distance < **sight_range)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| Destination::Organism { entity })
    }
}

mod data_collection {
    use super::*;
    use crate::bella::{data_collection::DataCollectionDirectory, time::SimTime};

    #[derive(Debug, serde::Serialize)]
    pub struct Animal {
        pub id: u64,
        pub hour: u32,
        pub day: u32,

        pub is_herbivorous: bool,

        pub health: f32,
        pub base_size: f32,
        pub ratio: f32,

        pub energy: f32,
        pub production_efficiency: f32,
        pub energy_needed_for_survival_per_mass_unit: f32,
        pub energy_needed_for_growth_per_mass_unit: f32,
        pub grow_by: f32,
    }

    pub fn save_animal_data(
        animals: Query<(Entity, &Health, &Size, &EnergyData, &Diet), With<AnimalMarker>>,
        path: Res<DataCollectionDirectory>,
        time: Res<SimTime>,
    ) {
        let path = path.0.join("animals.csv");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(path.exists())
            // .truncate(true)
            .create(true)
            .open(path)
            .unwrap();

        let needs_headers = std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0)).unwrap() == 0;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'|')
            .has_headers(needs_headers)
            .from_writer(file);

        for x in animals.iter() {
            let animal_record = Animal {
                id: x.0.to_bits(),

                hour: time.hours,
                day: time.days,

                is_herbivorous: matches!(x.4, Diet::Herbivorous(_)),

                health: x.1.hp,
                base_size: x.2.base_size,
                ratio: x.2.ratio,
                energy: x.3.energy,
                production_efficiency: x.3.production_efficiency,
                energy_needed_for_survival_per_mass_unit: x
                    .3
                    .energy_needed_for_survival_per_mass_unit,
                energy_needed_for_growth_per_mass_unit: x.3.energy_needed_for_growth_per_mass_unit,
                grow_by: x.3.grow_by,
            };

            writer
                .serialize(&animal_record)
                .unwrap_or_else(|_| panic!("Couldn't serialize object {:?}", animal_record));
        }

        writer
            .flush()
            .expect("Couldn't save new animal data to a file");
    }
}
