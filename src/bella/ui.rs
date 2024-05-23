pub mod camera;
pub mod layer;
pub mod window;

use bevy::prelude::*;

use self::{camera::MyCameraPlugin as MyCameraPlugin, layer::LayerPlugin, window::MyWindowPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MyWindowPlugin, MyCameraPlugin, LayerPlugin));
    }
}
