use bevy::prelude::*;
use extol_sprite_layer::*;

pub struct LayerPlugin;

impl Plugin for LayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpriteLayerPlugin::<SpriteLayer>::default())
            .add_systems(Startup, |mut options: ResMut<SpriteLayerOptions>| {
                options.y_sort = false
            });
    }
}

#[derive(Debug, Clone, Component, Hash, PartialEq, Eq)]
pub enum SpriteLayer {
    Terrain,
    Plant,
    Creature,
    UI,
}

impl LayerIndex for SpriteLayer {
    fn as_z_coordinate(&self) -> f32 {
        match *self {
            SpriteLayer::Terrain => 1.,
            SpriteLayer::Plant => 100.,
            SpriteLayer::Creature => 500.,
            SpriteLayer::UI => 900.,
        }
    }
}
