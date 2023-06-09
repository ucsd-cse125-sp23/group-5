use crate::screen::location_helper::{get_coords, to_absolute};
use crate::screen::objects;
use crate::screen::objects::ScreenInstance;
use common::configs::display_config::{ConfigDisplay, ConfigScreen};
use common::core::mesh_color::{MeshColor, MeshColorInstance};
use nalgebra_glm as glm;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub fn create_display_group(config: &ConfigDisplay) -> HashMap<String, objects::DisplayGroup> {
    let mut groups = HashMap::new();
    for g in &config.displays {
        let display_group = objects::DisplayGroup {
            id: g.id.clone(),
            screen: g.screen.clone(),
            scene: g.scene.clone(),
        };
        groups.insert(g.id.clone(), display_group);
    }
    groups
}

pub fn create_screen_map(
    config: &ConfigDisplay,
    device: &wgpu::Device,
    screen_width: u32,
    screen_height: u32,
) -> HashMap<String, objects::Screen> {
    let mut screen_map = HashMap::new();
    for s in &config.screens {
        let background = create_background(s, device);
        let mut icon_id_map: HashMap<String, usize> = HashMap::new();
        let icons = create_icon(s, device, &mut icon_id_map, screen_width, screen_height);

        let mut btn_id_map: HashMap<String, usize> = HashMap::new();
        let buttons = create_button(s, device, &mut btn_id_map, screen_width, screen_height);

        let screen = objects::Screen {
            id: s.id.clone(),
            background,
            icons,
            icon_id_map,
            buttons,
            btn_id_map,
        };
        screen_map.insert(s.id.clone(), screen);
    }
    screen_map
}

fn create_background(s: &ConfigScreen, device: &wgpu::Device) -> Option<objects::ScreenBackground> {
    s.background.as_ref().map(|bg| {
        let vertices = objects::TITLE_VERT;
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        objects::ScreenBackground {
            aspect: bg.aspect,
            vbuf,
            texture: bg.tex.clone(),
            mask_texture: bg.mask_tex.clone(),
        }
    })
}

fn create_icon(
    s: &ConfigScreen,
    device: &wgpu::Device,
    map: &mut HashMap<String, usize>,
    screen_width: u32,
    screen_height: u32,
) -> Vec<objects::Icon> {
    let mut ind = 0;
    s.icons
        .iter()
        .map(|i| {
            //println!("Creating icon: {}", i.id);
            map.insert(i.id.clone(), ind);
            ind += 1;
            let mut vertices = objects::TITLE_VERT;
            get_coords(
                &i.location,
                i.aspect,
                i.height,
                screen_width,
                screen_height,
                &mut vertices,
            );
            for v in &mut vertices {
                v.color = i.tint.clone();
            }
            let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            let instances: Vec<ScreenInstance> = i
                .instances
                .iter()
                .map(|instance_info| {
                    let mut inst_matrix: glm::Mat4 = glm::identity();
                    inst_matrix = glm::scale(
                        &inst_matrix,
                        &glm::vec3(instance_info.scale.0, instance_info.scale.1, 1.0),
                    );
                    inst_matrix = glm::rotate_z(&inst_matrix, instance_info.rotation);
                    let t = to_absolute(&instance_info.translation, screen_width, screen_height);
                    inst_matrix = glm::translate(&inst_matrix, &glm::vec3(t[0], t[1], 0.0));
                    objects::ScreenInstance {
                        transform: inst_matrix.into(),
                    }
                })
                .collect();

            let inst_range = 0..(instances.len() as u32);
            let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(" Some icon (implement ids for more useful messages!) Instance Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            objects::Icon {
                id: i.id.clone(),
                location: i.location,
                aspect: i.aspect,
                height: i.height,
                vertices,
                vbuf,
                tint: glm::make_vec4(&i.tint),
                texture: i.tex.clone(),
                mask_texture: i.mask_tex.clone(),
                instance_raw: i.instances.clone(),
                inst_buf,
                inst_range,
            }
        })
        .collect()
}

fn create_button(
    s: &ConfigScreen,
    device: &wgpu::Device,
    map: &mut HashMap<String, usize>,
    screen_width: u32,
    screen_height: u32,
) -> Vec<objects::Button> {
    let mut ind = 0;
    s.buttons
        .iter()
        .map(|b| {
            match &b.id {
                None => {}
                Some(btn_id) => {
                    map.insert(btn_id.clone(), ind);
                }
            }
            ind += 1;
            let mut vertices = objects::TITLE_VERT;
            get_coords(
                &b.location,
                b.aspect,
                b.height,
                screen_width,
                screen_height,
                &mut vertices,
            );

            for v in &mut vertices {
                v.color = b.default_tint.clone();
            }
            let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(" Some button (implement ids for more useful messages!) Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            objects::Button {
                id: b.id.clone(),
                location: b.location,
                aspect: b.aspect,
                height: b.height,
                vertices,
                vbuf,
                default_tint: glm::make_vec4(&b.default_tint),
                hover_tint: glm::make_vec4(&b.hover_tint),
                default_texture: b.default_tex.clone(),
                hover_texture: b.hover_tex.clone(),
                selected_texture: b.selected_tex.clone(),
                mask_texture: b.mask_tex.clone(),
                on_click: b.on_click.clone(),
                selected: false,
            }
        })
        .collect()
}
