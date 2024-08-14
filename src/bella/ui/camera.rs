use bevy::{
    input::mouse::{
        MouseScrollUnit::{Line, Pixel},
        MouseWheel,
    },
    prelude::*,
};

pub struct MyCameraPlugin;

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_and_light)
            .add_systems(Update, (camera_movement, camera_zoom, rotate_camera));
    }
}

#[derive(Component)]
struct MyGameCameraMarker;

fn spawn_camera_and_light(mut cmd: Commands) {
    cmd.spawn((
        MyGameCameraMarker,
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 200.0)
                .looking_to(Vec3::new(0.0, 0.3, -0.7), Vec3::new(0.0, 0.3, 0.7)),
            projection: OrthographicProjection {
                scale: 0.5,
                ..default()
            }
            .into(),
            ..default()
        },
    ));

    cmd.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 2_000.0),
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 100_000_000_000.0,
            range: 200_000.0,
            radius: 10.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

const CAMERA_BORDER_COEFF: f32 = 0.1;

fn camera_movement(
    mut camera_q: Query<(&mut Transform, &mut Projection), With<MyGameCameraMarker>>,
    window_q: Query<&Window>,
) {
    let window = window_q.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let (mut camera_transform, projection) = camera_q.single_mut();

    let scale = match projection.as_ref() {
        Projection::Perspective(_) => 1.,
        Projection::Orthographic(ortographic_projection) => ortographic_projection.scale,
    };
    let move_by_value = 10.0 * scale;

    let border_width = CAMERA_BORDER_COEFF * window.width();
    let right_border = window.width() - border_width;
    let left_border = border_width;

    let move_x_by = match cursor_pos.x {
        x if x > right_border => (x - right_border) / border_width,
        x if x < left_border => (x - left_border) / border_width,
        _ => 0.,
    } * move_by_value;

    let border_height = CAMERA_BORDER_COEFF * window.height();
    let top_border = window.height() - border_height;
    let bottom_border = border_height;

    // NOTE: cursor position (0, 0) in in top left corner, not bottom left, so we have to subtract
    let move_y_by = match window.height() - cursor_pos.y {
        y if y > top_border => (y - top_border) / border_height,
        y if y < bottom_border => (y - bottom_border) / border_height,
        _ => 0.,
    } * move_by_value;

    camera_transform.translation += Vec3::new(move_x_by, move_y_by, 0.0);
}

fn camera_zoom(
    mut scroll_er: EventReader<MouseWheel>,
    mut projection_q: Query<&mut Projection, With<Camera>>,
) {
    match projection_q.single_mut().as_mut() {
        Projection::Perspective(_) => (),
        Projection::Orthographic(ortographic_projection) => {
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

            let mut log_scale = ortographic_projection.scale.ln();
            log_scale -= change;
            ortographic_projection.scale = log_scale.exp();
        }
    }
}

// TODO: remove if not needed for debug
fn rotate_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<MyGameCameraMarker>>,
) {
    let mut translation = camera_q.single_mut();

    let sign = if input.pressed(KeyCode::ShiftLeft) {
        -1.
    } else {
        1.
    };

    if input.pressed(KeyCode::KeyX) {
        translation.rotate_x(sign * 1.0_f32.to_radians());
    }

    if input.pressed(KeyCode::KeyY) {
        translation.rotate_y(sign * 1.0_f32.to_radians());
    }

    if input.pressed(KeyCode::KeyZ) {
        translation.rotate_z(sign * 1.0_f32.to_radians());
    }
}
