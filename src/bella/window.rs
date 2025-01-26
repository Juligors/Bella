use std::{fs::File, sync::Mutex};
use std::env;
use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    log::{
        self,
        tracing_subscriber::{
            self,
            fmt::{self, format::FmtSpan},
            FmtSubscriber, Layer,
        },
        BoxedLayer, LogPlugin,
    },
    prelude::*,
    tasks::available_parallelism,
    utils::tracing::{self, level_filters::LevelFilter, Subscriber},
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};

pub struct MyWindowPlugin;

impl Plugin for MyWindowPlugin {
    fn build(&self, app: &mut App) {
        let program_args:Vec<_> = env::args().collect();
        let mut logging_level = "info".to_string();
        if program_args.len() > 1{
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
                        position: WindowPosition::At((75, 100).into()),
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
                .set(LogPlugin {
                    // filter: "info,wgpu_core=error,wgpu_hal=error,bevy_render=error,bevy_ecs=trace,bella=debug".into(),
                    filter: format!("error,bella={},bevy_ecs=trace", logging_level),
                    level: log::Level::DEBUG,
                    // TODO(LOGS): might be cool to customize it to save to file, but would need to filter better and fix formatting issues
                    // custom_layer: custom_logger_layer,
                    ..Default::default()
                }),
            // TODO(LOGS)
            // .disable::<bevy::log::LogPlugin>()
            // .build(),
            // bevy::diagnostic::LogDiagnosticsPlugin::default(),
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
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

struct CustomLoggingLayer;

impl<S: Subscriber> Layer<S> for CustomLoggingLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        println!("Got event!");
        println!("  level={}", event.metadata().level());
        println!("  target={}", event.metadata().target());
        println!("  name={}", event.metadata().name());
    }
}

fn custom_logger_layer(_app: &mut App) -> Option<BoxedLayer> {
    // return None;

    let file = File::create("debug_logs.txt").expect("Failed to create log file");
    let file = Mutex::new(file); // Mutex is needed because the writer may be accessed concurrently.

    let file_layer = fmt::layer()
        .with_writer(file)
        .with_span_events(FmtSpan::CLOSE) // Log spans on close
        .with_target(false) // Don't include target (module path) in file
        .with_filter(LevelFilter::DEBUG); // Only capture DEBUG+ logs

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        // .event_format(fmt::format().format(custom_format))
        .with_filter(LevelFilter::INFO);

    // Additional custom layer to hook into events (optional)
    // let custom_layer = CustomLoggingLayer.boxed();

    // Some(Box::new(vec![file_layer.boxed(), console_layer.boxed(), custom_layer]))
    Some(Box::new(vec![file_layer.boxed(), console_layer.boxed()]))
}

// fn custom_format(
//     writer: &mut dyn std::fmt::Write,
//     event: &tracing::Event<'_>,
//     _: impl tracing_subscriber::fmt::format::FormatFields<'_>,
// ) -> std::fmt::Result {
//     let metadata = event.metadata();
//     writeln!(
//         writer,
//         "[{}] {}: {}",
//         metadata.level(),
//         metadata.target(),
//         metadata.name().unwrap_or("unknown_event"),
//     )
// }
