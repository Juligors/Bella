use super::{
    animal::AnimalMarker, plant::PlantMarker, Energy, EnergyData, Health, Meat, PlantMatter,
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
                (decay_animal_carcasses, decay_plant_carcasses).run_if(on_event::<HourPassedEvent>),
            )
            .add_systems(
                Update,
                (
                    destroy_empty_animal_carcasses,
                    destroy_empty_plant_carcasses,
                ),
            )
            .add_systems(
                PostUpdate,
                transform_dead_organisms_into_carcasses.run_if(in_state(PauseState::Running)),
            );
    }
}

#[derive(Component, Reflect, Debug)]
pub struct Carcass {
    pub starting_energy: Energy,
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

fn transform_dead_organisms_into_carcasses(
    mut cmd: Commands,
    query: Query<(
        Entity,
        &Transform,
        &Health,
        &EnergyData,
        &Mesh3d,
        Option<&AnimalMarker>,
        Option<&PlantMarker>,
    )>,
    assets: Res<CarcassAssets>,
) {
    for (old_entity, transform, health, energy_data, mesh, maybe_animal, maybe_plant) in
        query.iter()
    {
        if health.hp <= 0.0 {
            let new_entity = cmd
                .spawn((
                    Carcass {
                        starting_energy: energy_data.energy, // TODO: make sure it's the correct energy (it should be StoredEnergy probably, yeah!)
                    },
                    mesh.clone(),
                    MeshMaterial3d(assets.carcass.clone()),
                    *transform,
                ))
                .id();

            if maybe_animal.is_some() {
                cmd.entity(new_entity).insert(Meat {
                    stored_energy: energy_data.energy,
                });
            }

            if maybe_plant.is_some() {
                cmd.entity(new_entity).insert(PlantMatter {
                    stored_energy: energy_data.energy,
                });
            }

            cmd.entity(old_entity).despawn_recursive();
        }
    }
}

fn decay_animal_carcasses(mut carcasses: Query<(&Carcass, &mut Meat)>, config: Res<SimConfig>) {
    for (carcass, mut meat) in carcasses.iter_mut() {
        meat.stored_energy -= (carcass.starting_energy * config.organism.carcass_energy_decay)
            .min(meat.stored_energy);
    }
}

fn decay_plant_carcasses(
    mut carcasses: Query<(&Carcass, &mut PlantMatter)>,
    config: Res<SimConfig>,
) {
    for (carcass, mut plant_matter) in carcasses.iter_mut() {
        plant_matter.stored_energy -= (carcass.starting_energy
            * config.organism.carcass_energy_decay)
            .min(plant_matter.stored_energy);
    }
}

fn destroy_empty_animal_carcasses(
    mut commands: Commands,
    carcasses: Query<(Entity, &Meat), With<Carcass>>,
) {
    for (entity, meat) in carcasses.iter() {
        if meat.stored_energy <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn destroy_empty_plant_carcasses(
    mut commands: Commands,
    carcasses: Query<(Entity, &PlantMatter), With<Carcass>>,
) {
    for (entity, plant_matter) in carcasses.iter() {
        if plant_matter.stored_energy <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
