extern crate nalgebra_glm as glm;
use glm::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A component that represents the position, rotation, and scale of an entity.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: glm::quat_identity(),
        }
    }
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
            ..Default::default()
        }
    }
}

/// A component that represents the physics of an entity.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Physics {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
}
