use bevy::{
    input::mouse::{
        MouseScrollUnit::{Line, Pixel},
        MouseWheel,
    },
    prelude::*,
};

const CAMERA_BORDER_COEFF: f32 = 0.1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_movement, camera_zoom));
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
    ));
}

fn camera_movement(
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<MyGameCameraMarker>>,
    window_q: Query<&Window>,
) {
    let window = window_q.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let (mut camera_transform, projection) = camera_q.single_mut();

    let move_by_value = 10.0 * projection.scale;
    let mut move_x_by = 0.0;
    let mut move_y_by = 0.0;

    let mut border_low = CAMERA_BORDER_COEFF * window.width();
    let mut border_high = (1.0 - CAMERA_BORDER_COEFF) * window.width();

    // left
    if cursor_pos.x < border_low {
        let diff = border_low - cursor_pos.x;
        move_x_by -= (diff / border_low) * move_by_value;
    }
    // right
    if cursor_pos.x > border_high {
        let diff = cursor_pos.x - border_high;
        move_x_by += (diff / border_low) * move_by_value;
    }

    border_low = CAMERA_BORDER_COEFF * window.height();
    border_high = (1.0 - CAMERA_BORDER_COEFF) * window.height();

    // down
    if cursor_pos.y < border_low {
        let diff = border_low - cursor_pos.y;
        move_y_by += (diff / border_low) * move_by_value;
    }
    // up
    if cursor_pos.y > border_high {
        let diff = cursor_pos.y - border_high;
        move_y_by -= (diff / border_low) * move_by_value;
    }

    camera_transform.translation += Vec3::new(move_x_by, move_y_by, 0.0);
}

fn camera_zoom(
    mut scroll_er: EventReader<MouseWheel>,
    mut projection_q: Query<&mut OrthographicProjection, With<Camera>>,
) {
    let mut projection = projection_q.single_mut();
    let mut change = 0.0;

    for event in scroll_er.read() {
        match event.unit {
            Line => {
                change = 0.1 * event.y;
            }
            Pixel => {
                change = 0.2 * event.y;
            }
        }
    }

    let mut log_scale = projection.scale.ln();
    log_scale -= change;
    projection.scale = log_scale.exp();
}
