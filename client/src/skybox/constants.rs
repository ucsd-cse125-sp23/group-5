#[rustfmt::skip]
pub const SKYBOX_VTX : [[f32; 3]; 8] = [
    [-1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0, -1.0, -1.0],
];

#[rustfmt::skip]
pub const SKYBOX_IND : [u16; 36] = [
    //front
    0, 1, 2, 0, 2, 3,
    //top
    1, 5, 6, 1, 6, 2,
    //back
    4, 6, 5, 4, 7, 6,
    //bottom
    0, 7, 4, 0, 3, 7,
    //left
    0, 5, 1, 0, 4, 5,
    //right
    3, 2, 6, 3, 6, 7,
];

pub const VTX_ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
    0 => Float32x3,
];

pub fn vtx_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VTX_ATTRIBS,
    }
}
