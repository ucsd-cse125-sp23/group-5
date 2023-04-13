extern crate nalgebra_glm as glm;
pub struct Camera {
    eye: glm::TVec3<f32>,
    target: glm::TVec3<f32>,
    up: glm::TVec3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: glm::TVec3<f32>,
        target: glm::TVec3<f32>,
        up: glm::TVec3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Camera {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> glm::TMat4<f32> {
        let OPENGL_TO_WGPU_MATRIX: glm::TMat4<f32> = glm::mat4(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
        );
        // 1.
        let view = glm::look_at_rh(&self.eye, &self.target, &self.up);
        // 2.
        let proj = glm::perspective(
            self.aspect,
            self.fovy / 180.0 * glm::pi::<f32>(),
            self.znear,
            self.zfar,
        );

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn update_aspect(&mut self, new_ratio: f32) {
        self.aspect = new_ratio;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glm::mat4(
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            )
            .into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

// --- end Camera
