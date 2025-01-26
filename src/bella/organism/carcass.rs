use super::{
    animal::AnimalMarker, plant::PlantMarker, Energy, EnergyDatav3, Health, KillOrganismEvent,
};
use crate::bella::{config::SimConfig, pause::PauseState, time::HourPassedEvent};
use bevy::prelude::*;

pub struct CarcassPlugin;

impl Plugin for CarcassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Carcass>()
            .add_systems(Startup, prepare_assets)
            .add_systems(
                Update,
                check_if_organisms_should_die.run_if(in_state(PauseState::Running)),
            )
            .add_systems(
                Update,
                (decay_and_destoy_carcasses_if_needed).run_if(on_event::<HourPassedEvent>),
            )
            .add_systems(
                PostUpdate,
                transform_dead_organisms_into_carcasses.run_if(in_state(PauseState::Running)),
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

fn prepare_assets(mut cmd: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    cmd.insert_resource(CarcassAssets {
        carcass: materials.add(Color::BLACK),
    });
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
    mut cmd: Commands,
    mut event_reader: EventReader<KillOrganismEvent>,
    query: Query<(
        Entity,
        &Transform,
        &EnergyDatav3,
        &Mesh3d,
        Option<&AnimalMarker>,
        Option<&PlantMarker>,
    )>,
    assets: Res<CarcassAssets>,
) {
    for event in event_reader.read() {
        if let Ok((old_entity, transform, energy_data, mesh, maybe_animal, maybe_plant)) =
            query.get(event.entity)
        {
            let new_entity = cmd
                .spawn((
                    Carcass {
                        mass: energy_data.mass,
                        starting_mass: energy_data.mass,
                        energy_per_mass_unit: energy_data.energy_per_mass_unit_gene.phenotype(),
                    },
                    mesh.clone(),
                    MeshMaterial3d(assets.carcass.clone()),
                    *transform,
                ))
                .id();

            if maybe_animal.is_some() {
                cmd.entity(new_entity).insert(AnimalMarker);
            }

            if maybe_plant.is_some() {
                cmd.entity(new_entity).insert(PlantMarker);
            }

            cmd.entity(old_entity).despawn_recursive();
        };
    }
}

fn decay_and_destoy_carcasses_if_needed(
    mut commands: Commands,
    mut carcasses: Query<(Entity, &mut Carcass)>,
    config: Res<SimConfig>,
) {
    for (entity, mut carcass) in carcasses.iter_mut() {
        carcass.mass -= carcass.starting_mass * config.organism.carcass_mass_decay_percentage;

        if carcass.mass <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
