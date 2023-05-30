#[rustfmt::skip]
pub const VTXS: &[[f32; 2]; 4] = &[
    [0.0, 1.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
];

pub const VTX_ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
    0 => Float32x2,
];

pub const POINT_PARTICLE: u32 = 0;
pub const RIBBON_PARTICLE: u32 = 1;
pub const TRAIL_PARTICLE: u32 = 2;

pub fn vtx_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VTX_ATTRIBS,
    }
}

#[rustfmt::skip]
pub const INDS: &[u16; 6] = &[
    0, 1, 2,
    0, 2, 3,
];
