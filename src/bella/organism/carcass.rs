use super::{
    animal::{AnimalBundle, AnimalMarker, AnimalMatterMarker},
    plant::{PlantBundle, PlantMarker, PlantMatterMarker},
    Energy, EnergyData, Health, KillOrganismEvent,
};
use crate::bella::{
    config::SimulationConfig,
    pause::PauseState,
    restart::SimulationState,
    terrain::{tile::TileLayout, ObjectsInTile},
    time::TimeUnitPassedEvent,
};
use bevy::prelude::*;

pub struct CarcassPlugin;

impl Plugin for CarcassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Carcass>()
            .add_systems(Startup, prepare_assets)
            .add_systems(OnExit(SimulationState::Simulation), despawn_carcasses)
            .add_systems(
                Update,
                (check_if_organisms_should_die, destoy_carcasses_if_needed)
                    .run_if(in_state(PauseState::Running))
                    .run_if(in_state(SimulationState::Simulation)),
            )
            .add_systems(
                Update,
                (decay_carcasses).run_if(on_event::<TimeUnitPassedEvent>),
            )
            .add_systems(
                PostUpdate,
                transform_dead_organisms_into_carcasses
                    .run_if(in_state(PauseState::Running))
                    .run_if(in_state(SimulationState::Simulation)),
            );
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Carcass {
    pub mass: f32,
    pub starting_mass: f32,
    pub energy_per_mass_unit: Energy,
}

#[derive(Resource)]
pub struct CarcassAssets {
    pub carcass: Handle<StandardMaterial>,
}

fn prepare_assets(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(CarcassAssets {
        carcass: materials.add(Color::BLACK),
    });
}

fn despawn_carcasses(mut commands: Commands, query: Query<Entity, With<Carcass>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn check_if_organisms_should_die(
    query: Query<(Entity, &Health)>,
    mut ew: EventWriter<KillOrganismEvent>,
) {
    for (entity, health) in query.iter() {
        if health.hp <= 0.0 {
            ew.send(KillOrganismEvent { entity });
        }
    }
}

fn transform_dead_organisms_into_carcasses(
    mut commands: Commands,
    mut event_reader: EventReader<KillOrganismEvent>,
    query: Query<(
        Entity,
        &Transform,
        &EnergyData,
        Option<&AnimalMarker>,
        Option<&PlantMarker>,
    )>,
    assets: Res<CarcassAssets>,
    mut objects_in_tile_query: Query<&mut ObjectsInTile>,
    tile_layout: Res<TileLayout>,
) {
    for event in event_reader.read() {
        if let Ok((entity, transform, energy_data, maybe_animal, maybe_plant)) =
            query.get(event.entity)
        {
            let mut entity_commands = commands.entity(entity);
            let tile_entity = tile_layout.get_tile_entity_for_transform(transform);
            let mut objects = objects_in_tile_query
                .get_mut(tile_entity)
                .expect("Failed to get objects in tile while transforming into carcasses");

            objects.remove_any_entity(entity);

            if maybe_animal.is_some() {
                entity_commands.remove::<AnimalBundle>();
                entity_commands.insert(AnimalMatterMarker);
                objects.add_animal_carcass_entity(entity);
            }

            if maybe_plant.is_some() {
                entity_commands.remove::<PlantBundle>();
                entity_commands.insert(PlantMatterMarker);
                objects.add_plant_carcass_entity(entity);
            }

            entity_commands.insert((
                MeshMaterial3d(assets.carcass.clone()),
                Carcass {
                    mass: energy_data.mass,
                    starting_mass: energy_data.mass,
                    energy_per_mass_unit: energy_data.energy_per_mass_unit_gene.phenotype(),
                },
            ));
        };
    }
}

fn decay_carcasses(mut carcasses: Query<&mut Carcass>, config: Res<SimulationConfig>) {
    for mut carcass in carcasses.iter_mut() {
        carcass.mass -= f32::max(
            0.0,
            carcass.starting_mass * config.organism.carcass_mass_decay_percentage,
        );
    }
}

fn destoy_carcasses_if_needed(
    mut commands: Commands,
    carcasses: Query<(Entity, &Carcass, &Transform)>,
    tile_layout: Res<TileLayout>,
    mut objects_in_tile_query: Query<&mut ObjectsInTile>,
) {
    for (carcass_entity, carcass, carcass_transform) in carcasses.iter() {
        if carcass.mass <= 0.0 {
            let tile_entity = tile_layout.get_tile_entity_for_transform(carcass_transform);
            objects_in_tile_query
                .get_mut(tile_entity)
                .expect("Failed to get tile under carcass")
                .remove_any_entity(carcass_entity);
            commands.entity(carcass_entity).despawn_recursive();
        }
    }
}
