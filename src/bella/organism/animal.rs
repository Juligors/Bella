pub mod animal_ai;
#[cfg(not(feature = "bella_headless"))]
pub mod gizmos;

use std::cell::RefCell;

use self::animal_ai::Mobile;
use super::{
    gene::FloatGene, Age, BasicBundle, EnergyData, OrganismBundle, OrganismEnergyEfficiency,
    SexualMaturity,
};
use crate::bella::{
    config::SimulationConfig,
    organism::Health,
    restart::SimulationState,
    terrain::{
        tile::{Tile, TileLayout},
        BiomeType, ObjectsInTile,
    },
    time::TimeUnitPassedEvent,
    ui_facade::choose_entity_observer,
};
use animal_ai::{Action, AnimalAiPlugin};
use bevy::prelude::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimalAiPlugin)
            .register_type::<Diet>()
            .register_type::<ActionRange>()
            .register_type::<SightRange>()
            .register_type::<AttackDmg>()
            .add_event::<ReproduceAnimalsEvent>()
            .add_systems(OnEnter(SimulationState::LoadAssets), prepare_animal_assets)
            .add_systems(OnEnter(SimulationState::OrganismGeneration), spawn_animals)
            .add_systems(OnExit(SimulationState::Simulation), despawn_all_animals)
            .add_systems(
                Update,
                reproduce.run_if(in_state(SimulationState::Simulation)),
            );
    }
}

#[derive(Component)]
pub struct AnimalMarker;

#[derive(Component)]
pub struct AnimalMatterMarker;

#[derive(Bundle)]
pub struct AnimalBundle {
    organism_bundle: OrganismBundle,
    marker: AnimalMarker,
    matter_marker: AnimalMatterMarker,
    animal_energy_efficiency: AnimalEnergyEfficiency,
    action_range: ActionRange,
    mobile: Mobile,
    attack: AttackDmg,
    sight_range: SightRange,
    diet: Diet,
    action: Action,
}

#[derive(Event)]
pub struct ReproduceAnimalsEvent {
    pub parent1: Entity,
    pub parent2: Entity,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct AnimalEnergyEfficiency {
    // pub energy_consumption_to_survive_per_mass_unit_gene: FloatGene,
}

impl AnimalEnergyEfficiency {
    pub fn new() -> Self {
        Self {
            // production_from_solar_gene: production_from_solar_gene.into(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub enum Diet {
    Herbivore,
    Carnivore,
    Omnivore,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct SightRange {
    gene: FloatGene,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct ActionRange {
    pub gene: FloatGene,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct AttackDmg {
    pub gene: FloatGene,
}

#[derive(Resource)]
pub struct AnimalAssets {
    pub carnivore: Handle<StandardMaterial>,
    pub herbivore: Handle<StandardMaterial>,
    pub omnivore: Handle<StandardMaterial>,
}

pub fn prepare_animal_assets(mut cmd: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let animal_assets = AnimalAssets {
        carnivore: materials.add(Color::srgb(1.0, 0.3, 0.3)),
        herbivore: materials.add(Color::srgb(0.3, 1.0, 0.7)),
        omnivore: materials.add(Color::srgb(0.3, 0.3, 1.0)),
    };

    cmd.insert_resource(animal_assets);
}

fn spawn_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    animal_assets: Res<AnimalAssets>,
    config: Res<SimulationConfig>,
    mut tiles: Query<(&BiomeType, &Tile, &mut ObjectsInTile)>,
    tile_layout: Res<TileLayout>,
) {
    let mesh_handle = meshes.add(Sphere::new(1.0));
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for (biome_type, tile, mut objects_in_tile) in tiles.iter_mut() {
        if !biome_type.animals_can_live_here() {
            continue;
        }

        if !config.animal.group_spawn_chance.happened() {
            continue;
        }

        let animal_count = config.animal.group_size_dist.sample();

        let diet = match config.animal.diet_dist.sample() {
            0 => Diet::Herbivore,
            1 => Diet::Carnivore,
            _ => Diet::Omnivore,
        };

        for _ in 0..animal_count {
            let health = Health::new(config.organism.max_health_gene_config.into());
            let starting_age = config.organism.starting_age_dist.sample();
            let age = Age::new(starting_age, config.organism.age_penalty_gene_config.into());
            let sexual_maturity = SexualMaturity::new(
                config
                    .animal_species
                    .get(&diet)
                    .maturity_age_gene_config
                    .into(),
                config
                    .animal_species
                    .get(&diet)
                    .reproduction_cooldown_gene_config
                    .into(),
                starting_age,
            );
            let energy_data = EnergyData::new(
                config.organism.max_active_energy_gene_config.into(),
                config.organism.max_active_energy_gene_config.into(),
                config.organism.starting_mass_dist.sample(),
            );
            let organism_energy_efficiency = OrganismEnergyEfficiency::new(
                config
                    .animal_species
                    .get(&diet)
                    .energy_to_survive_per_mass_unit_gene_config
                    .into(),
                config.organism.reproduction_energy_cost_gene_config.into(),
            );

            let animal_energy_efficiency = AnimalEnergyEfficiency::new();
            let mobile = Mobile {
                speed: config.animal_species.get(&diet).speed_gene_config.into(),
                destination: None,
                next_step_destination: None,
            };
            let sight_range = SightRange {
                gene: config
                    .animal_species
                    .get(&diet)
                    .sight_range_gene_config
                    .into(),
            };
            let action_range = ActionRange {
                gene: config
                    .animal_species
                    .get(&diet)
                    .action_range_gene_config
                    .into(),
            };
            let attack = AttackDmg {
                gene: config
                    .animal_species
                    .get(&diet)
                    .attack_damage_gene_config
                    .into(),
            };
            let action = Action::DoingNothing { for_hours: 0 };
            let size = energy_data.get_size();
            let position = tile_layout.get_random_position_in_tile(tile);

            let entity = commands
                .spawn((
                    BasicBundle {
                        mesh: Mesh3d(mesh_handle.clone()),
                        material: MeshMaterial3d(get_animal_asset(&animal_assets, &diet)),
                        transform: Transform::from_translation(position.extend(size / 2.0))
                            .with_scale(Vec3::splat(size)),
                    },
                    AnimalBundle {
                        organism_bundle: OrganismBundle {
                            health,
                            age,
                            sexual_maturity,
                            energy_data,
                            organism_energy_efficiency,
                        },
                        marker: AnimalMarker,
                        matter_marker: AnimalMatterMarker,
                        animal_energy_efficiency,
                        action_range,
                        mobile,
                        attack,
                        sight_range,
                        diet: diet.clone(),
                        action,
                    },
                ))
                .id();
            objects_in_tile.add_animal_entity(entity);
            choose_entity_observer.watch_entity(entity);
        }
    }

    commands.spawn(choose_entity_observer);
}

fn despawn_all_animals(mut commands: Commands, animals: Query<Entity, With<AnimalMarker>>) {
    for animal_entity in animals.iter() {
        commands.entity(animal_entity).despawn_recursive();
    }
}

fn reproduce(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    tile_layout: Res<TileLayout>,
    animal_assets: Res<AnimalAssets>,
    animal_query: Query<
        (
            &Mesh3d,
            &Transform,
            &Health,
            &Age,
            &mut SexualMaturity,
            &EnergyData,
            &OrganismEnergyEfficiency,
            &AnimalEnergyEfficiency,
            &ActionRange,
            &Mobile,
            &AttackDmg,
            &SightRange,
            &Diet,
        ),
        With<AnimalMarker>,
    >,
    mut tiles: Query<(&BiomeType, &mut ObjectsInTile)>,
    mut event_reader: EventReader<ReproduceAnimalsEvent>,
) {
    let mut choose_entity_observer = Observer::new(choose_entity_observer);

    'outer: for event in event_reader.read() {
        let Ok((
            mesh1,
            transform1,
            health1,
            age1,
            sexual_maturity1,
            energy_data1,
            organism_energy_efficiency1,
            animal_energy_efficiency1,
            action_range1,
            mobile1,
            attack1,
            sight_range1,
            diet1,
        )) = animal_query.get(event.parent1)
        else {
            continue;
        };

        let Ok((
            _,
            transform2,
            health2,
            age2,
            sexual_maturity2,
            energy_data2,
            organism_energy_efficiency2,
            animal_energy_efficiency2,
            action_range2,
            mobile2,
            attack2,
            sight_range2,
            diet2,
        )) = animal_query.get(event.parent2)
        else {
            continue;
        };

        let starting_age = 0;
        let diet = diet1.clone();
        let animal_energy_efficiency = AnimalEnergyEfficiency::new();
        let energy_data = EnergyData::new(
            energy_data1
                .max_active_energy_gene
                .mixed_with(&energy_data2.max_active_energy_gene),
            energy_data1
                .energy_per_mass_unit_gene
                .mixed_with(&energy_data2.energy_per_mass_unit_gene),
            config.organism.starting_mass_dist.sample(),
        );

        // NOTE: position of a random parent "birthing", shouldn't be needed once we introduce sex
        let point = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            if rng.gen_bool(0.5) {
                transform1.translation.truncate()
            } else {
                transform2.translation.truncate()
            }
        });

        // NOTE: if animal is on the terrain it can't live on, we just don't spawn new animal.
        let new_animal_position = tile_layout.get_random_position_in_ring(
            point,
            config.organism.offspring_spawn_range,
            config.organism.offspring_spawn_range / 2.0,
        );
        let entity_of_tile_under = tile_layout.get_tile_entity_for_position(new_animal_position);
        let (biome_under_new_animal, mut objects_in_tile) = tiles
            .get_mut(entity_of_tile_under)
            .expect("Failed to get tile components of tile for new animal position");
        if !biome_under_new_animal.animals_can_live_here() {
            continue 'outer;
        }

        let new_size = energy_data.get_size();
        let mut transform =
            Transform::from_translation(new_animal_position.extend(energy_data.get_size() / 2.0))
                .with_scale(Vec3::splat(new_size));
        transform.translation.z = new_size / 2.0;

        let new_entity = commands
            .spawn((
                BasicBundle {
                    mesh: mesh1.clone(),
                    material: MeshMaterial3d(get_animal_asset(&animal_assets, &diet)),
                    transform,
                },
                AnimalBundle {
                    organism_bundle: OrganismBundle {
                        health: Health::new(health1.max_hp_gene.mixed_with(&health2.max_hp_gene)),
                        age: Age::new(
                            starting_age,
                            age1.age_penalty_gene.mixed_with(&age2.age_penalty_gene),
                        ),
                        sexual_maturity: SexualMaturity::new(
                            sexual_maturity1
                                .maturity_age_gene
                                .mixed_with(&sexual_maturity2.maturity_age_gene),
                            sexual_maturity1
                                .reproduction_cooldown_gene
                                .mixed_with(&sexual_maturity2.reproduction_cooldown_gene),
                            starting_age,
                        ),
                        energy_data,
                        organism_energy_efficiency: OrganismEnergyEfficiency::new(
                            organism_energy_efficiency1
                                .energy_consumption_to_survive_per_mass_unit_gene
                                .mixed_with(
                                    &organism_energy_efficiency2
                                        .energy_consumption_to_survive_per_mass_unit_gene,
                                ),
                            organism_energy_efficiency1
                                .reproduction_energy_cost_gene
                                .mixed_with(
                                    &organism_energy_efficiency2.reproduction_energy_cost_gene,
                                ),
                        ),
                    },
                    marker: AnimalMarker,
                    matter_marker: AnimalMatterMarker,
                    animal_energy_efficiency,
                    mobile: Mobile {
                        speed: mobile1.speed.mixed_with(&mobile2.speed),
                        destination: None,
                        next_step_destination: None,
                    },
                    action_range: ActionRange {
                        gene: action_range1.gene.mixed_with(&action_range2.gene),
                    },
                    sight_range: SightRange {
                        gene: sight_range1.gene.mixed_with(&sight_range2.gene),
                    },
                    attack: AttackDmg {
                        gene: attack1.gene.mixed_with(&attack2.gene),
                    },
                    // TODO: diet should also be a gene
                    diet,
                    action: Action::DoingNothing { for_hours: 0 },
                },
            ))
            .id();
        objects_in_tile.add_animal_entity(new_entity);
        choose_entity_observer.watch_entity(new_entity);
    }
}

fn get_animal_asset(assets: &AnimalAssets, diet: &Diet) -> Handle<StandardMaterial> {
    match &diet {
        Diet::Carnivore => assets.carnivore.clone(),
        Diet::Herbivore => assets.herbivore.clone(),
        Diet::Omnivore => assets.omnivore.clone(),
    }
}
