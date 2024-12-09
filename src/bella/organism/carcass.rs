use super::Health;
use crate::bella::pause::PauseState;
use bevy::prelude::*;

pub struct CarcassPlugin;

impl Plugin for CarcassPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OrganismDiedEvent>()
            .add_systems(Startup, prepare_assets)
            // .add_systems(PostUpdate, transform_dead_organisms_into_carcasses);
            .add_systems(
                PostUpdate,
                despawn_dead_organisms.run_if(in_state(PauseState::Running)),
            );
    }
}

#[derive(Component)]
pub struct CarcassMarker;

#[derive(Resource)]
pub struct CarcassAssets {
    pub carcass: Handle<ColorMaterial>,
}

#[derive(Event)]
pub struct OrganismDiedEvent {
    entity: Entity,
}

fn prepare_assets(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    cmd.insert_resource(CarcassAssets {
        carcass: materials.add(Color::BLACK),
    });
}

fn despawn_dead_organisms(mut cmd: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.hp <= 0. {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

// TODO: for now we just remove organism in function above, don't spawn anything like carcass
// fn transform_dead_organisms_into_carcasses(
//     mut cmd: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut er_organism_died: EventReader<OrganismDiedEvent>,
//     query: Query<&Transform>,
//     assets: Res<CarcassAssets>,
// ) {
//     for event in er_organism_died.read() {
//         let transform = query
//             .get(event.entity)
//             .expect("Entity supposed to be transformed into carcass not found!");

//         let triangle_size = 5.;
//         let mesh_handle = Mesh2dHandle(meshes.add(Triangle2d::new(
//             Vec2::new(0., 0.),
//             Vec2::new(triangle_size, 0.),
//             Vec2::new(triangle_size / 2., triangle_size * 3.0f32.sqrt() / 2.),
//         )));

//         cmd.spawn((
//             CarcassMarker,
//             MaterialMesh2dBundle {
//                 mesh: mesh_handle.clone(),
//                 material: assets.carcass.clone(),
//                 transform: Transform::from_translation(transform.translation),
//                 ..default()
//             },
//         ));
//         cmd.entity(event.entity).despawn_recursive();
//     }
// }
