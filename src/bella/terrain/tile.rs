use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

#[derive(Resource, Debug)]
pub struct TileLayout {
    tiles: Vec<Vec<Tile>>,
    tile_size: f32,
}

impl TileLayout {
    pub fn new(width: u32, height: u32, tile_size: f32) -> Self {
        let tiles = (0..height)
            .map(|x| {
                (0..width)
                    .map(|y| Tile {
                        x: x as f32 * tile_size,
                        y: y as f32 * tile_size,
                    })
                    .collect()
            })
            .collect();

        Self { tiles, tile_size }
    }

    pub fn generate_mesh() -> Mesh {
        generate_square_mesh()
    }

    pub fn is_point_inside(&self, point: Vec2) -> bool {
        self.tiles
            .iter()
            .flatten()
            .any(|tile| tile.is_point_inside(point, self.tile_size))
    }
}

#[derive(Debug)]
pub struct Tile {
    x: f32,
    y: f32,
}

impl Tile {
    pub fn is_point_inside(&self, point: Vec2, tile_size: f32) -> bool {
        let half_a = tile_size / 2.0;

        if point.x > self.x + half_a || point.x < self.x - half_a {
            return false;
        }

        if point.y > self.y + half_a || point.y < self.y - half_a {
            return false;
        }

        true
    }
}

fn generate_square_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        // NOTE: RENDER_WORLD for rendering, MAIN_WORLD for bevy_picking
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    let a = 1.0;
    let x = a / 2.0;

    let vertexes: Vec<[f32; 3]> = vec![
        [-a, 0., a],  // 0 top left
        [a, 0., a],   // 1 top right
        [-a, 0., -a], // 2 bottom left
        [a, 0., -a],  // 3 bottom right
    ];

    let indices = vec![
        2, 1, 0, // top left
        2, 3, 1, // bottom right
    ];
    let normals: Vec<[f32; 3]> = [[0., 1., 0.]].repeat(vertexes.len());
    let uvs: Vec<[f32; 2]> = (0..vertexes.len()).map(|_| [0., 0.]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertexes);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

fn generate_hex_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        // NOTE: RENDER_WORLD for rendering, MAIN_WORLD for bevy_picking
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    let x = 1.0;
    let h = x * 3.0_f32.sqrt() / 2.0;

    let vertexes: Vec<[f32; 3]> = vec![
        [0., 0., 0.],        // 0 center
        [0.5 * x, 0.0, -h],  // 1 top right
        [x, 0.0, 0.0],       // 2 right
        [0.5 * x, 0.0, h],   // 3 bottom right
        [-0.5 * x, 0.0, h],  // 4 bottom left
        [-x, 0.0, 0.0],      // 5 left
        [-0.5 * x, 0.0, -h], // 6 top left
    ];

    let indices = vec![
        0, 1, 6, // top
        0, 2, 1, // top right
        0, 3, 2, // bottom right
        0, 4, 3, // bottom
        0, 5, 4, // bottom left
        0, 6, 5, // top left
    ];
    let normals: Vec<[f32; 3]> = [[0., 1., 0.]].repeat(vertexes.len());
    let uvs: Vec<[f32; 2]> = (0..vertexes.len()).map(|_| [0., 0.]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertexes);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}
