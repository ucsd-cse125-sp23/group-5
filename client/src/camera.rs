extern crate nalgebra_glm as glm;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Camera {
    pub position: glm::TVec3<f32>,
    pub target: glm::TVec3<f32>,
    up: glm::TVec3<f32>,
}

impl Camera {
    pub fn new(
        position: glm::TVec3<f32>,
        target: glm::TVec3<f32>,
        up: glm::TVec3<f32>,
    ) -> Self {
        Self {
            position,
            target,
            up, 
        }
    }

    pub fn calc_matrix(&self) -> glm::TMat4<f32> {
        glm::look_at_rh(&self.position,
            &self.target,
            &self.up,
        )
    }
    
    pub fn spherical_coords(&self) -> &glm::TVec3<f32> {
        &self.spherical_coords
    }
    
    pub fn position(&self) -> &glm::TVec3<f32> {
        &self.position
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(
        width: u32,
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> glm::TMat4<f32> {
        #[rustfmt::skip]
        let opengl_to_wgpu_matrix = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );
        opengl_to_wgpu_matrix * glm::perspective(self.aspect, self.fovy, self.znear, self.zfar)
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub inv_view_proj: [[f32; 4]; 4],
    pub location: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: glm::vec4(0.0, 0.0, 0.0, 0.0).into(),
            #[rustfmt::skip]
            view_proj: glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0,
            )
            .into(),
            #[rustfmt::skip]
            inv_view_proj: glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            )
            .into(),
            location: [0.0, 0.0, 1.0, 1.0],
        }
    }


    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = glm::vec4(camera.position.x, camera.position.y, camera.position.z, 1.0).into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
        self.inv_view_proj = glm::inverse(&(projection.calc_matrix() * camera.calc_matrix())).into();
        self.location = [camera.position[0], camera.position[1], camera.position[2], 1.0];
        // print!("{:?}\n", self.location);
    }
}
pub struct CameraState{
    pub camera: Camera,
    pub projection: Projection,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl CameraState{
    pub fn new(
        device: &wgpu::Device,
        eye: glm::TVec3<f32>, target: glm::TVec3<f32>, up: glm::TVec3<f32>, //camera
        w: u32, h: u32, fovy: f32, znear: f32, zfar: f32, //projection
    ) -> Self{
        let camera = Camera::new(eye, target, up);
        let projection = Projection::new(w, h, fovy, znear, zfar);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraState { 
            camera, 
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
         }
    }
}