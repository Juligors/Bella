use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub struct MyCameraPlugin;

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, spawn_camera_and_light);
    }
}

#[derive(Component)]
struct MyGameCameraMarker;

fn spawn_camera_and_light(mut cmd: Commands) {
    let mut projection = OrthographicProjection::default_3d();
    projection.scale = 0.5;

    cmd.spawn((
        MyGameCameraMarker,
        Camera3d::default(),
        PanOrbitCamera {
            allow_upside_down: true,
            focus: Vec3::new(0., 300., 0.),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 1600.0),
    ));

    cmd.spawn((
        Transform::from_xyz(0.0, 0.0, 2_000.0),
        PointLight {
            color: Color::WHITE,
            intensity: 100_000_000_000.0,
            range: 200_000.0,
            radius: 10.0,
            shadows_enabled: true,
            ..Default::default()
        },
    ));
}
