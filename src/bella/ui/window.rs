use bevy::{
    prelude::*,
    window::{close_on_esc, CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};

pub struct MyWindowPlugin;

impl Plugin for MyWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
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
