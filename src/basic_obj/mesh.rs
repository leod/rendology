use crate::basic_obj::{BasicObj, Vertex};
use crate::mesh::Mesh;
use crate::CreationError;

pub fn mesh_from_slices<F: glium::backend::Facade>(
    facade: &F,
    primitive_type: glium::index::PrimitiveType,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
) -> Result<Mesh<Vertex>, CreationError> {
    let vertices = positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| Vertex {
            position: p,
            normal: n,
        })
        .collect::<Vec<_>>();

    Mesh::create_with_indices(facade, primitive_type, &vertices, indices)
}

#[rustfmt::skip]
pub fn create_mesh<F: glium::backend::Facade>(
    object: BasicObj,
    facade: &F,
) -> Result<Mesh<Vertex>, CreationError> {
    match object {
        BasicObj::Triangle => {
            let positions = vec![
                [0.0, -0.5, 0.0],
                [0.0,  0.5, 0.0],
                [1.0,  0.0, 0.0],
            ];

            let normals = vec![
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
            ];

            let indices = vec![0, 1, 2];

            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &positions,
                &normals,
                &indices,
            )
        }
        BasicObj::Quad => {
            let positions = vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ];

            let normals = vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ];

            let indices = vec![
                0, 1, 2,
                2, 3, 0,
            ];

            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &positions,
                &normals,
                &indices,
            )
        }
        BasicObj::Cube => {
            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                CUBE_POSITIONS,
                CUBE_NORMALS,
                CUBE_INDICES,
            )
        }
        BasicObj::Sphere => {
            // For reference: http://www.songho.ca/opengl/gl_sphere.html

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();

            let radius = 0.5;
            let num_stacks = 10;
            let num_sectors = 10;

            let sector_step = 2.0 * std::f32::consts::PI / num_sectors as f32;
            let stack_step = std::f32::consts::PI / num_stacks as f32;

            for i in 0..=num_stacks {
                // Phi goes from pi/2 (top) to -pi/2 (bottom)
                let phi = std::f32::consts::PI / 2.0 - i as f32 * stack_step;

                for j in 0..=num_sectors {
                    // Theta goes from 0 to 2*pi (around the sphere)
                    let theta = j as f32 * sector_step;

                    let x = phi.cos() * theta.cos();
                    let y = phi.cos() * theta.sin();
                    let z = phi.sin();

                    positions.push([x * radius, y * radius, z * radius]);
                    normals.push([x, y, z]);
                }
            }

            for i in 0..num_stacks {
                // Beginning of current stack
                let k_1 = i * (num_sectors + 1);

                // Beginning of next stack
                let k_2 = k_1 + (num_sectors + 1);

                for j in 0..num_sectors {
                    if i != 0 {
                        indices.push(k_1 + j);
                        indices.push(k_2 + j);
                        indices.push(k_1 + j + 1);
                    }

                    if i + 1 != num_stacks {
                        indices.push(k_1 + j + 1);
                        indices.push(k_2 + j);
                        indices.push(k_2 + j + 1);
                    }
                }
            }

            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &positions,
                &normals,
                &indices,
            )
        }
        BasicObj::LineX => {
            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::LinesList,
                &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                &[[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
                &[0, 1],
            )
        }
        BasicObj::LineY => {
            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::LinesList,
                &[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                &[[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
                &[0, 1],
            )
        }
        BasicObj::LineZ => {
            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::LinesList,
                &[[0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                &[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                &[0, 1],
            )
        }
        BasicObj::TessellatedCube => {
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();

            // Number of subdivisions along the x axis
            let n = 64;

            for i in 0..n {
                // Add one x-slice of the cube
                let x_offset = i as f32 / n as f32 - 0.5;

                for (&position, &normal) in CUBE_POSITIONS.iter().zip(CUBE_NORMALS.iter()) {
                    positions.push([x_offset + position[0] / n as f32, position[1], position[2]]);
                    normals.push(normal);
                }

                for &index in CUBE_INDICES {
                    indices.push((CUBE_POSITIONS.len() * i) as u32 + index);
                }
            }

            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &positions,
                &normals,
                &indices,
            )
        }
        BasicObj::TessellatedCylinder => {
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();

            // Number of subdivisions along the x axis
            let n = 32;

            // Number of subdivisions along the angle
            let m = 8;

            // Add positions and normals
            for i in 0..=n {
                let x = i as f32 / n as f32 - 0.5;

                // Add one x-slice of the cylinder
                for j in 0..m {
                    // Add one stripe of the cylinder
                    let theta = j as f32 / m as f32 * 2.0 * std::f32::consts::PI;

                    let y = theta.sin();
                    let z = theta.cos();
                    positions.push([x, y, z]);
                    normals.push([0.0, y, z]);
                }
            }

            // Add triangles
            for i in 0..n {
                for j in 0..m {
                    indices.push((i + 0) * m + ((j + 0) % m));
                    indices.push((i + 1) * m + ((j + 0) % m));
                    indices.push((i + 0) * m + ((j + 1) % m));

                    indices.push((i + 0) * m + ((j + 1) % m));
                    indices.push((i + 1) * m + ((j + 0) % m));
                    indices.push((i + 1) * m + ((j + 1) % m));
                }
            }

            mesh_from_slices(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &positions,
                &normals,
                &indices,
            )
        }
    }
}

#[rustfmt::skip]
pub const CUBE_POSITIONS: &[[f32; 3]] = &[
    // Front
    [-0.5, -0.5,  0.5],
    [ 0.5, -0.5,  0.5],
    [ 0.5,  0.5,  0.5],
    [-0.5,  0.5,  0.5],

    // Right
    [ 0.5,  0.5, -0.5],
    [ 0.5,  0.5,  0.5],
    [ 0.5, -0.5,  0.5],
    [ 0.5, -0.5, -0.5],

    // Back
    [ 0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5,  0.5, -0.5],
    [ 0.5,  0.5, -0.5],

    // Left
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5,  0.5],
    [-0.5,  0.5,  0.5],
    [-0.5,  0.5, -0.5],

    // Top
    [-0.5,  0.5,  0.5],
    [ 0.5,  0.5,  0.5],
    [ 0.5,  0.5, -0.5],
    [-0.5,  0.5, -0.5],

    // Bottom
    [-0.5, -0.5, -0.5],
    [ 0.5, -0.5, -0.5],
    [ 0.5, -0.5,  0.5],
    [-0.5, -0.5,  0.5],
];

#[rustfmt::skip]
pub const CUBE_NORMALS: &[[f32; 3]] = &[
    // Front
    [ 0.0,  0.0,  1.0],
    [ 0.0,  0.0,  1.0],
    [ 0.0,  0.0,  1.0],
    [ 0.0,  0.0,  1.0],

    // Right
    [ 1.0,  0.0,  0.0],
    [ 1.0,  0.0,  0.0],
    [ 1.0,  0.0,  0.0],
    [ 1.0,  0.0,  0.0],

    // Back
    [ 0.0,  0.0, -1.0],
    [ 0.0,  0.0, -1.0],
    [ 0.0,  0.0, -1.0],
    [ 0.0,  0.0, -1.0],

    // Left
    [-1.0,  0.0,  0.0],
    [-1.0,  0.0,  0.0],
    [-1.0,  0.0,  0.0],
    [-1.0,  0.0,  0.0],

    // Top
    [ 0.0,  1.0,  0.0],
    [ 0.0,  1.0,  0.0],
    [ 0.0,  1.0,  0.0],
    [ 0.0,  1.0,  0.0],

    // Bottom
    [ 0.0, -1.0,  0.0],
    [ 0.0, -1.0,  0.0],
    [ 0.0, -1.0,  0.0],
    [ 0.0, -1.0,  0.0],
];

#[rustfmt::skip]
pub const CUBE_INDICES: &[u32] = &[
    // Front
    0, 1, 2, 0, 2, 3,

    // Right
    4, 5, 6, 4, 6, 7,

    // Back
    8, 9, 10, 8, 10, 11,

    // Left
    12, 13, 14, 12, 14, 15,

    // Top
    16, 17, 18, 16, 18, 19,

    // Bottom
    20, 21, 22, 20, 22, 23,
];
