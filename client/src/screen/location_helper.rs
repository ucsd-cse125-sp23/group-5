use common::configs::display_config::ScreenLocation;
use crate::screen::objects::Vertex;

pub fn to_absolute(location: &ScreenLocation, width: u32, height: u32) -> [f32; 2] {
    let aspect: f32 = (width as f32) / (height as f32);
    let w = location.horz_disp.0 + location.horz_disp.1 / aspect;
    let h = location.vert_disp.1 + location.vert_disp.0 * aspect;
    [w, h]
}

pub fn get_width(
    height: f32,
    object_aspect: f32,
    screen_width: u32,
    screen_height: u32,
) -> f32 {
    let r_aspect: f32 = (screen_height as f32) / (screen_width as f32);
    return height * object_aspect * r_aspect;
}

pub fn get_coords(
    location: &ScreenLocation,
    aspect: f32,
    obj_height: f32,
    screen_width: u32,
    screen_height: u32,
    vertices: &mut [Vertex; 4],
) {
    let center = to_absolute(location, screen_width, screen_height);
    let width = get_width(obj_height, aspect, screen_width, screen_height);
    vertices[0].position = [center[0] - width / 2.0, center[1] - obj_height / 2.0];
    vertices[1].position = [center[0] - width / 2.0, center[1] + obj_height / 2.0];
    vertices[2].position = [center[0] + width / 2.0, center[1] + obj_height / 2.0];
    vertices[3].position = [center[0] + width / 2.0, center[1] - obj_height / 2.0];
}
