use bevy::prelude::*;


#[derive(Component, Hash, PartialEq, Eq, Debug)]
pub enum TerrainType {
    Stone,
    Dirt,
    Grass,
    Water,
}