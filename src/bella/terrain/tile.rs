use std::{cell::RefCell, f32::consts::PI};

use bevy::prelude::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::Uniform;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[derive(Component, Reflect, Debug)]
pub struct Tile {
    pub col: u32,
    pub row: u32,
}

#[derive(Resource, Reflect)]
pub struct TileLayout {
    pub rows: u32,
    pub cols: u32,

    pub tile_size: f32,
    pub half_tile_size: f32,
    pub width: f32,
    pub height: f32,

    #[reflect(ignore)]
    pub entities: Vec<Vec<Entity>>,
}

impl TileLayout {
    pub fn new(rows: u32, cols: u32, tile_size: f32) -> Self {
        Self {
            rows,
            cols,

            tile_size,
            half_tile_size: tile_size / 2.0,
            width: cols as f32 * tile_size,
            height: rows as f32 * tile_size,

            entities: vec![],
        }
    }

    pub fn add_new_row(&mut self) {
        self.entities.push(vec![]);
    }

    pub fn add_new_tile_to_last_row(&mut self, entity: Entity) {
        self.entities
            .last_mut()
            .expect("No vector for entities")
            .push(entity);
    }

    pub fn get_neighbour_entities(&self, col: u32, row: u32) -> Vec<Entity> {
        let neighbor_offsets: [(i32, i32); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let mut entities = Vec::new();

        for (row_offset, col_offset) in neighbor_offsets.into_iter() {
            let neighbour_row = (row as i32 + row_offset) as u32;
            let neighbour_col = (col as i32 + col_offset) as u32;

            if self.is_tile_in_bounds(neighbour_col, neighbour_row) {
                entities.push(self.entities[neighbour_row as usize][neighbour_col as usize]);
            }
        }

        entities
    }

    pub fn get_tile_entity_for_position(&self, position: impl Into<Vec2>) -> Entity {
        self.try_get_tile_entity_for_position(position)
            .expect("Failed to get entity for position")
    }

    pub fn try_get_tile_entity_for_position(&self, position: impl Into<Vec2>) -> Option<Entity> {
        let pos = position.into();

        if self.is_position_in_bounds(pos) {
            let mut col = ((pos.x - pos.x % self.tile_size) / self.tile_size) as usize;
            let mut row = ((pos.y - pos.y % self.tile_size) / self.tile_size) as usize;

            // NOTE: make sure that if position is on the border, we treat it as in bounds
            if row as u32 == self.rows {
                row -= 1;
            }

            if col as u32 == self.cols {
                col -= 1;
            }

            Some(self.entities[row][col])
        } else {
            None
        }
    }

    pub fn get_tile_entity_for_transform(&self, transform: &Transform) -> Entity {
        self.try_get_tile_entity_for_position(transform.translation.truncate())
            .expect("Failed to get entity for transform")
    }

    pub fn try_get_tile_entity_for_transform(&self, transform: &Transform) -> Option<Entity> {
        self.try_get_tile_entity_for_position(transform.translation.truncate())
    }

    pub fn get_tile_entities_in_range(&self, position: impl Into<Vec2>, range: f32) -> Vec<Entity> {

        let pos: Vec2 = position.into();
        let mut tile_entities = Vec::new();

        // get bounds of square around circle
        let min_x = pos.x - range;
        let max_x = pos.x + range;
        let min_y = pos.y - range;
        let max_y = pos.y + range;

        let mut x = min_x;
        let mut y = min_y;
        while x <= max_x {
            while y <= max_y {
                if let Some(tile_entity) = self.try_get_tile_entity_for_position(pos) {
                    // if tile_in_range(){
                    tile_entities.push(tile_entity);
                    // }
                }

                y += self.tile_size;
            }
            x += self.tile_size;
        }

        tile_entities
    }

    pub fn is_tile_in_bounds(&self, col: u32, row: u32) -> bool {
        row < self.rows && col < self.cols
    }

    pub fn is_position_in_bounds(&self, position: impl Into<Vec2>) -> bool {
        let pos: Vec2 = position.into();

        self.is_x_coordinate_in_bounds(pos.x) && self.is_y_coordinate_in_bounds(pos.y)
    }

    pub fn is_x_coordinate_in_bounds(&self, x: f32) -> bool {
        x >= 0.0 && x <= self.width
    }

    pub fn is_y_coordinate_in_bounds(&self, y: f32) -> bool {
        y >= 0.0 && y <= self.height
    }

    pub fn is_position_inside_tile(&self, position: impl Into<Vec2>, tile: &Tile) -> bool {
        let pos = position.into();

        let (tile_min, tile_max) = self.get_tile_bounds(tile);

        if pos.x < tile_min.x || pos.x > tile_max.x {
            return false;
        }

        if pos.y < tile_min.y || pos.y > tile_max.y {
            return false;
        }

        true
    }

    pub fn get_tile_position(&self, tile: &Tile) -> Vec2 {
        Vec2 {
            x: tile.col as f32 * self.tile_size + self.half_tile_size,
            y: tile.row as f32 * self.tile_size + self.half_tile_size,
        }
    }

    pub fn get_tile_bounds(&self, tile: &Tile) -> (Vec2, Vec2) {
        let min = Vec2 {
            x: tile.col as f32 * self.tile_size,
            y: tile.row as f32 * self.tile_size,
        };

        let max = Vec2 {
            x: tile.col as f32 * self.tile_size + self.tile_size,
            y: tile.row as f32 * self.tile_size + self.tile_size,
        };

        (min, max)
    }

    pub fn get_random_position_in_tile(&self, tile: &Tile) -> Vec2 {
        let (pos_min, pos_max) = self.get_tile_bounds(tile);

        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            let x = rng.gen_range(pos_min.x..pos_max.x);
            let y = rng.gen_range(pos_min.y..pos_max.y);

            Vec2::new(x, y)
        })
    }

    pub fn get_random_position_in_ring(
        &self,
        position: impl Into<Vec2>,
        range: f32,
        inner_range: f32,
    ) -> Vec2 {
        let pos = position.into();

        let (r, theta) = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            let r: f32 = rng.gen_range(inner_range..range);
            let theta: f32 = rng.gen_range(0.0..(2.0 * PI));

            (r, theta)
        });

        let x_diff = r * theta.cos();
        let y_diff = r * theta.sin();

        let x_possibly_outside_bounds = pos.x + x_diff;
        let y_possibly_outside_bounds = pos.y + y_diff;

        let mut x = x_possibly_outside_bounds.clamp(0.0, self.width);
        let mut y = y_possibly_outside_bounds.clamp(0.0, self.height);

        // NOTE: we move back position in case it's "smashed" against map bounds
        if x == 0.0 {
            x += self.half_tile_size;
        }
        if x == self.width {
            x -= self.half_tile_size;
        }
        if y == 0.0 {
            y += self.half_tile_size;
        }
        if y == self.height {
            y -= self.half_tile_size;
        }

        Vec2::new(x, y)
    }

    pub fn get_random_position_in_range(&self, position: impl Into<Vec2>, range: f32) -> Vec2 {
        self.get_random_position_in_ring(position, range, 0.0)
    }

    pub fn generate_mesh(&self) -> Mesh {
        Cuboid::new(1.0, 1.0, 0.00001).into()

        // NOTE: this standard Cuboid works better with lighting, custom mesh would probably be more performant, but it's not important for now

        // let mut mesh = Mesh::new(
        //     PrimitiveTopology::TriangleList,
        //     // NOTE: RENDER_WORLD for rendering, MAIN_WORLD for bevy_picking
        //     RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        // );

        // let side_of_the_square = 1.0;
        // let x = side_of_the_square / 2.0;

        // let vertexes: Vec<[f32; 3]> = vec![
        //     [-x, x, 0.],  // 0 top left
        //     [x, x, 0.],   // 1 top right
        //     [-x, -x, 0.], // 2 bottom left
        //     [x, -x, 0.],  // 3 bottom right
        // ];

        // let indices = vec![
        //     2, 1, 0, // top left
        //     2, 3, 1, // bottom right
        // ];
        // let normals: Vec<[f32; 3]> = [[0., 1., 0.]].repeat(vertexes.len());
        // let uvs: Vec<[f32; 2]> = (0..vertexes.len()).map(|_| [0., 0.]).collect();

        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertexes);
        // mesh.insert_indices(Indices::U32(indices));
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        // mesh
    }
}
