use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::bella::ui::layer::SpriteLayer;

pub struct CarcassPlugin;

impl Plugin for CarcassPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OrganismDiedEvent>()
            .add_systems(Startup, prepare_assets)
            .add_systems(PostUpdate, transform_dead_organisms_into_carcasses);
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

fn transform_dead_organisms_into_carcasses(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut er_organism_died: EventReader<OrganismDiedEvent>,
    query: Query<&Transform>,
    assets: Res<CarcassAssets>,
) {
    for event in er_organism_died.read() {
        let transform = query
            .get(event.entity)
            .expect("Entity supposed to be transformed into carcass not found!");

        let triangle_size = 5.;
        let mesh_handle = Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(0., 0.),
            Vec2::new(triangle_size, 0.),
            Vec2::new(triangle_size / 2., triangle_size * 3.0f32.sqrt() / 2.),
        )));

        cmd.spawn((
            CarcassMarker,
            SpriteLayer::Creature,
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone(),
                material: assets.carcass.clone(),
                transform: Transform::from_translation(transform.translation),
                ..default()
            },
        ));
        cmd.entity(event.entity).despawn_recursive();
    }
}
