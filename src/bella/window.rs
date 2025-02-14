use bevy::winit::{UpdateMode, WinitSettings};
use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    prelude::*,
    tasks::available_parallelism,
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};
use std::env;

pub struct MyWindowPlugin;

impl Plugin for MyWindowPlugin {
    fn build(&self, app: &mut App) {
        let program_args: Vec<_> = env::args().collect();
        let mut logging_level = "info".to_string();
        if program_args.len() > 1 {
            logging_level = program_args[1].clone();
        }

        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bella".into(),
                        resolution: (1400., 700.).into(),
                        present_mode: PresentMode::AutoVsync,
                        window_theme: Some(WindowTheme::Dark),
                        window_level: WindowLevel::AlwaysOnTop,
                        position: WindowPosition::At((75, 50).into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        compute: TaskPoolThreadAssignmentPolicy {
                            // set the minimum # of compute threads to the total number of available threads
                            min_threads: available_parallelism(),
                            // unlimited max threads
                            max_threads: usize::MAX,
                            // this value is irrelevant in this case
                            percent: 1.0,
                        },
                        ..default()
                    },
                })
                // .set(LogPlugin {
                //     filter: format!("wgpu=error,naga=warn,bella={}", logging_level),
                //     ..Default::default()
                // }),
                // NOTE: We disable LogPlugin because it causes memory leak. It's needed for tracy traces tho!
                // .disable::<bevy::log::LogPlugin>()
                // .build(),
            // bevy::diagnostic::LogDiagnosticsPlugin {
            //     wait_duration: std::time::Duration::from_secs(5),
            //     ..Default::default()
            // },
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_systems(Startup, setup_window_cursor_lock)
        .add_systems(Update, close_on_esc);
    }
}

fn setup_window_cursor_lock(mut window_q: Query<&mut Window>) {
    let mut window = window_q.single_mut();
    // window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
