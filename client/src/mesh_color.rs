use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshColor {
    pub rgb_color: [f32; 3],
}

impl MeshColor {
    pub fn new(rgb: [f32; 3]) -> Self {
        MeshColor { rgb_color: rgb }
    }

    pub fn default() -> Self {
        MeshColor {
            rgb_color: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug)]
pub struct MeshColorInstance {
    pub color: MeshColor,
    pub color_buffer: wgpu::Buffer,
    pub color_bind_group: wgpu::BindGroup,
}

impl MeshColorInstance {
    pub fn new(
        device: &wgpu::Device,
        color_bind_group_layout: &wgpu::BindGroupLayout,
        color: MeshColor,
    ) -> Self {
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh color"),
            contents: bytemuck::cast_slice(&[color]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("color_bind_group"),
        });

        MeshColorInstance {
            color,
            color_buffer,
            color_bind_group,
        }
    }
}
