use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    prelude::*,
    tasks::available_parallelism,
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};

pub struct MyWindowPlugin;

impl Plugin for MyWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bella".into(),
                        resolution: (1000., 700.).into(),
                        present_mode: PresentMode::AutoVsync,
                        window_theme: Some(WindowTheme::Dark),
                        window_level: WindowLevel::AlwaysOnTop,
                        position: WindowPosition::At((400, 100).into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        compute: TaskPoolThreadAssignmentPolicy {
                            // set the minimum # of compute threads
                            // to the total number of available threads
                            min_threads: available_parallelism(),
                            max_threads: std::usize::MAX, // unlimited max threads
                            percent: 1.0,                 // this value is irrelevant in this case
                        },
                        // keep the defaults for everything else
                        ..default()
                    },
                }),
            // bevy::diagnostic::LogDiagnosticsPlugin::default(),
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_systems(Startup, setup_window_cursor_lock)
        .add_systems(Update, close_on_esc);
    }
}

fn setup_window_cursor_lock(mut window_q: Query<&mut Window>) {
    let mut window = window_q.single_mut();

    window.cursor.grab_mode = CursorGrabMode::Confined;
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
