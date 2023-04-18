// Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
    emission: [f32; 3],
    s: f32
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 7] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 
                                 3 => Float32x3, 4 => Float32x3, 5 => Float32x3, 6 => Float32];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.9, -0.9, 0.0], diffuse: [1.0, 1.0, 1.0] }, // A
//     Vertex { position: [-0.9, -0.8, 0.0], diffuse: [1.0, 1.0, 1.0] }, // B
//     Vertex { position: [-0.7, -0.8, 0.0], diffuse: [1.0, 1.0, 1.0] }, // C
//     Vertex { position: [-0.7, -0.9, 0.0], diffuse: [1.0, 1.0, 1.0] }, // D
// ];

// const INDICES: &[u16] = &[
//     0, 2, 1,
//     0, 3, 2,
// ];

#[rustfmt::skip]
pub const CUBE_VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5,  0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5, -0.5,  0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5,  0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5,  0.5,  0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Back face
    Vertex { position: [ 0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 1.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Right face
    Vertex { position: [ 0.5, -0.5,  0.5], diffuse: [0.0, 0.0, 1.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 1.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5,  0.5], diffuse: [0.0, 1.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Top face
    Vertex { position: [-0.5,  0.5,  0.5], diffuse: [0.0, 1.0, 1.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5,  0.5], diffuse: [0.0, 1.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 1.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Left face
    Vertex { position: [-0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5, -0.5,  0.5], diffuse: [0.5, 0.5, 0.5], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5,  0.5,  0.5], diffuse: [0.0, 1.0, 1.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5,  0.5, -0.5], diffuse: [1.0, 1.0, 1.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0],  ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5, -0.5, -0.5], diffuse: [1.0, 0.0, 1.0], normal: [0.0, -1.0, 0.0],  ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.5, -0.5,  0.5], diffuse: [0.0, 0.0, 1.0], normal: [0.0, -1.0, 0.0],  ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.5, -0.5,  0.5], diffuse: [0.5, 0.5, 0.5], normal: [0.0, -1.0, 0.0],  ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    //Inverted Hull
    // Front face
    Vertex { position: [-0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0,  1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Back face
    Vertex { position: [ 0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Right face
    Vertex { position: [ 0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [ 1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Top face
    Vertex { position: [-0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0,  1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Left face
    Vertex { position: [-0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52,  0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52,  0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    // Bottom face
    Vertex { position: [-0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52, -0.52, -0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [ 0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
    Vertex { position: [-0.52, -0.52,  0.52], diffuse: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0], ambient: [0.0, 0.0, 0.0], specular: [0.0, 0.0, 0.0], emission: [0.0, 0.0, 0.0], s: 0.0},
];

#[rustfmt::skip]
pub const CUBE_INDICES: &[u16] = &[
    // Original
    0,  1,  2,  0,  2,  3, // front
    4,  5,  6,  4,  6,  7, // back
    8,  9, 10,  8, 10, 11, // right
    12, 13, 14, 12, 14, 15, // top
    16, 17, 18, 16, 18, 19, // left
    20, 21, 22, 20, 22, 23, // bottom
    // Inverted hull
    26, 25, 24, 27, 26, 24, // front
    30, 29, 28, 31, 30, 28, // back
    34, 33, 32, 35, 34, 32, // right
    38, 37, 36, 39, 38, 36, // top
    42, 41, 40, 43, 42, 40, // left
    46, 45, 44, 47, 46, 44, // bottom
];

#[rustfmt::skip]
pub const SPHERE_VERTICES: &[Vertex] = &[
    // Interior
    Vertex { position: [0.00, -1.00, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.00, -1.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Strip 1 from bottom
    Vertex { position: [0.50, -0.87, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.50, -0.87, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, -0.87, 0.25], diffuse: [0.20, 0.80, 0.80], normal: [0.43, -0.87, 0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.25, -0.87, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.25, -0.87, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.00, -0.87, 0.50], diffuse: [0.20, 0.80, 0.80], normal: [0.00, -0.87, 0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.25, -0.87, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.25, -0.87, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, -0.87, 0.25], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, -0.87, 0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.50, -0.87, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [-0.50, -0.87, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, -0.87, -0.25], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, -0.87, -0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.25, -0.87, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.25, -0.87, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.00, -0.87, -0.50], diffuse: [0.20, 0.80, 0.80], normal: [-0.00, -0.87, -0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.25, -0.87, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.25, -0.87, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, -0.87, -0.25], diffuse: [0.20, 0.80, 0.80], normal: [0.43, -0.87, -0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Strip 2 from bottom
    Vertex { position: [0.87, -0.50, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.87, -0.50, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.75, -0.50, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.75, -0.50, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, -0.50, 0.75], diffuse: [0.20, 0.80, 0.80], normal: [0.43, -0.50, 0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.00, -0.50, 0.87], diffuse: [0.20, 0.80, 0.80], normal: [0.00, -0.50, 0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, -0.50, 0.75], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, -0.50, 0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.75, -0.50, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.75, -0.50, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.87, -0.50, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [-0.87, -0.50, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.75, -0.50, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.75, -0.50, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, -0.50, -0.75], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, -0.50, -0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.00, -0.50, -0.87], diffuse: [0.20, 0.80, 0.80], normal: [-0.00, -0.50, -0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, -0.50, -0.75], diffuse: [0.20, 0.80, 0.80], normal: [0.43, -0.50, -0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.75, -0.50, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.75, -0.50, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Strip 3 from bottom
    Vertex { position: [1.00, 0.00, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [1.00, 0.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.87, 0.00, 0.50], diffuse: [0.20, 0.80, 0.80], normal: [0.87, 0.00, 0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.50, 0.00, 0.87], diffuse: [0.20, 0.80, 0.80], normal: [0.50, 0.00, 0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.00, 0.00, 1.00], diffuse: [0.20, 0.80, 0.80], normal: [0.00, 0.00, 1.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.50, 0.00, 0.87], diffuse: [0.20, 0.80, 0.80], normal: [-0.50, 0.00, 0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.87, 0.00, 0.50], diffuse: [0.20, 0.80, 0.80], normal: [-0.87, 0.00, 0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-1.00, 0.00, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [-1.00, 0.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.87, 0.00, -0.50], diffuse: [0.20, 0.80, 0.80], normal: [-0.87, 0.00, -0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.50, 0.00, -0.87], diffuse: [0.20, 0.80, 0.80], normal: [-0.50, 0.00, -0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.00, 0.00, -1.00], diffuse: [0.20, 0.80, 0.80], normal: [-0.00, 0.00, -1.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.50, 0.00, -0.87], diffuse: [0.20, 0.80, 0.80], normal: [0.50, 0.00, -0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.87, 0.00, -0.50], diffuse: [0.20, 0.80, 0.80], normal: [0.87, 0.00, -0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Strip 4 from bottom
    Vertex { position: [0.87, 0.50, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.87, 0.50, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.75, 0.50, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.75, 0.50, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, 0.50, 0.75], diffuse: [0.20, 0.80, 0.80], normal: [0.43, 0.50, 0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.00, 0.50, 0.87], diffuse: [0.20, 0.80, 0.80], normal: [0.00, 0.50, 0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, 0.50, 0.75], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, 0.50, 0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.75, 0.50, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.75, 0.50, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.87, 0.50, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [-0.87, 0.50, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.75, 0.50, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.75, 0.50, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, 0.50, -0.75], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, 0.50, -0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.00, 0.50, -0.87], diffuse: [0.20, 0.80, 0.80], normal: [-0.00, 0.50, -0.87],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, 0.50, -0.75], diffuse: [0.20, 0.80, 0.80], normal: [0.43, 0.50, -0.75],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.75, 0.50, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.75, 0.50, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Strip 5 from bottom
    Vertex { position: [0.50, 0.87, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.50, 0.87, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, 0.87, 0.25], diffuse: [0.20, 0.80, 0.80], normal: [0.43, 0.87, 0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.25, 0.87, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.25, 0.87, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.00, 0.87, 0.50], diffuse: [0.20, 0.80, 0.80], normal: [0.00, 0.87, 0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.25, 0.87, 0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.25, 0.87, 0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, 0.87, 0.25], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, 0.87, 0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.50, 0.87, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [-0.50, 0.87, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.43, 0.87, -0.25], diffuse: [0.20, 0.80, 0.80], normal: [-0.43, 0.87, -0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.25, 0.87, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [-0.25, 0.87, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [-0.00, 0.87, -0.50], diffuse: [0.20, 0.80, 0.80], normal: [-0.00, 0.87, -0.50],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.25, 0.87, -0.43], diffuse: [0.20, 0.80, 0.80], normal: [0.25, 0.87, -0.43],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    Vertex { position: [0.43, 0.87, -0.25], diffuse: [0.20, 0.80, 0.80], normal: [0.43, 0.87, -0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },
    // Top point
    Vertex { position: [0.00, 1.00, 0.00], diffuse: [0.20, 0.80, 0.80], normal: [0.00, 1.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.90, 0.90, 0.90], emission: [0.00, 0.00, 0.00], s: 10.00 },

    // Inverted hull
    Vertex { position: [0.00, -1.02, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.00, -1.02, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Strip 1 from bottom
    Vertex { position: [0.51, -0.88, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.51, -0.88, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, -0.88, 0.25], diffuse: [0.00, 0.00, 0.00], normal: [0.44, -0.88, 0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.26, -0.88, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.26, -0.88, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.00, -0.88, 0.51], diffuse: [0.00, 0.00, 0.00], normal: [0.00, -0.88, 0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.25, -0.88, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.25, -0.88, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, -0.88, 0.26], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, -0.88, 0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.51, -0.88, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [-0.51, -0.88, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, -0.88, -0.25], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, -0.88, -0.25],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.26, -0.88, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.26, -0.88, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.00, -0.88, -0.51], diffuse: [0.00, 0.00, 0.00], normal: [-0.00, -0.88, -0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.25, -0.88, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.25, -0.88, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, -0.88, -0.26], diffuse: [0.00, 0.00, 0.00], normal: [0.44, -0.88, -0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Strip 2 from bottom
    Vertex { position: [0.88, -0.51, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.88, -0.51, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.77, -0.51, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.77, -0.51, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, -0.51, 0.76], diffuse: [0.00, 0.00, 0.00], normal: [0.44, -0.51, 0.76],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.00, -0.51, 0.88], diffuse: [0.00, 0.00, 0.00], normal: [0.00, -0.51, 0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, -0.51, 0.77], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, -0.51, 0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.76, -0.51, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.76, -0.51, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.88, -0.51, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [-0.88, -0.51, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.77, -0.51, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.77, -0.51, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, -0.51, -0.76], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, -0.51, -0.76],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.00, -0.51, -0.88], diffuse: [0.00, 0.00, 0.00], normal: [-0.00, -0.51, -0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, -0.51, -0.77], diffuse: [0.00, 0.00, 0.00], normal: [0.44, -0.51, -0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.76, -0.51, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.76, -0.51, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Strip 3 from bottom
    Vertex { position: [1.02, 0.00, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [1.02, 0.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.88, 0.00, 0.51], diffuse: [0.00, 0.00, 0.00], normal: [0.88, 0.00, 0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.51, 0.00, 0.88], diffuse: [0.00, 0.00, 0.00], normal: [0.51, 0.00, 0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.00, 0.00, 1.02], diffuse: [0.00, 0.00, 0.00], normal: [0.00, 0.00, 1.02],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.51, 0.00, 0.88], diffuse: [0.00, 0.00, 0.00], normal: [-0.51, 0.00, 0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.88, 0.00, 0.51], diffuse: [0.00, 0.00, 0.00], normal: [-0.88, 0.00, 0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-1.02, 0.00, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [-1.02, 0.00, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.88, 0.00, -0.51], diffuse: [0.00, 0.00, 0.00], normal: [-0.88, 0.00, -0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.51, 0.00, -0.88], diffuse: [0.00, 0.00, 0.00], normal: [-0.51, 0.00, -0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.00, 0.00, -1.02], diffuse: [0.00, 0.00, 0.00], normal: [-0.00, 0.00, -1.02],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.51, 0.00, -0.88], diffuse: [0.00, 0.00, 0.00], normal: [0.51, 0.00, -0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.88, 0.00, -0.51], diffuse: [0.00, 0.00, 0.00], normal: [0.88, 0.00, -0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Strip 4 from bottom
    Vertex { position: [0.88, 0.51, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.88, 0.51, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.77, 0.51, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.77, 0.51, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, 0.51, 0.77], diffuse: [0.00, 0.00, 0.00], normal: [0.44, 0.51, 0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.00, 0.51, 0.88], diffuse: [0.00, 0.00, 0.00], normal: [0.00, 0.51, 0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, 0.51, 0.77], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, 0.51, 0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.77, 0.51, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.77, 0.51, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.88, 0.51, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [-0.88, 0.51, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.77, 0.51, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.77, 0.51, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, 0.51, -0.77], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, 0.51, -0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.00, 0.51, -0.88], diffuse: [0.00, 0.00, 0.00], normal: [-0.00, 0.51, -0.88],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, 0.51, -0.77], diffuse: [0.00, 0.00, 0.00], normal: [0.44, 0.51, -0.77],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.76, 0.51, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.76, 0.51, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Strip 5 from bottom
    Vertex { position: [0.51, 0.88, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.51, 0.88, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, 0.88, 0.26], diffuse: [0.00, 0.00, 0.00], normal: [0.44, 0.88, 0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.26, 0.88, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.26, 0.88, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.00, 0.88, 0.51], diffuse: [0.00, 0.00, 0.00], normal: [0.00, 0.88, 0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.26, 0.88, 0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.26, 0.88, 0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, 0.88, 0.26], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, 0.88, 0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.51, 0.88, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [-0.51, 0.88, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.44, 0.88, -0.26], diffuse: [0.00, 0.00, 0.00], normal: [-0.44, 0.88, -0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.26, 0.88, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [-0.26, 0.88, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [-0.00, 0.88, -0.51], diffuse: [0.00, 0.00, 0.00], normal: [-0.00, 0.88, -0.51],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.25, 0.88, -0.44], diffuse: [0.00, 0.00, 0.00], normal: [0.25, 0.88, -0.44],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    Vertex { position: [0.44, 0.88, -0.26], diffuse: [0.00, 0.00, 0.00], normal: [0.44, 0.88, -0.26],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
    // Top point
    Vertex { position: [0.00, 1.02, 0.00], diffuse: [0.00, 0.00, 0.00], normal: [0.00, 1.02, 0.00],
             ambient: [0.00, 0.00, 0.00], specular: [0.00, 0.00, 0.00], emission: [0.0, 0.0, 0.0], s: 0.0 },
];

#[rustfmt::skip]
pub const SPHERE_INDICES: &[u16] = &[
    // Original
    // strip 1 from bottom
    0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 7, 0, 7, 8, 0, 8, 9, 0, 9, 10, 0, 10, 11, 0, 11, 12, 0, 12, 1,
    // strip 2 from bottom
    1, 13, 14, 2, 14, 15, 3, 15, 16, 4, 16, 17, 5, 17, 18, 6, 18, 19, 7, 19, 20, 8, 20, 21, 9, 21, 22, 10, 22, 23, 11, 23, 24, 12, 24, 13, 1, 14, 2, 2, 15, 3, 3, 16, 4, 4, 17, 5, 5, 18, 6, 6, 19, 7, 7, 20, 8, 8, 21, 9, 9, 22, 10, 10, 23, 11, 11, 24, 12, 12, 13, 1,
    // strip 3 from bottom
    13, 25, 26, 14, 26, 27, 15, 27, 28, 16, 28, 29, 17, 29, 30, 18, 30, 31, 19, 31, 32, 20, 32, 33, 21, 33, 34, 22, 34, 35, 23, 35, 36, 24, 36, 25, 13, 26, 14, 14, 27, 15, 15, 28, 16, 16, 29, 17, 17, 30, 18, 18, 31, 19, 19, 32, 20, 20, 33, 21, 21, 34, 22, 22, 35, 23, 23, 36, 24, 24, 25, 13,
    // strip 4 from bottom
    25, 37, 38, 26, 38, 39, 27, 39, 40, 28, 40, 41, 29, 41, 42, 30, 42, 43, 31, 43, 44, 32, 44, 45, 33, 45, 46, 34, 46, 47, 35, 47, 48, 36, 48, 37, 25, 38, 26, 26, 39, 27, 27, 40, 28, 28, 41, 29, 29, 42, 30, 30, 43, 31, 31, 44, 32, 32, 45, 33, 33, 46, 34, 34, 47, 35, 35, 48, 36, 36, 37, 25,
    // strip 5 from bottom
    37, 49, 50, 38, 50, 51, 39, 51, 52, 40, 52, 53, 41, 53, 54, 42, 54, 55, 43, 55, 56, 44, 56, 57, 45, 57, 58, 46, 58, 59, 47, 59, 60, 48, 60, 49, 37, 50, 38, 38, 51, 39, 39, 52, 40, 40, 53, 41, 41, 54, 42, 42, 55, 43, 43, 56, 44, 44, 57, 45, 45, 58, 46, 46, 59, 47, 47, 60, 48, 48, 49, 37,
    // strip 6 from bottom
    49, 61, 50, 50, 61, 51, 51, 61, 52, 52, 61, 53, 53, 61, 54, 54, 61, 55, 55, 61, 56, 56, 61, 57, 57, 61, 58, 58, 61, 59, 59, 61, 60, 60, 61, 49,
    // Inverted hull
    // strip 1 from bottom
    64, 63, 62, 65, 64, 62, 66, 65, 62, 67, 66, 62, 68, 67, 62, 69, 68, 62, 70, 69, 62, 71, 70, 62, 72, 71, 62, 73, 72, 62, 74, 73, 62, 63, 74, 62,
    // strip 2 from bottom
    76, 75, 63, 77, 76, 64, 78, 77, 65, 79, 78, 66, 80, 79, 67, 81, 80, 68, 82, 81, 69, 83, 82, 70, 84, 83, 71, 85, 84, 72, 86, 85, 73, 75, 86, 74, 64, 76, 63, 65, 77, 64, 66, 78, 65, 67, 79, 66, 68, 80, 67, 69, 81, 68, 70, 82, 69, 71, 83, 70, 72, 84, 71, 73, 85, 72, 74, 86, 73, 63, 75, 74,
    // strip 3 from bottom
    88, 87, 75, 89, 88, 76, 90, 89, 77, 91, 90, 78, 92, 91, 79, 93, 92, 80, 94, 93, 81, 95, 94, 82, 96, 95, 83, 97, 96, 84, 98, 97, 85, 87, 98, 86, 76, 88, 75, 77, 89, 76, 78, 90, 77, 79, 91, 78, 80, 92, 79, 81, 93, 80, 82, 94, 81, 83, 95, 82, 84, 96, 83, 85, 97, 84, 86, 98, 85, 75, 87, 86,
    // strip 4 from bottom
    100, 99, 87, 101, 100, 88, 102, 101, 89, 103, 102, 90, 104, 103, 91, 105, 104, 92, 106, 105, 93, 107, 106, 94, 108, 107, 95, 109, 108, 96, 110, 109, 97, 99, 110, 98, 88, 100, 87, 89, 101, 88, 90, 102, 89, 91, 103, 90, 92, 104, 91, 93, 105, 92, 94, 106, 93, 95, 107, 94, 96, 108, 95, 97, 109, 96, 98, 110, 97, 87, 99, 98,
    // strip 5 from bottom
    112, 111, 99, 113, 112, 100, 114, 113, 101, 115, 114, 102, 116, 115, 103, 117, 116, 104, 118, 117, 105, 119, 118, 106, 120, 119, 107, 121, 120, 108, 122, 121, 109, 111, 122, 110, 100, 112, 99, 101, 113, 100, 102, 114, 101, 103, 115, 102, 104, 116, 103, 105, 117, 104, 106, 118, 105, 107, 119, 106, 108, 120, 107, 109, 121, 108, 110, 122, 109, 99, 111, 110,
    // strip 6 from bottom
    112, 123, 111, 113, 123, 112, 114, 123, 113, 115, 123, 114, 116, 123, 115, 117, 123, 116, 118, 123, 117, 119, 123, 118, 120, 123, 119, 121, 123, 120, 122, 123, 121, 111, 123, 122,
];
