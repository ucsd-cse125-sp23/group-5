use nalgebra_glm as glm;
use phf::phf_map;
use wgpu::util::DeviceExt;
use std::collections::HashMap;

use crate::mesh_color::{MeshColor, MeshColorInstance};
use crate::screen::{self, FinalChoices};
use crate::screen::location::ScreenLocation;

pub static BUTTON_MAP: phf::Map<&'static str, fn(&mut screen::Display, Option<MeshColor>, Option<String>)> = phf_map!{
    "go_to_lobby" => go_to_lobby,
    "game_start" => game_start,
    "next_model" => next_model,
    "change_player_color" => change_player_color,
    "customize_body" => customize_body,
    "customize_leaf" => customize_leaf,
};

// Place click events here ----------------------
fn game_start(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    let final_choices = FinalChoices::new(&display.customization_choices);
    println!("{:#?}", final_choices);
    // TODO: sened final customization choices to server
    display.current = display.game_display.clone();
}

fn customize_body(display: &mut screen::Display,_: Option<MeshColor>, button_id : Option<String>){
    curr_type_selection(display, button_id);
    curr_color_selections(display, Some(display.customization_choices.cur_body_color.clone()));
    display.customization_choices.current_type_choice = "body".to_owned();
}

fn customize_leaf(display: &mut screen::Display, _: Option<MeshColor>, button_id: Option<String>){
    curr_type_selection(display, button_id);
    curr_color_selections(display, Some(display.customization_choices.cur_leaf_color.clone()));
    display.customization_choices.current_type_choice = "leaf".to_owned();
}

fn change_player_color(display: &mut screen::Display, color: Option<MeshColor>, button_id: Option<String>){
    curr_color_selections(display, button_id.clone());

    let actual_color = match color {None => MeshColor::default(), Some(c) => c};
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    match curr_group.scene.clone() {
        None => {},
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id){
                None => {},
                Some(scene) => {
                    // TODO: fix later, hard-coded for now
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {},
                        Some((_, _, color)) => {
                            let curr_choice = display.customization_choices.current_type_choice.clone();
                            if curr_choice == "leaf".to_owned(){
                                display.customization_choices.cur_leaf_color = button_id.clone().unwrap();
                                display.customization_choices.color.insert("eyes_eyes_mesh".to_owned(), actual_color);
                            }
                            else if curr_choice == "body".to_owned() {
                                display.customization_choices.cur_body_color = button_id.clone().unwrap();
                                display.customization_choices.color.insert("leg0R_leg0R_mesh".to_owned(), actual_color);
                            } 
                            *color = display.customization_choices.color.clone();
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
}

fn go_to_lobby(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    display.current = "display:lobby".to_owned();
}

fn next_model(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    match curr_group.scene.clone() {
        None => {},
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id){
                None => {},
                Some(scene) => {
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {},
                        Some((node, _, _)) => {
                            match node.models[0].0.as_str() {
                                "cube" => {
                                    display.customization_choices.current_model = "ferris".to_owned();
                                    node.models[0].0 = "ferris".to_string();
                                },
                                "ferris" => {
                                    display.customization_choices.current_model = "cube".to_owned();
                                    node.models[0].0 = "cube".to_string();
                                },
                                _ => {}
                            }
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
}
// end click events ----------------------

// TODO: merge these two functions into one
fn curr_color_selections(display: &mut screen::Display, button_id: Option<String>){
    let display_group = display.groups.get_mut(&display.current).unwrap();
    let screen: &mut Screen;
    match display_group.screen.as_ref() {
        None => return,
        Some(s) => screen = display.screen_map.get_mut(s).unwrap()
    };
    
    let mut btn = None;
    for button in &mut screen.buttons{
        if button.id.clone().unwrap_or("DNE".to_string()) == display.customization_choices.prev_color_selection.0 {
            button.default_texture = display.customization_choices.prev_color_selection.1.clone();
        }
        if button.id.clone().unwrap_or("DNE".to_string()) == button_id.clone().unwrap_or("dne ".to_string()) {
            btn = Some(button);
        }
    }

    match btn {
        None => {},
        Some(b) => {
            let prev_id = b.id.clone().unwrap().clone();
            let prev_tex = b.default_texture.clone();
            display.customization_choices.prev_color_selection = (prev_id, prev_tex);
            b.default_texture = b.hover_texture.clone();
        }
    }
}

fn curr_type_selection(display: &mut screen::Display, button_id: Option<String>){
    let display_group = display.groups.get_mut(&display.current).unwrap();
    let screen: &mut Screen;
    match display_group.screen.as_ref() {
        None => return,
        Some(s) => screen = display.screen_map.get_mut(s).unwrap()
    };
    
    let mut btn = None;
    for button in &mut screen.buttons{
        if button.id.clone().unwrap_or("DNE".to_string()) == display.customization_choices.prev_type_selection.0 {
            button.default_texture = display.customization_choices.prev_type_selection.1.clone();
        }
        if button.id.clone().unwrap_or("DNE".to_string()) == button_id.clone().unwrap_or("dne ".to_string()) {
            btn = Some(button);
        }
    }

    match btn {
        None => {},
        Some(b) => {
            let prev_id = b.id.clone().unwrap().clone();
            let prev_tex = b.default_texture.clone();
            display.customization_choices.prev_type_selection = (prev_id, prev_tex);
            b.default_texture = b.hover_texture.clone();
        }
    }
}
// Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub texture: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[rustfmt::skip]
pub const RECT_IND : [u16; 6] = [
    0, 2, 1,
    0, 3, 2,
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct ScreenInstance {
    pub transform: [[f32; 4]; 4],
}

impl ScreenInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4
    ];

    pub fn default() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl crate::model::Vertex for ScreenInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const TITLE_VERT: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0],
        color: [1.0, 1.0, 1.0, 1.0],
        texture: [0.0, 1.0],
    }, // A
    Vertex {
        position: [-1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
        texture: [0.0, 0.0],
    }, // B
    Vertex {
        position: [1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
        texture: [1.0, 0.0],
    }, // C
    Vertex {
        position: [1.0, -1.0],
        color: [1.0, 1.0, 1.0, 1.0],
        texture: [1.0, 1.0],
    }, // D
];

// NOTE: all units are relative to either screen height or screen width
//    with the exception of aspect ratio, which is unitless

// #[derive(Debug)]
pub struct DisplayGroup {
    pub id: String,
    pub screen: Option<String>,
    pub scene: Option<String>, // To load the scene graph
}

#[derive(Debug)]
pub struct Screen {
    pub id: String,
    pub background: Option<ScreenBackground>,
    pub icons: Vec<Icon>,
    pub buttons: Vec<Button>,
    pub default_color: MeshColorInstance,
}

#[derive(Debug)]
pub struct ScreenBackground{
    pub aspect: f32,
    pub vbuf: wgpu::Buffer,
    pub texture: String,
    pub color: Option<MeshColorInstance>,
}

#[derive(Debug)]
pub struct Button{
    pub id: Option<String>,
    pub location: ScreenLocation,
    pub aspect: f32,    // both textures must be the same aspect ratio
    pub height: f32,
    pub vertices: [Vertex; 4],
    pub vbuf: wgpu::Buffer,
    pub default_tint: glm::Vec4,
    pub hover_tint: glm::Vec4,
    pub default_texture: String,
    pub hover_texture: String,
    pub color: Option<MeshColorInstance>,
    pub on_click: String,
}

#[derive(Debug)]
pub struct Icon{
    pub location: ScreenLocation,
    pub aspect: f32,
    pub height: f32,
    pub vertices: [Vertex; 4],
    pub vbuf: wgpu::Buffer,
    pub tint: glm::Vec4,
    pub texture: String,
    pub instances: Vec<ScreenInstance>,
    pub inst_buf: wgpu::Buffer,
    pub inst_range: std::ops::Range<u32>,
}

impl Screen{
    pub fn resize(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ){
        match self.background.as_mut() {
            None => {},
            Some(bkgd) => {bkgd.resize(
                    screen_width,
                    screen_height,
                    device,
                    queue,
                );
            }
        };
        for i in &mut self.buttons{
            i.resize(screen_width, screen_height, queue);
        }
        for i in &mut self.icons{
            i.resize(screen_width, screen_height, queue);
        }
    }
}

impl ScreenBackground{
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) {
        let aspect: f32 = (width as f32) / (height as f32);
        const TITLE_AR: f32 = 16.0 / 9.0;
        let title_x_span_half = (glm::clamp_scalar(aspect / TITLE_AR, 0.0, 1.0)) / 2.0;
        let title_vert: Vec<Vertex> = vec![
            Vertex {
                position: [-1.0, -1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                texture: [0.5 - title_x_span_half, 1.0],
            }, // A
            Vertex {
                position: [-1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                texture: [0.5 - title_x_span_half, 0.0],
            }, // B
            Vertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                texture: [0.5 + title_x_span_half, 0.0],
            }, // C
            Vertex {
                position: [1.0, -1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                texture: [0.5 + title_x_span_half, 1.0],
            }, // D
        ];
        queue.write_buffer(
            &self.vbuf,
            0,
            bytemuck::cast_slice(&title_vert),
        );
    }
}

impl Button{
    pub fn resize(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        queue: &wgpu::Queue,
    ){
        self.location.get_coords(
            self.aspect,
            self.height,
            screen_width,
            screen_height,
            &mut self.vertices
        );
        queue.write_buffer(
            &self.vbuf,
            0,
            bytemuck::cast_slice(&self.vertices),
        );
    }

    pub fn is_hover(&self, mouse: &[f32; 2]) -> bool{
        // 1st vertex is lower left
        // 3rd is upper right
        // check x bound
        if self.vertices[0].position[0] > mouse[0]
            || self.vertices[2].position[0] < mouse[0] {
            return false;
        }
        if self.vertices[0].position[1] > mouse[1]
            || self.vertices[2].position[1] < mouse[1] {
            return false;
        }
        return true;
    }
}

impl Icon{
    pub fn resize(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        queue: &wgpu::Queue,
    ){
        self.location.get_coords(
            self.aspect,
            self.height,
            screen_width,
            screen_height,
            &mut self.vertices
        );
        queue.write_buffer(
            &self.vbuf,
            0,
            bytemuck::cast_slice(&self.vertices),
        );
    }
}

// For testing
pub fn get_display_groups(
    device: &wgpu::Device,
    color_bind_group_layout: &wgpu::BindGroupLayout,
    groups: &mut HashMap<String, DisplayGroup>,
){
    // title screen
    let id1 = String::from("display:title");
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("title background Obj Vertex Buffer"),
        contents: bytemuck::cast_slice(&TITLE_VERT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let bkgd1 = ScreenBackground{
        aspect: 16.0/9.0,
        vbuf,
        texture: String::from("bkgd:title"),
        color: None,
    };
    let button1_loc = ScreenLocation{
        vert_disp: (0.0, -0.5),
        horz_disp: (0.0,  0.0)
    };
    let mut b_vert = TITLE_VERT;
    button1_loc.get_coords(1.0, 0.463, 1920, 1080, &mut b_vert);
    let b_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("title button Vertex Buffer"),
        contents: bytemuck::cast_slice(&TITLE_VERT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let button1 = Button{
        id: None,
        location: button1_loc,
        aspect: 1.0,
        height: 0.463 * 2.0,
        vertices: b_vert,
        vbuf: b_vbuf,
        default_tint: glm::vec4(1.0, 1.0, 1.0, 1.0),
        hover_tint: glm::vec4(1.0, 1.0, 1.0, 1.0),
        default_texture: String::from("btn:title"),
        hover_texture: String::from("btn:title_hover"),
        color: None,
        on_click: String::from("game_start"),
    };
    let title_screen = Screen{
        id: String::from("screen:title"),
        background: Some(bkgd1),
        buttons: vec![button1],
        icons: vec![],
        default_color:  MeshColorInstance::new(device, color_bind_group_layout, MeshColor::default()),
    };
    let title_dg = DisplayGroup{
        id: id1.clone(),
        screen: Some(String::from("screen:title")),
        scene: None,
    };
    groups.insert(id1, title_dg);
    
    // game group
    let id2 = String::from("display:game");
    let game_dg = DisplayGroup{
        id: id2.clone(),
        screen: None,
        scene: Some(String::from("scene:game")),
    };
    groups.insert(id2, game_dg);
    return;
}