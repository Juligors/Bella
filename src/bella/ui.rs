pub mod camera;

use bevy::prelude::*;

use self::camera::MyCameraPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MyCameraPlugin);
    }
}
