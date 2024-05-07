use bevy::prelude::*;
use hexx::*;

use crate::config::SimConfig;

#[derive(Component)]
pub struct ThermalConductor {
    /// Position on the hex grid
    pub hex_pos: Hex,
    /// https://pl.wikipedia.org/wiki/Ciep%C5%82o
    pub heat: f32,
    /// With `heat_capacity` we can calculate heat<->temperature conversion:
    /// `Q = C * delta_T`, where Q is heat, C is heat_capacity and delta_T is temperature difference.
    /// More: https://pl.wikipedia.org/wiki/Pojemno%C5%9B%C4%87_cieplna
    pub heat_capacity: f32,
    /// `Thermal_conductivity` controls how much Heat is transfered between 2 or more `Medium` instances with different temperatures.
    /// More: https://pl.wikipedia.org/wiki/Przewodno%C5%9B%C4%87_cieplna
    pub thermal_conductivity: f32,
}

impl ThermalConductor {
    pub fn temperature(&self) -> f32 {
        self.heat / self.heat_capacity
    }

    pub const fn min_temperature() -> f32 {
        0.
    }

    pub const fn max_temperature() -> f32 {
        100.
    }

    pub const fn default_heat_capacity() -> f32 {
        1000.
    }

    pub const fn default_thermal_conductivity() -> f32 {
        20.
    }

    pub const fn default_heat_lose() -> f32 {
        50.
    }
}

pub fn update_temperatures(mut media: Query<&mut ThermalConductor>) {
    let mut media_org: Vec<_> = media.iter_mut().collect();

    // we will set all heat at once to make sure order of iteration doesn't matter and energy is conserved
    let mut heat_diffs: Vec<f32> = Vec::with_capacity(media_org.capacity());

    for i in 0..media_org.len() {
        let all_neighbour_hexes = media_org[i].hex_pos.all_neighbors();
        let all_existing_neighbours = media_org
            .iter()
            .enumerate()
            .filter(|(_, t)| all_neighbour_hexes.contains(&t.hex_pos))
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let heat_diff = all_existing_neighbours
            .iter()
            .map(|&neighbour_i| {
                let temp_diff = media_org[i].temperature() - media_org[neighbour_i].temperature();
                media_org[i].thermal_conductivity * temp_diff
            })
            .sum();

        heat_diffs.push(heat_diff);
    }

    // set all heat at once
    // also lower temperature by constant, like it's getting colder with time
    for i in 0..media_org.len() {
        media_org[i].heat -= heat_diffs[i] + ThermalConductor::default_heat_lose();

        // if temperature is below minimal set it to minimal
        if media_org[i].temperature() < ThermalConductor::min_temperature() {
            media_org[i].heat = ThermalConductor::min_temperature() * media_org[i].heat_capacity;
        }
    }
}
