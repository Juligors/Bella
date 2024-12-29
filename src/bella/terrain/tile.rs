use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

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
            let neighbour_row = row + row_offset as u32;
            let neighbour_col = col + col_offset as u32;

            if self.is_tile_in_bounds(neighbour_col, neighbour_row) {
                entities.push(self.entities[neighbour_row as usize][neighbour_col as usize]);
            }
        }

        entities
    }

    pub fn get_entity_for_position(&self, point: Vec2) -> Option<Entity> {
        if self.is_position_in_bounds(point) {
            let mut col = ((point.x - point.x % self.tile_size) / self.tile_size) as usize;
            let mut row = ((point.y - point.y % self.tile_size) / self.tile_size) as usize;

            // NOTE: make sure that if position is on the border, we treat it as in bounds
            if row as u32 == self.rows {
                row -= 1;
            }

            if col as u32 == self.cols {
                col -= 1;
            }

            // if row as u32 >= self.rows || col as u32 >= self.cols {
            //     println!("Something went wrong!");

            //     println!("Col: {}, row: {}", col, row);
            //     println!("{}", point);

            //     panic!("PANICCCCCCCCCC");
            // }
            Some(self.entities[row][col])
        } else {
            None
        }
    }

    pub fn is_tile_in_bounds(&self, col: u32, row: u32) -> bool {
        row < self.rows && col < self.cols
    }

    pub fn is_position_in_bounds(&self, point: Vec2) -> bool {
        point.x >= 0.0 && point.x <= self.width && point.y >= 0.0 && point.y <= self.height
    }

    pub fn is_position_inside_tile(&self, point: Vec2, tile: &Tile) -> bool {
        let (tile_min, tile_max) = self.get_tile_bounds(tile);

        if point.x < tile_min.x || point.x > tile_max.x {
            return false;
        }

        if point.y < tile_min.y || point.y > tile_max.y {
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

    // pub fn connect_tile_to_entity(x: u32, y: u32, entity: Entity, ){
    //     entity

    // }

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

#[derive(Component, Reflect, Debug)]
pub struct Tile {
    pub col: u32,
    pub row: u32,
}
