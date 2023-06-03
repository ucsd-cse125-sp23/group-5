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

pub const ATK_BASE_IND: u32 = 0;
pub const ATK_NUM_TEX_TYPES: u32 = 9;
pub const SOFT_CIRCLE_IND: u32 = 36;
pub const LABEL_BASE_IND: u32 = 37;
pub const POWER_UP_IND: u32 = 41;
pub const RAIN_IND: u32 = 42;
pub const STREAK_IND: u32 = 43;
pub const SOLID_IND: u32 = 44;
pub const SNOW_BASE_IND: u32 = 45;
pub const SNOW_NUM_TEX_TYPES: u32 = 2;

// pub const MODEL_1: &str = "attack shape 1";
// pub const MODEL_2: &str = "attack shape 2";
// pub const MODEL_3: &str = "attack shape 3";
// pub const MODEL_4: &str = "attack shape 4";

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
