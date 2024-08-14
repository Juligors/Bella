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
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_movement, camera_zoom, rotate_camera));
    }
}

#[derive(Component)]
struct MyGameCameraMarker;

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        MyGameCameraMarker,
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, -30.0, 100.0)
                .looking_to(Vec3::new(0.0, 0.3, -1.0), Vec3::Y),
            projection: OrthographicProjection {
                scale: 0.5,
                ..default()
            }
            .into(),
            ..default()
        },
    ));

    // TODO: move to separate function
    cmd.spawn(PointLightBundle {
        transform: Transform::from_xyz(-50.0, -20.0, 200.0),
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 1_000_000_000.0,
            range: 20_000.0,
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
    let mut move_x_by = 0.0;
    let mut move_y_by = 0.0;

    let border_low = CAMERA_BORDER_COEFF * window.width();
    let border_high = (1.0 - CAMERA_BORDER_COEFF) * window.width();

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

    let border_low = CAMERA_BORDER_COEFF * window.height();
    let border_high = (1.0 - CAMERA_BORDER_COEFF) * window.height();

    // down
    if cursor_pos.y < border_low {
        let diff = border_low - cursor_pos.y;
        move_y_by -= (diff / border_low) * move_by_value;
    }
    // up
    if cursor_pos.y > border_high {
        let diff = cursor_pos.y - border_high;
        move_y_by += (diff / border_low) * move_by_value;
    }

    camera_transform.translation += Vec3::new(move_x_by, -move_y_by, 0.0);
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
