use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::camera::ScalingMode,
};

use crate::bella::{config::SimulationConfig, ui_facade::EguiFocusState};

pub struct MyCameraPlugin;

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_and_light)
            .add_systems(
                Update,
                (camera_movement, camera_zoom, rotate_camera)
                    .run_if(in_state(EguiFocusState::IsNotFocused)),
            );
    }
}

#[derive(Component)]
struct MyGameCameraMarker;

fn spawn_camera_and_light(mut cmd: Commands, config: Res<SimulationConfig>) {
    cmd.spawn((
        MyGameCameraMarker,
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 200.0)
            .looking_to(Vec3::new(0.0, 0.3, -0.7), Vec3::new(0.0, 0.3, 0.7)),
        Projection::from(OrthographicProjection {
            scale: 0.25,
            near: -1_000_000.0,
            far: 1_000_000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: ScalingMode::WindowSize,
            area: Rect::new(-1.0, -1.0, 1.0, 1.0),
        }),
    ));

    let light_x = config.terrain.map_width as f32 * config.terrain.tile_size / 2.0;
    let light_y = config.terrain.map_height as f32 * config.terrain.tile_size / 2.0;

    cmd.spawn((
        Transform::from_xyz(light_x, light_y, 500_000.0),
        PointLight {
            color: Color::WHITE,
            intensity: 1e16,
            range: 1e20,
            // radius: 1.0,
            // shadows_enabled: true,
            ..Default::default()
        },
    ));
}

fn camera_movement(
    camera: Single<(&mut Transform, &Projection), With<MyGameCameraMarker>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if !mouse_button.pressed(MouseButton::Right) {
        return;
    }

    let (mut transform, projection) = camera.into_inner();

    if let Projection::Orthographic(ortographic_projection) = projection {
        let mut move_by = mouse_motion.delta.extend(0.0);
        move_by.x *= -1.0;
        move_by *= ortographic_projection.scale;

        transform.translation += move_by;
    }
}

fn camera_zoom(
    mut projection: Single<&mut Projection, With<MyGameCameraMarker>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    if let Projection::Orthographic(ortographic_projection) = projection.as_mut() {
        let change = 0.1 * mouse_wheel_input.delta.y.clamp(-1.0, 1.0);
        let log_scale = ortographic_projection.scale.ln() - change;
        ortographic_projection.scale = log_scale.exp();
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
