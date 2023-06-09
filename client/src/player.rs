use common::core::command::Command;
use common::core::powerup_system::{PowerUp, PowerUpStatus, StatusEffect};
use common::core::states::PlayerState;
use instant::Duration;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use winit::dpi::PhysicalPosition;
use winit::event::*;

extern crate nalgebra_glm as glm;

use crate::camera::CameraState;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.001;

fn cartesian_to_spherical(cartesian: &glm::Vec3) -> glm::Vec3 {
    let r = glm::length(cartesian);

    if cartesian.x == 0.0 && cartesian.z == 0.0 {
        return glm::vec3(r, 0.0, 0.0);
    }

    let mut yaw = (cartesian.z / cartesian.x).atan();
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

#[derive(Debug, Default)]
pub struct Player {
    pub position: glm::TVec3<f32>,
    pub rotation: glm::Quat,
    pub is_dead: bool,
    pub wind_charge: u32,
    pub on_cooldown: HashMap<Command, f32>,
    pub power_up: Option<(PowerUp, PowerUpStatus)>,
    pub status_effects: HashMap<StatusEffect, f32 /* time till status effect expire */>,
}

impl Player {
    pub fn new(position: glm::TVec3<f32>) -> Self {
        Self {
            position,
            rotation: glm::quat_identity(),
            is_dead: false,
            ..Default::default()
        }
    }

    pub fn calc_transf_matrix(position: glm::TVec3<f32>, rotation: glm::Quat) -> glm::TMat4<f32> {
        glm::translation(&position) * glm::quat_to_mat4(&rotation)
    }
}

#[derive(Debug)]
pub struct PlayerController {
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    x_sensitivity: f32,
    y_sensitivity: f32,
    scroll_sensitivity: f32,
}

impl PlayerController {
    pub fn new(x_sensitivity: f32, y_sensitivity: f32, scroll_sensitivity: f32) -> Self {
        Self {
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            x_sensitivity,
            y_sensitivity,
            scroll_sensitivity,
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
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    /// update the player's position, cooldowns, camera's position and target based on incoming player state
    ///
    pub fn update(
        &mut self,
        player: &mut Player,
        camera_state: &mut CameraState,
        incoming_player_state: &PlayerState,
        dt: Duration,
    ) {
        let translation = incoming_player_state.transform.translation;
        let rotation = incoming_player_state.transform.rotation;

        camera_state.camera.target = translation;

        let pos_delta = translation - player.position;

        player.position = translation;
        player.rotation = rotation;

        camera_state.camera.position += pos_delta;

        let mut spherical_coords =
            cartesian_to_spherical(&(camera_state.camera.position - player.position));

        // update camera
        let dt = dt.as_secs_f32();
        let delta_yaw = self.rotate_horizontal * self.x_sensitivity * dt;
        let delta_pitch = self.rotate_vertical * self.y_sensitivity * dt;

        spherical_coords.x = 10.0; // keep the camera at a fixed distance from the player
        spherical_coords.y = (spherical_coords.y + delta_yaw) % (2.0 * PI);
        // keep the camera's angle from going too high/low
        spherical_coords.z =
            (spherical_coords.z + delta_pitch).clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        camera_state.camera.position = translation + spherical_to_cartesian(&spherical_coords);

        // update camera zoom (can tune parameters later)
        camera_state.projection.fovy = (camera_state.projection.fovy
            + self.scroll * self.scroll_sensitivity * dt)
            .clamp(PI / 6.0, PI / 3.0);
        self.scroll = 0.0;

        // update dead status
        player.is_dead = incoming_player_state.is_dead;

        // update cooldowns
        player.on_cooldown = incoming_player_state.on_cooldown.clone();

        // update ammo count
        player.wind_charge = incoming_player_state.wind_charge;

        // update the powerup and status effects
        player.status_effects = incoming_player_state.status_effects.clone();
        player.power_up = incoming_player_state.power_up.clone();
    }
}
