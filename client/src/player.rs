use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;
use std::collections::HashMap;
use common::core::states::PlayerState;

extern crate nalgebra_glm as glm;

use crate::camera::Camera;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.001;

fn cartesian_to_spherical(cartesian: &glm::Vec3) -> glm::Vec3 {
    let r = glm::length(cartesian);

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
pub struct Player {
    pub position: glm::TVec3<f32>,
    pub rotation: glm:: Quat,
    up: glm::TVec3<f32>,
}

impl Player {
    pub fn new(
        position: glm::TVec3<f32>,
    ) -> Self {
        let up = glm::vec3(0.0, 1.0, 0.0);
        Self {
            position,
            rotation: glm::quat_identity(),
            up,
        }
    }

    pub fn calc_transf_matrix(&self) -> glm::TMat4<f32> {
        glm::translation(&self.position) * glm::quat_to_mat4(&self.rotation)
    }
}

#[derive(Debug)]
pub struct PlayerController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    key_map: HashMap<VirtualKeyCode, bool>,
    amount_up: f32,
    amount_down: f32,
    camera_forward: glm::TVec3<f32>,
    camera_right: glm::TVec3<f32>,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    x_sensitivity: f32,
    y_sensitivity: f32,

}

impl PlayerController {
    pub fn new(speed: f32, x_sensitivity: f32, y_sensitivity: f32) -> Self {
        let mut key_map = HashMap::new();
        key_map.insert(VirtualKeyCode::W, false);
        key_map.insert(VirtualKeyCode::A, false);
        key_map.insert(VirtualKeyCode::S, false);
        key_map.insert(VirtualKeyCode::D, false);

        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            key_map,
            amount_up: 0.0,
            amount_down: 0.0,
            camera_forward: glm::vec3(1.0, 0.0, 0.0),
            camera_right: glm::vec3(0.0, 0.0, 1.0),
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,

            speed,
            x_sensitivity,
            y_sensitivity,
            
        }
    }

    // TODO: moved this function to server side
    // pub fn process_keyboard(&mut self, key: VirtualKeyCode, player: &mut Player, state: ElementState) -> bool{
    //     let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
    //     if state == ElementState::Pressed {
    //         self.key_map.entry(key).and_modify(|e| {*e = true});
    //         match key {
    //             VirtualKeyCode::W => {
    //                 self.amount_forward = amount;
    //
    //                 // diagonal movement
    //                 if self.key_map.get(&VirtualKeyCode::A) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, PI/4.0, &player.up)
    //                 } else if self.key_map.get(&VirtualKeyCode::D) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, -PI/4.0, &player.up)
    //                 } else {
    //                     player.rotation = self.camera_forward;
    //                 }
    //
    //                 true
    //             }
    //             VirtualKeyCode::S => {
    //                 self.amount_backward = amount;
    //
    //                 // diagonal movement
    //                 if self.key_map.get(&VirtualKeyCode::A) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, 3.0 * PI/4.0, &player.up)
    //                 } else if self.key_map.get(&VirtualKeyCode::D) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, -3.0 * PI/4.0, &player.up)
    //                 } else {
    //                     player.rotation = -&self.camera_forward;
    //                 }
    //
    //                 true
    //             }
    //             VirtualKeyCode::A => {
    //                 self.amount_left = amount;
    //
    //                 // diagonal movement
    //                 if self.key_map.get(&VirtualKeyCode::W) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, PI/4.0, &player.up)
    //                 } else if self.key_map.get(&VirtualKeyCode::S) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, 3.0 * PI/4.0, &player.up)
    //                 } else {
    //                     player.rotation = -&self.camera_right;
    //                 }
    //
    //                 true
    //             }
    //             VirtualKeyCode::D => {
    //                 self.amount_right = amount;
    //
    //                 // diagonal movement
    //                 if self.key_map.get(&VirtualKeyCode::W) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, -PI/4.0, &player.up)
    //                 } else if self.key_map.get(&VirtualKeyCode::S) == Some(&true) {
    //                     player.rotation = glm::rotate_vec3(&self.camera_forward, -3.0 * PI/4.0, &player.up)
    //                 } else {
    //                     player.rotation = self.camera_right;
    //                 }
    //                 true
    //             }
    //             VirtualKeyCode::Space => {
    //                 self.amount_up = amount;
    //                 true
    //             }
    //             VirtualKeyCode::LShift => {
    //                 self.amount_down = amount;
    //                 true
    //             }
    //             _ => false,
    //         }
    //     } else if state == ElementState::Released {
    //         self.key_map.entry(key).and_modify(|e| {*e = false});
    //         match key {
    //             VirtualKeyCode::W => {
    //                 self.amount_forward = amount;
    //                 true
    //             }
    //             VirtualKeyCode::S => {
    //                 self.amount_backward = amount;
    //                 true
    //             }
    //             VirtualKeyCode::A => {
    //                 self.amount_left = amount;
    //                 true
    //             }
    //             VirtualKeyCode::D => {
    //                 self.amount_right = amount;
    //                 true
    //             }
    //             VirtualKeyCode::Space => {
    //                 self.amount_up = amount;
    //                 true
    //             }
    //             VirtualKeyCode::LShift => {
    //                 self.amount_down = amount;
    //                 true
    //             }
    //             _ => false,
    //         }
    //     } else {
    //         false
    //     }
    //
    //
    // }

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

    pub fn update_player(&mut self, player: &mut Player, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        let direction = camera.target - camera.position;
        
        // Move forward/backward and left/right
        self.camera_forward = glm::normalize(&glm::vec3(direction.x, 0.0, direction.z));
        self.camera_right = glm::cross::<f32>(&self.camera_forward, &player.up);
        
        let delta_forward = self.camera_forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        let delta_right = self.camera_right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        /* 
        let sin_pitch = camera.pitch.sin(); 
        let cos_pitch = camera.pitch.cos(); 
        let scrollward = glm::normalize(&glm::vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw));
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;
        */

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        let delta_y = (self.amount_up - self.amount_down) * self.speed * dt;

        // Modify camera and player position based on deltas
        let delta_combined  = delta_forward + delta_right + glm::vec3(0.0, delta_y, 0.0);

        camera.position += delta_combined;
        camera.target += delta_combined;
        player.position += delta_combined; 
        //player.forward = forward;
        
        // Rotate
        let delta_yaw = self.rotate_horizontal * self.x_sensitivity * dt;
        let delta_pitch = self.rotate_vertical * self.y_sensitivity * dt;
        
        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        
        let mut spherical_coords = cartesian_to_spherical(&(camera.position - player.position));
        spherical_coords.y = (spherical_coords.y + delta_yaw) % (2.0 * PI); 

        // keep the camera's angle from going too high/low
        spherical_coords.z = (spherical_coords.z + delta_pitch).clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2); 

        // calculate new camera position from rotated spherical coords
        // not sure about this with movement, might need fix later
        camera.position = player.position + spherical_to_cartesian(&spherical_coords);
        
    }

    /// update the player's position and camera's position and target based on incoming player state
    pub fn update(&mut self, player: &mut Player, camera: &mut Camera, incoming_player_state: &PlayerState, dt: Duration) {
        let translation = incoming_player_state.transform.translation;
        let rotation = incoming_player_state.transform.rotation;

        camera.target = translation;

        let pos_delta= translation - player.position;

        player.position = translation;
        player.rotation = rotation;

        camera.position += pos_delta;

        let mut spherical_coords = cartesian_to_spherical(&(camera.position - player.position));

        // update camera
        let dt = dt.as_secs_f32();
        let delta_yaw = self.rotate_horizontal * self.x_sensitivity * dt;
        let delta_pitch = self.rotate_vertical * self.y_sensitivity * dt;

        spherical_coords.x = 10.0; // keep the camera at a fixed distance from the player
        spherical_coords.y = (spherical_coords.y + delta_yaw) % (2.0 * PI);
        // keep the camera's angle from going too high/low
        spherical_coords.z = (spherical_coords.z + delta_pitch).clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        camera.position = translation + spherical_to_cartesian(&spherical_coords);
    }


}

