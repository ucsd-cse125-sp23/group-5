use nalgebra_glm as glm;
use phf::phf_map;
use wgpu::util::DeviceExt;
use std::collections::HashMap;

use crate::scene::Scene;
use crate::screen;
use crate::screen::location::ScreenLocation;

static BUTTON_MAP: phf::Map<&'static str, fn(&mut screen::Display)> = phf_map!{
    "game_start" => game_start,
};

// Place click events here
fn game_start(display: &mut screen::Display){
    display.current = String::from("display:game");
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
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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

const TITLE_VERT: [Vertex; 4] = [
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
    pub screen: Option<Screen>,
    pub scene: Option<Scene>, // To load the scene graph
}

#[derive(Debug)]
pub struct Screen {
    pub background: Option<ScreenBackground>,
    pub icons: Vec<Icon>,
    pub buttons: Vec<Button>,
}

#[derive(Debug)]
pub struct ScreenBackground{
    pub aspect: f32,
    pub vbuf: wgpu::Buffer,
    pub texture: String,
}

#[derive(Debug)]
pub struct Button{
    pub location: ScreenLocation,
    pub aspect: f32,    // both textures must be the same aspect ratio
    pub height: f32,
    pub vertices: [Vertex; 4],
    pub vbuf: wgpu::Buffer,
    pub default_tint: glm::Vec4,
    pub hover_tint: glm::Vec4,
    pub default_texture: String,
    pub hover_texture: String,
    pub is_hover: bool, // For now, we assume nothing can overlap
    pub on_click: Option<String>,
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
    // pub instances: Vec<ConfigTransform>, // TODO: what did they do w/ the transform
    pub inst_range: std::ops::Range<u32>,
}

impl DisplayGroup{
    pub fn resize(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ){
        match self.screen.as_mut(){
            None => {},
            Some(s) => s.resize(screen_width, screen_height, device, queue)
        };
    }
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
            i.resize(screen_width, screen_height);
        }
        for i in &mut self.icons{
            i.resize(screen_width, screen_height);
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
    ){
        self.location.get_coords(
            self.aspect,
            self.height,
            screen_width,
            screen_height,
            &mut self.vertices
        );
    }
}

impl Icon{
    pub fn resize(
        &mut self,
        screen_width: u32,
        screen_height: u32,
    ){
        self.location.get_coords(
            self.aspect,
            self.height,
            screen_width,
            screen_height,
            &mut self.vertices
        );
    }
}

// For testing
pub fn get_display_groups(
    device: &wgpu::Device,
    scene_game: Scene,
    groups: &mut HashMap<String, DisplayGroup>,
){
    // title screen
    let id1 = String::from("display:title");
    let scene1 = None;
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("title background Obj Vertex Buffer"),
        contents: bytemuck::cast_slice(&TITLE_VERT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let bkgd1 = ScreenBackground{
        aspect: 16.0/9.0,
        vbuf,
        texture: String::from("bkgd:title"),
    };
    let button1_loc = ScreenLocation{
        vert_disp: (-0.5, 0.0),
        horz_disp: ( 0.0, 0.0)
    };
    let mut b_vert = TITLE_VERT;
    button1_loc.get_coords(1.0, 0.463, 1920, 1080, &mut b_vert);
    let b_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("title button Vertex Buffer"),
        contents: bytemuck::cast_slice(&TITLE_VERT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let button1 = Button{
        location: button1_loc,
        aspect: 1.0,
        height: 0.463,
        vertices: b_vert,
        vbuf: b_vbuf,
        default_tint: glm::vec4(1.0, 1.0, 1.0, 1.0),
        hover_tint: glm::vec4(1.0, 1.0, 1.0, 1.0),
        default_texture: String::from("btn:title"),
        hover_texture: String::from("btn:title_hover"),
        is_hover: false,
        on_click: Some(String::from("game_start")),
    };
    let title_screen = Screen{
        background: Some(bkgd1),
        buttons: vec![button1],
        icons: vec![],
    };
    let title_dg = DisplayGroup{
        id: id1.clone(),
        screen: Some(title_screen),
        scene: scene1,
    };
    groups.insert(id1, title_dg);
    
    // game group
    let id2 = String::from("display:game");
    let game_dg = DisplayGroup{
        id: id2.clone(),
        screen: None,
        scene: Some(scene_game),
    };
    groups.insert(id2, game_dg);
    return;
}