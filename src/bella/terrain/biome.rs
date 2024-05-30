use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::bella::state::TerrainOverlayState;

pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            initialize_assets_map_biomes,
        )
        .add_systems(
            Update,
            update_tile_color_for_biome.run_if(in_state(TerrainOverlayState::Bioms)),
        );
    }
}

#[derive(Component, Reflect, Hash, PartialEq, Eq, Debug)]
pub enum BiomeType {
    Stone,
    Sand,
    Dirt,
    Grass,
    Water,
}

#[derive(Resource)]
pub struct AssetsMapBiomes {
    pub medium_type_materials: HashMap<BiomeType, Handle<ColorMaterial>>,
}
fn initialize_assets_map_biomes(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let medium_type_materials = HashMap::from([
        (BiomeType::Stone, materials.add(Color::rgb(0.5, 0.5, 0.5))),
        (BiomeType::Sand, materials.add(Color::rgb(0.9, 0.9, 0.2))),
        (BiomeType::Dirt, materials.add(Color::rgb(0.8, 0.5, 0.2))),
        (BiomeType::Grass, materials.add(Color::rgb(0.4, 0.9, 0.4))),
        (BiomeType::Water, materials.add(Color::rgb(0.2, 0.4, 0.9))),
    ]);

    cmd.insert_resource(AssetsMapBiomes {
        medium_type_materials,
    });
}
fn update_tile_color_for_biome(
    mut tiles: Query<(&mut Handle<ColorMaterial>, &BiomeType)>,
    assets_map: Res<AssetsMapBiomes>,
) {
    for (mut handle, medium_type) in tiles.iter_mut() {
        *handle = assets_map
            .medium_type_materials
            .get(medium_type)
            .unwrap()
            .clone();
    }
}
