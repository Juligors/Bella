use bevy::winit::{UpdateMode, WinitSettings};
use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    prelude::*,
    tasks::available_parallelism,
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};

pub struct MyWindowPlugin;

impl Plugin for MyWindowPlugin {
    fn build(&self, app: &mut App) {
        // let program_args: Vec<_> = env::args().collect();
        // let mut logging_level = "info".to_string();
        // if program_args.len() > 1 {
        //     logging_level = program_args[1].clone();
        // }

        #[cfg(not(target_arch = "wasm32"))]
        let default_plugins = DefaultPlugins.set(WindowPlugin {
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
        });

        #[cfg(target_arch = "wasm32")]
        let default_plugins = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bella".into(),
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy-bella".to_owned()),
                fit_canvas_to_parent: true,
                // Tells wasm to override or not default event handling, like F5 and Ctrl+R and right click!
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        });

        app.add_plugins(
            default_plugins
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
                // NOTE: We disable LogPlugin because it causes memory leak
                .disable::<bevy::log::LogPlugin>()
                .build(),
            // bevy::diagnostic::LogDiagnosticsPlugin {
            //     wait_duration: std::time::Duration::from_secs(5),
            //     ..Default::default()
            // },
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        )
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_systems(Startup, setup_window_cursor_lock);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, close_on_esc);
    }
}

fn setup_window_cursor_lock(mut window_q: Query<&mut Window>) {
    let mut window = window_q.single_mut();
    // window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}

#[cfg(not(target_arch = "wasm32"))]
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
