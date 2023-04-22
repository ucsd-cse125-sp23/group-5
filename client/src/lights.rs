use wgpu::util::DeviceExt;
extern crate nalgebra_glm as glm;

#[derive(Debug)]
pub struct Light {
    pub position: glm::TVec4<f32>,
    pub color: glm::TVec3<f32>,
}

const MAX_LIGHT: usize = 16;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightsUniform {
    positions: [[f32; 4]; MAX_LIGHT],
    colors: [[f32; 4]; MAX_LIGHT],
    num_lights: u32,
    _padding: [f32; 3],
}

impl LightsUniform {
    pub fn new(arr: &Vec<Light>) -> Self {
        let mut positions: [[f32; 4]; MAX_LIGHT] = [[0.0; 4]; MAX_LIGHT];
        let mut colors = [[0.0; 4]; MAX_LIGHT];
        let num_lights = std::cmp::min(MAX_LIGHT, arr.len());
        // print!("num lights: {}\n", num_lights);

        for ind in 0..num_lights {
            // there's gotta be a better way...
            positions[ind][0] = arr[ind].position[0];
            positions[ind][1] = arr[ind].position[1];
            positions[ind][2] = arr[ind].position[2];
            positions[ind][3] = arr[ind].position[3];
            colors[ind][0] = arr[ind].color[0];
            colors[ind][1] = arr[ind].color[1];
            colors[ind][2] = arr[ind].color[2];
        }

        Self {
            positions,
            colors,
            num_lights: num_lights as u32,
            _padding: [0.0, 0.0, 0.0],
        }
    }
}

pub struct LightState {
    pub lighting: Vec<Light>,
    pub light_uniform: LightsUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
}

impl LightState {
    pub fn new(lights: Vec<Light>, device: &wgpu::Device) -> Self {
        let light_uniform = LightsUniform::new(&lights);
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            lighting: lights,
            light_uniform,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
        }
    }
}
