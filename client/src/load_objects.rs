use std::io::{BufReader, Cursor};
use std::convert::TryInto;
use tobj;

//use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::objects::Vertex;

//use crate::{model, texture};

// #[cfg(target_arch = "wasm32")]
// fn format_url(file_name: &str) -> reqwest::Url {
//     let window = web_sys::window().unwrap();
//     let location = window.location();
//     let base = reqwest::Url::parse(&format!(
//         "{}/{}/",
//         location.origin().unwrap(),
//         option_env!("RES_PATH").unwrap_or("assets"),
//     ))
//     .unwrap();
//     base.join(file_name).unwrap()
// }

// pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
//     cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             log::warn!("Load model on web");

//             let url = format_url(file_name);
//             let txt = reqwest::get(url)
//                 .await?
//                 .text()
//                 .await?;

//             log::warn!("{}", txt);

//         } else {
//             let path = std::path::Path::new("assets")
//                 .join(file_name);
//             let txt = std::fs::read_to_string(path)?;
//         }
//     }

//     Ok(txt)
// }

// pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
//     cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             let url = format_url(file_name);
//             let data = reqwest::get(url)
//                 .await?
//                 .bytes()
//                 .await?
//                 .to_vec();
//         } else {
//             let path = std::path::Path::new("assets")
//                 .join(file_name);
//             let data = std::fs::read(path)?;
//         }
//     }

//     Ok(data)
// }

// pub async fn load_texture(
//     file_name: &str,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
// ) -> anyhow::Result<texture::Texture> {
//     let data = load_binary(file_name).await?;
//     texture::Texture::from_bytes(device, queue, &data, file_name)
// }

// TODO: Add support for the mapped texture files
// TODO: Add support for more than one material
// TODO: Add default material if material doesn't exist
pub fn load_model(
    path: &str
    //device: &wgpu::Device,
    //queue: &wgpu::Queue,
) -> (Vec<Vertex>, Vec<u32>) { //-> anyhow::Result<model::Model> {
    let object_file = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
    let (models, materials) = object_file.expect("Failed to load OBJ file");

    let materials = materials.expect("Failed to load MTL file");
    // let mut total: usize = 0;

    // for (i, m) in models.iter().enumerate() {
    //     let mesh = &m.mesh;
    //     total = total + mesh.positions.len() / 3;
    // }
    // const num_vertices = total;

    let mut object_vertices: Vec<Vertex> = Vec::new();
    let mut object_indices: Vec<u32> = Vec::new(); 
    let material: tobj::Material = materials.get(0).unwrap().clone();

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("Indices: {} : {:?}", mesh.indices.len(), mesh.indices);
        println!("Normals: {} : {:?}", mesh.normals.len(), mesh.normals);
        println!("Positions: {} : {:?}", mesh.positions.len(), mesh.positions);

        // if(i == 0) {
        //     println!("{:?}", mesh.indices);
        //     println!("Num vertices {}", mesh.positions.len() / 3);
        //     println!("Num normals {}", mesh.normals.len() / 3);
        //     println!("Num faces {}", mesh.indices.len() / 3);
        // }
        object_indices.append(&mut mesh.indices.clone());
        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..mesh.positions.len() / 3 {
            object_vertices.push(Vertex { position: [mesh.positions[3 * v], mesh.positions[3 * v + 1], mesh.positions[3 * v + 2]], 
                diffuse: [material.diffuse[0], material.diffuse[1], material.diffuse[2]], 
                normal: [mesh.normals[3 * v], mesh.normals[3 * v + 1], mesh.normals[3 * v + 2]], //
                ambient: [material.ambient[0], material.ambient[1], material.ambient[2]], 
                specular: [material.specular[0], material.specular[1], material.specular[2]], 
                emission: [0.0, 0.0, 0.0], // TODO: Figure out if MTL file format supports emission (Ke) --> it does, use the material.unknown_param HashMap
                s:  material.shininess});
        }
    }

    return (object_vertices, object_indices);
    // let mut obj_reader = BufReader::new(obj_cursor);

    // let (models, obj_materials) = tobj::load_obj_buf_async(
    //     &mut obj_reader,
    //     &tobj::LoadOptions {
    //         triangulate: true,
    //         single_index: true,
    //         ..Default::default()
    //     },
    //     |p| async move {
    //         let mat_text = load_string(&p).await.unwrap();
    //         tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
    //     },
    // )
    // .await?;

    // let mut materials = Vec::new();
    // for m in obj_materials? {
    //     let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
    //     materials.push(model::Material {
    //         name: m.name,
    //         diffuse_texture,
    //     })
    // }

    // let meshes = models
    //     .into_iter()
    //     .map(|m| {
    //         let vertices = (0..m.mesh.positions.len() / 3)
    //             .map(|i| model::ModelVertex {
    //                 position: [
    //                     m.mesh.positions[i * 3],
    //                     m.mesh.positions[i * 3 + 1],
    //                     m.mesh.positions[i * 3 + 2],
    //                 ],
    //                 tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
    //                 normal: [
    //                     m.mesh.normals[i * 3],
    //                     m.mesh.normals[i * 3 + 1],
    //                     m.mesh.normals[i * 3 + 2],
    //                 ],
    //             })
    //             .collect::<Vec<_>>();

    //         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Vertex Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&vertices),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         });
    //         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Index Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&m.mesh.indices),
    //             usage: wgpu::BufferUsages::INDEX,
    //         });

    //         model::Mesh {
    //             name: file_name.to_string(),
    //             vertex_buffer,
    //             index_buffer,
    //             num_elements: m.mesh.indices.len() as u32,
    //             material: m.mesh.material_id.unwrap_or(0),
    //         }
    //     })
    //     .collect::<Vec<_>>();

    // Ok(model::Model { meshes, materials })
}


// TODO: fix this. Temporary just to test load_objects
pub fn vTA1<T>(v: Vec<T>) -> [T; 24] where T: Copy {
    let slice = v.as_slice();
    let array: [T; 24] = match slice.try_into() {
        Ok(ba) => ba,
        Err(_) => panic!("Expected a Vec of length {} but it was {}", 24, v.len()),
    };
    array
}

pub fn vTA2<T>(v: Vec<T>) -> [T; 36] where T: Copy {
    let slice = v.as_slice();
    let array: [T; 36] = match slice.try_into() {
        Ok(ba) => ba,
        Err(_) => panic!("Expected a Vec of length {} but it was {}", 36, v.len()),
    };
    array
}