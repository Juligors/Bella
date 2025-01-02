pub mod gizmos;
pub mod mobile;

use self::mobile::Mobile;
use super::{plant::PlantMarker, EnergyData, ReproductionState, Size};
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
use itertools::multiunzip;
use mobile::{Destination, MobilePlugin};

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
    Carnivorous,
    Herbivorous,
    Omnivore,
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
pub struct SightRange(f32);

#[derive(Component, Reflect, Debug)]
pub struct Attack {
    pub range: f32,
    pub damage: f32,
}

#[derive(Resource)]
pub struct AnimalAssets {
    pub carnivorous: Handle<StandardMaterial>,
    pub herbivorous: Handle<StandardMaterial>,
    pub omnivore: Handle<StandardMaterial>,
}

pub fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let animal_assets = AnimalAssets {
        carnivorous: materials.add(Color::srgb(1.0, 0.3, 0.3)),
        herbivorous: materials.add(Color::srgb(0.3, 1.0, 0.7)),
        omnivore: materials.add(Color::srgb(0.3, 0.3, 1.0)),
    };

    cmd.insert_resource(animal_assets);
}

fn spawn_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimConfig>,
    tiles: Query<(&BiomeType, &Tile)>,
    tile_layout: Res<TileLayout>,
) {
    let mesh_handle = meshes.add(Sphere::new(1.0));
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile) in tiles.iter() {
        if *biome_type != BiomeType::Sand {
            continue;
        }

        if !config.animal.group_spawn_on_sand_chance.happened() {
            continue;
        }

        let size = config.animal.size_dist.sample();
        let animal_count = config.animal.group_size_dist.sample();

        for _ in 0..animal_count {
            let energy_data = EnergyData {
                energy: 10_000.,                               // FIXME: magic number
                production_efficiency: 0.01,                   // FIXME: magic number
                energy_needed_for_survival_per_mass_unit: 0.1, // FIXME: magic number
                energy_needed_for_growth_per_mass_unit: 5.,    // FIXME: magic number
                grow_by: 0.2,                                  // FIXME: magic number
            };

            let max_hp = config.animal.max_health_dist.sample();
            let health = Health {
                max: max_hp,
                hp: max_hp / 2.0,
            };
            let diet = match config.animal.diet_dist.sample() {
                0 => Diet::Herbivorous,
                1 => Diet::Carnivorous,
                _ => Diet::Omnivore,
            };

            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn((
                    AnimalMarker,
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(get_animal_asset(&animal_assets, &diet)),
                    Transform::from_translation(position.extend(size))
                        .with_scale(Vec3::splat(size)),
                    Mobile {
                        speed: config.animal.speed_dist.sample(),
                        destination: None,
                        next_step_destination: None,
                    },
                    HungerLevel::Hungry(100), // FIXME: magic number
                    SightRange(config.animal.sight_range_dist.sample()),
                    Attack {
                        range: config.animal.attack_range_dist.sample(),
                        damage: config.animal.attack_damage_dist.sample(),
                    },
                    Size { size },
                    ReproductionState::Developing(config.animal.development_time), // TODO: probably need to fix that?
                    health,
                    energy_data,
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
                    Diet::Carnivorous => {
                        utils::find_closest_animal(&entities, &transforms, sight_ranges[i], i)
                    }
                    Diet::Herbivorous => {
                        utils::find_closest_plant(transforms[i], sight_ranges[i], &plants)
                    }
                    Diet::Omnivore => {
                        // TODO: fix that
                        // let animal = utils::find_closest_animal(&entities, &transforms, sight_ranges[i], i)
                        // let plant = utils::find_closest_plant(transforms[i], sight_ranges[i], &plants)

                        utils::find_closest_animal(&entities, &transforms, sight_ranges[i], i)
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
                    energy_data.energy_needed_for_growth_per_mass_unit * size.real_volume();

                if energy_data.energy < energy_consumed_to_grow {
                    continue;
                }

                *reproduction_state = ReproductionState::Developing(time - 1);
                energy_data.energy -= energy_consumed_to_grow;
                size.size += energy_data.grow_by;

                *transform = transform.with_scale(Vec3::splat(size.size));
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
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Size,
            &Attack,
            &SightRange,
            &Mobile,
        ),
        With<AnimalMarker>,
    >,
    config: Res<SimConfig>,
    tile_layout: Res<TileLayout>,
) {
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (
        mut parent_life_cycle_state,
        mut parent_energy_data,
        mut parent_health,
        parent_transform,
        parent_diet,
        parent_mesh,
        parent_material,
        parent_size,
        parent_attack,
        parent_sight_range,
        parent_mobile,
    ) in query.iter_mut()
    {
        // TODO: this should happen somewhere else and emit ReproduceEvent with Entity being parent
        match *parent_life_cycle_state {
            ReproductionState::Developing(_) => continue,
            ReproductionState::WaitingToReproduce(cooldown) => {
                *parent_life_cycle_state = ReproductionState::WaitingToReproduce(cooldown - 1);
            }
            ReproductionState::ReadyToReproduce => {
                *parent_life_cycle_state = ReproductionState::WaitingToReproduce(
                    config.plant.waiting_for_reproduction_time,
                );

                let size = Size {
                    size: parent_size.size,
                };
                let health = Health {
                    max: parent_health.max,
                    hp: parent_health.max / 2.0,
                };
                let mobile = Mobile {
                    speed: parent_mobile.speed,
                    destination: None,
                    next_step_destination: None,
                };

                let energy_data = EnergyData {
                    energy: 1000.,
                    production_efficiency: 0.01,
                    energy_needed_for_survival_per_mass_unit: 5.,
                    energy_needed_for_growth_per_mass_unit: 5.,
                    grow_by: 0.2,
                };
                let attack = Attack {
                    damage: parent_attack.damage,
                    range: parent_attack.range,
                };
                let sight_range = SightRange(parent_sight_range.0);
                let diet = (parent_diet).clone();

                let position = tile_layout.get_random_position_in_range(
                    parent_transform.translation.truncate(),
                    config.animal.reproduction_range,
                );

                let entity = commands
                    .spawn((
                        AnimalMarker,
                        Transform::from_translation(position.extend(size.size))
                            .with_scale(Vec3::splat(size.size)),
                        HungerLevel::Hungry(100), // FIXME: magic numbetruct
                        ReproductionState::Developing(config.animal.development_time), // TODO: probably need to fix that?
                        parent_mesh.clone(),
                        parent_material.clone(),
                        attack,
                        sight_range,
                        diet,
                        health,
                        mobile,
                        size,
                        energy_data,
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
        pub size: f32,

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

                is_herbivorous: matches!(x.4, Diet::Herbivorous),

                health: x.1.hp,
                size: x.2.size,
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

fn get_animal_asset(assets: &AnimalAssets, diet: &Diet) -> Handle<StandardMaterial> {
    match &diet {
        Diet::Carnivorous => assets.carnivorous.clone(),
        Diet::Herbivorous => assets.herbivorous.clone(),
        Diet::Omnivore => assets.omnivore.clone(),
    }
}
