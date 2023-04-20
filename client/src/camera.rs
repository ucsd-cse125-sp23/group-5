use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;
extern crate nalgebra_glm as glm;
use wgpu::util::DeviceExt;


const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.001;

fn cartesian_to_spherical(cartesian: &glm::Vec3) -> glm::Vec3 {
    let r = glm::length(&cartesian);

    if cartesian.x == 0.0 && cartesian.z == 0.0{
        return glm::vec3(r, 0.0, 0.0);
    }

    let mut yaw = (cartesian.z/cartesian.x).atan();
    if cartesian.x < 0.0 {
        yaw += PI;
    }
    let pitch = (cartesian.y / r).asin();
    glm::vec3(r, yaw, pitch)
}

fn spherical_to_cartesian(spherical: &glm::Vec3) -> glm::Vec3 {
    let (r, yaw, pitch) = (spherical.x, spherical.y, spherical.z);
    let x = r * yaw.cos() * pitch.cos();
    let y = r * pitch.sin();
    let z = r * yaw.sin() * pitch.cos();
    glm::vec3(x, y, z)
}

#[derive(Debug)]
pub struct Camera {
    pub position: glm::TVec3<f32>,
    target: glm::TVec3<f32>,
    up: glm::TVec3<f32>,
    spherical_coords: glm::TVec3<f32>, // r yaw pitch
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
            spherical_coords: cartesian_to_spherical(&position), 
        }
    }

    pub fn calc_matrix(&self) -> glm::TMat4<f32> {
        glm::look_at_rh(&self.position,
            &self.target,
            &self.up,
        )
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

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    x_sensitivity: f32,
    y_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, x_sensitivity: f32, y_sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            x_sensitivity,
            y_sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        let direction = glm::normalize(&(camera.target - camera.position));
        /* 
        // Move forward/backward and left/right
        let sin_yaw = camera.yaw.sin(); 
        let cos_yaw = camera.yaw.cos(); 
        let forward = glm::normalize(&glm::vec3(cos_yaw, 0.0, sin_yaw));
        let right = glm::normalize(&glm::vec3(-sin_yaw, 0.0, cos_yaw));
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let sin_pitch = camera.pitch.sin(); 
        let cos_pitch = camera.pitch.cos(); 
        let scrollward = glm::normalize(&glm::vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw));
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;
        */
        // Rotate
        let delta_yaw = self.rotate_horizontal * self.x_sensitivity * dt;
        let delta_pitch = self.rotate_vertical * self.y_sensitivity * dt;
        
        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        
        camera.spherical_coords.y = (camera.spherical_coords.y + delta_yaw) % (2.0 * PI); 

        // keep the camera's angle from going too high/low
        camera.spherical_coords.z = (camera.spherical_coords.z + delta_pitch).clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2); 

        // calculate new camera position from rotated spherical coords
        camera.position = spherical_to_cartesian(&camera.spherical_coords);
        
    } 


}

pub struct CameraState{
    pub camera: Camera,
    pub projection: Projection,
    pub camera_controller: CameraController,
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
        speed: f32, x_sensitivity: f32, y_sensitivity: f32, //camera controller
    ) -> Self{
        let camera = Camera::new(eye, target, up);
        let projection = Projection::new(w, h, fovy, znear, zfar);
        let camera_controller = CameraController::new(speed, x_sensitivity, y_sensitivity);

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
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
         }
    }
}