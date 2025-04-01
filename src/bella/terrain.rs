pub mod terrain_overlay_state;
pub mod thermal_conductor;
pub mod tile;

use std::{cell::RefCell, collections::VecDeque};

use self::thermal_conductor::{
    init_thermal_overlay_update_timer, update_temperatures, ThermalConductor,
    ThermalConductorPlugin,
};
use super::{restart::SimulationState, time::TimeUnitPassedEvent};
use crate::bella::config::SimulationConfig;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    HybridMulti, Perlin,
};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use terrain_overlay_state::{TerrainOverlayState, TerrainOverlayStatePlugin};
use tile::{Tile, TileLayout};

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThermalConductorPlugin, TerrainOverlayStatePlugin))
            .register_type::<BiomeType>()
            .register_type::<Tile>()
            .register_type::<Humidity>()
            .register_type::<Nutrients>()
            .register_type::<ObjectsInTile>()
            .add_systems(
                OnEnter(SimulationState::LoadAssets),
                initialize_assets_map_biomes,
            )
            .add_systems(
                OnEnter(SimulationState::PreSimulation),
                init_thermal_overlay_update_timer, // TODO: do we still need it? Probably just use events
            )
            .add_systems(
                OnEnter(SimulationState::TerrainGeneration),
                (generate_terrain, update_humidity).chain(),
            )
            .add_systems(OnExit(SimulationState::Simulation), despawn_terrain)
            .add_systems(
                Update,
                update_tile_color_for_biome
                    .run_if(in_state(TerrainOverlayState::Bioms))
                    .run_if(in_state(SimulationState::Simulation)),
            )
            .add_systems(
                Update,
                (update_temperatures, reset_nutrients).run_if(on_event::<TimeUnitPassedEvent>),
            );
    }
}

#[derive(Component)]
pub struct TerrainMarker;

#[derive(Bundle)]
pub struct TerrainBundle {
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    transform: Transform,

    marker: TerrainMarker,
    tile: Tile,
    biome: BiomeType,
    thermal_conductor: ThermalConductor,
    nutrients: Nutrients,
    humidity: Humidity,
    objects_in_tile: ObjectsInTile,
}

#[derive(Component, Reflect, Debug, Hash, PartialEq, Eq)]
pub enum BiomeType {
    Stone,
    Sand,
    Dirt,
    Grass,
    Water,
}

impl BiomeType {
    pub fn plants_can_live_here(&self) -> bool {
        *self == BiomeType::Dirt
    }

    pub fn animals_can_live_here(&self) -> bool {
        *self != BiomeType::Water
    }
}

/// ensures that there are more plants (and maybe more animals?) near the water, so we don't have the same number of organisms everywhere (less homogenous?)
#[derive(Component, Reflect, Debug, Clone)]
pub struct Humidity {
    pub value: f32,
}

/// ensure that there are not too many plants on the same chunk.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Nutrients {
    value: f32,
    base_value: f32,
}

impl Nutrients {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            base_value: value,
        }
    }

    pub fn restore_value(&mut self) {
        self.value = self.base_value;
    }

    pub fn take_part_of_nutrients(&mut self, nutrients_to_take: f32) -> f32 {
        let value_to_give = if self.value >= nutrients_to_take {
            nutrients_to_take
        } else {
            self.value
        };

        self.value -= value_to_give;

        value_to_give
    }
}

#[derive(Component, Reflect, Debug)]
pub struct ObjectsInTile {
    pub plants: Vec<Entity>,
    pub animals: Vec<Entity>,
    pub plant_carcasses: Vec<Entity>,
    pub animal_carcasses: Vec<Entity>,
}

impl ObjectsInTile {
    pub fn remove_any_entity(&mut self, entity: Entity) {
        if let Some(index) = self
            .plants
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.plants.swap_remove(index);
            // info!("Removed any plant {}", entity);
            return;
        }

        if let Some(index) = self
            .animals
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.animals.swap_remove(index);
            // info!("Removed any animal {}", entity);
            return;
        }

        if let Some(index) = self
            .plant_carcasses
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.plant_carcasses.swap_remove(index);
            // info!("Removed any plant carcass {}", entity);
            return;
        }

        if let Some(index) = self
            .animal_carcasses
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.animal_carcasses.swap_remove(index);
            // info!("Removed any animal carcass {}", entity);
            return;
        }

        warn!(
            "Cannot remove entity from any ObjectsInTile, entity {:?} not found anywhere",
            entity
        );
        warn!("Plant entities: {:?}", self.plants);
        warn!("Animal entities: {:?}", self.animals);
        warn!("Plant carcass entities: {:?}", self.plant_carcasses);
        warn!("Animal carcass entities: {:?}", self.animal_carcasses);
    }

    pub fn remove_plant_entity(&mut self, entity: Entity) {
        if let Some(index) = self
            .plants
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.plants.swap_remove(index);
            // info!("Removed plant {}", entity);
        } else {
            warn!("Cannot remove plant entity {:?} from ObjectsInTile", entity);
            warn!("Plant entities: {:?}", self.plants);
        }
    }

    pub fn remove_animal_entity(&mut self, entity: Entity) {
        if let Some(index) = self
            .animals
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.animals.swap_remove(index);
            // info!("Removed animal {}", entity);
        } else {
            warn!(
                "Cannot remove animal entity {:?} from ObjectsInTile",
                entity
            );
            warn!("Animal entities: {:?}", self.animals);
        }
    }

    pub fn remove_animal_carcass_entity(&mut self, entity: Entity) {
        if let Some(index) = self
            .animal_carcasses
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.animal_carcasses.swap_remove(index);
            // info!("Removed animal carcass{}", entity);
        } else {
            warn!(
                "Cannot remove animal carcass entity {:?} from ObjectsInTile",
                entity
            );
            warn!("Animal carcass entities: {:?}", self.animal_carcasses);
        }
    }

    pub fn remove_plant_carcass_entity(&mut self, entity: Entity) {
        if let Some(index) = self
            .plant_carcasses
            .iter()
            .position(|&other_entity| other_entity == entity)
        {
            self.plant_carcasses.swap_remove(index);
            // info!("Removed plant carcass {}", entity);
        } else {
            warn!(
                "Cannot remove plant carcass entity {:?} from ObjectsInTile",
                entity
            );
            warn!("Plant carcass entities: {:?}", self.plant_carcasses);
        }
    }

    pub fn add_plant_entity(&mut self, entity: Entity) {
        if !self.plants.contains(&entity) {
            self.plants.push(entity);
            // info!("Added plant {}", entity);
        } else {
            warn!(
                "Cannot add plant entity {:?} to ObjectsInTile, as it has been added already",
                entity
            );
        }
    }

    pub fn add_animal_entity(&mut self, entity: Entity) {
        if !self.animals.contains(&entity) {
            self.animals.push(entity);
            // info!("Added animal {}", entity);
        } else {
            warn!(
                "Cannot add animal entity {:?} to ObjectsInTile, as it has been added already",
                entity
            );
        }
    }

    pub fn add_plant_carcass_entity(&mut self, entity: Entity) {
        if !self.plant_carcasses.contains(&entity) {
            self.plant_carcasses.push(entity);
            // info!("Added plant carcass {}", entity);
        } else {
            warn!(
                "Cannot add plant carcass entity {:?} to ObjectsInTile, as it has been added already",
                entity
            );
        }
    }

    pub fn add_animal_carcass_entity(&mut self, entity: Entity) {
        if !self.animal_carcasses.contains(&entity) {
            self.animal_carcasses.push(entity);
            // info!("Added animal carcass {}", entity);
        } else {
            warn!(
                "Cannot add animal carcass entity {:?} to ObjectsInTile, as it has been added already",
                entity
            );
        }
    }
}

fn generate_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<SimulationConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rows_count = config.terrain.map_height;
    let cols_count = config.terrain.map_width;

    let mut rng = rand::thread_rng();

    let seed = RNG.with(|rng| rng.borrow_mut().gen::<u32>());
    let noise_map = PlaneMapBuilder::new(HybridMulti::<Perlin>::new(seed))
        .set_size(cols_count as usize, rows_count as usize)
        .build();

    // noise_map.write_to_file(std::path::Path::new("test.png"));

    let mut tile_layout = TileLayout::new(rows_count, cols_count, config.terrain.tile_size);

    let mesh = tile_layout.generate_mesh();
    let mesh_handle = meshes.add(mesh);

    // let mut choose_entity_observer = Observer::new(choose_entity_observer);

    for row in 0..rows_count {
        tile_layout.add_new_row();

        for col in 0..cols_count {
            let tile = Tile { row, col };
            let tile_position = tile_layout.get_tile_position(&tile);

            let noise_value = noise_map.get_value(col as usize, row as usize);
            // TODO: for now let's use only Dirt and Water. Need to generate bigger biomes
            let biome = match noise_value {
                x if x < 0.6 => BiomeType::Dirt,
                // x if x < 0.3 => BiomeType::Dirt,
                // x if x < 0.6 => BiomeType::Sand,
                x if x < 1.0 => BiomeType::Water,
                _ => BiomeType::Water,
            };

            let heat_capacity = ThermalConductor::default_heat_capacity();
            let min_heat = heat_capacity * ThermalConductor::min_temperature();
            let max_heat = heat_capacity * ThermalConductor::max_temperature();
            let heat = rng.gen_range(min_heat..max_heat); // TODO: should we just remove heat? It's not really useful in any way
            let k = ThermalConductor::default_thermal_conductivity();
            let thermal_conductor = ThermalConductor {
                heat,
                heat_capacity,
                thermal_conductivity: k,
            };

            let humidity = Humidity { value: 0.0 };
            let nutrients = match biome {
                BiomeType::Dirt => Nutrients::new(config.terrain.nutrients_per_tile_dirt),
                BiomeType::Sand => Nutrients::new(-config.terrain.nutrients_per_tile_sand),
                _ => Nutrients::new(0.0), // TODO: maybe just don't insert it? do Option<Nutrients> in bundle?
            };
            let objects_in_tile = ObjectsInTile {
                plants: Vec::new(),
                animals: Vec::new(),
                plant_carcasses: Vec::new(),
                animal_carcasses: Vec::new(),
            };

            let transform = Transform::from_xyz(tile_position.x, tile_position.y, 0.)
                .with_scale(Vec3::splat(config.terrain.tile_size));

            let entity = commands
                .spawn(TerrainBundle {
                    mesh: Mesh3d(mesh_handle.clone()),
                    material: MeshMaterial3d(materials.add(Color::linear_rgb(0.9, 0.3, 0.3))),
                    transform,
                    marker: TerrainMarker,
                    tile,
                    biome,
                    thermal_conductor,
                    nutrients,
                    humidity,
                    objects_in_tile,
                })
                .id();

            // choose_entity_observer.watch_entity(entity);

            tile_layout.add_new_tile_to_last_row(entity);
        }
    }

    // commands.spawn(choose_entity_observer);

    commands.insert_resource(tile_layout);
}

fn despawn_terrain(mut cmd: Commands, terrain: Query<Entity, With<TerrainMarker>>) {
    for terrain_entity in terrain.iter() {
        cmd.entity(terrain_entity).despawn_recursive();
    }
}

#[derive(Resource)]
pub struct AssetsMapBiomes {
    pub medium_type_materials: HashMap<BiomeType, Handle<StandardMaterial>>,
}

fn initialize_assets_map_biomes(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let medium_type_materials = HashMap::from([
        (BiomeType::Stone, materials.add(Color::srgb(0.5, 0.5, 0.5))),
        (BiomeType::Sand, materials.add(Color::srgb(0.9, 0.9, 0.2))),
        (BiomeType::Dirt, materials.add(Color::srgb(0.8, 0.5, 0.2))),
        (BiomeType::Grass, materials.add(Color::srgb(0.4, 0.9, 0.4))),
        (BiomeType::Water, materials.add(Color::srgb(0.2, 0.4, 0.9))),
    ]);

    cmd.insert_resource(AssetsMapBiomes {
        medium_type_materials,
    });
}

fn update_tile_color_for_biome(
    mut tiles: Query<(&mut MeshMaterial3d<StandardMaterial>, &BiomeType)>,
    assets_map: Res<AssetsMapBiomes>,
) {
    for (mut mesh_material, medium_type) in tiles.iter_mut() {
        mesh_material.0 = assets_map
            .medium_type_materials
            .get(medium_type)
            .unwrap()
            .clone();
    }
}

fn reset_nutrients(mut query: Query<&mut Nutrients>) {
    for mut tile_nutrients in query.iter_mut() {
        if tile_nutrients.value < tile_nutrients.base_value {
            tile_nutrients.restore_value();
        }
    }
}

fn update_humidity(
    query: Query<(&mut Humidity, &BiomeType, &Tile)>,
    tile_layout: Res<TileLayout>,
    config: Res<SimulationConfig>,
) {
    let mut tiles_map: Vec<Vec<_>> = tile_layout
        .entities
        .iter()
        .map(|tile_row| {
            tile_row
                .iter()
                .map(|tile_entity| unsafe {
                    query
                        .get_unchecked(*tile_entity)
                        .expect("Failed to get tile by Entity")
                })
                .collect()
        })
        .collect();

    let mut queue = VecDeque::new();

    // add all water tiles to queue and set their humidity
    for (y, row) in tiles_map.iter_mut().enumerate() {
        for (x, tile) in row.iter_mut().enumerate() {
            if *tile.1 == BiomeType::Water {
                tile.0.value = config.environment.water_humidity;
                queue.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        let src_humidity = tiles_map[y][x].0.value;

        for (nx, ny) in neighbors(x, y, tile_layout.cols as usize, tile_layout.rows as usize) {
            let dist_humidity = &mut tiles_map[ny][nx].0;

            if dist_humidity.value >= src_humidity {
                continue;
            }

            let humidity_to_add = (src_humidity - dist_humidity.value)
                * config.environment.humidity_spread_coefficient;
            if humidity_to_add > dist_humidity.value {
                dist_humidity.value += humidity_to_add;
                queue.push_back((nx, ny));
            }
        }
    }
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut neighbours = Vec::new();

    if x > 0 {
        neighbours.push((x - 1, y));
    }

    if x + 1 < width {
        neighbours.push((x + 1, y));
    }

    if y > 0 {
        neighbours.push((x, y - 1));
    }

    if y + 1 < height {
        neighbours.push((x, y + 1));
    }

    neighbours
}
