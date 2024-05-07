use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .add_systems(Startup, spawn_camera);
    }
}

#[derive(Component)]
struct MyGameCameraMarker;

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        MyGameCameraMarker,
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 1000.),
            projection: OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::WindowSize(2.5),
                ..Default::default()
            },
            ..default()
        },
        PanCam::default(),
    ));
}
