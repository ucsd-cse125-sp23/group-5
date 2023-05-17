use anyhow::Context;
use std::fs::{read, read_to_string};
use std::io::{BufReader, Cursor};
use std::sync::Arc;

extern crate nalgebra_glm as glm;

use const_format::formatcp;
use wgpu::util::DeviceExt;

use crate::{
    model::{self},
    texture,
};

//assuming we run from root (group-5 folder)
#[rustfmt::skip]
const SEARCH_PATH : [&str; 4] = [
    formatcp!(""), 
    formatcp!("assets"), 
    formatcp!("client{}res", std::path::MAIN_SEPARATOR_STR), 
    formatcp!("client{}res{}textures", std::path::MAIN_SEPARATOR_STR, std::path::MAIN_SEPARATOR_STR),
];

// #[cfg(target_arch = "wasm32")]
// fn format_url(file_name: &str) -> reqwest::Url {
//     let window = web_sys::window().unwrap();
//     let location = window.location();
//     let mut origin = location.origin().unwrap();
//     if !origin.ends_with("learn-wgpu") {
//         origin = format!("{}/learn-wgpu", origin);
//     }
//     let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
//     base.join(file_name).unwrap()
// }

pub fn find_in_search_path(file_name: &str) -> Option<std::path::PathBuf> {
    for p in SEARCH_PATH {
        let path = std::path::Path::new(p).join(file_name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = find_in_search_path(file_name)
        .ok_or_else(|| anyhow::Error::msg(format!("error finding {file_name}")))?;

    read_to_string(path).map_err(|e| anyhow::Error::msg(format!("error reading {file_name}: {e}")))
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = find_in_search_path(file_name)
        .ok_or_else(|| anyhow::Error::msg(format!("error finding {file_name}")))?;

    read(path).map_err(|e| anyhow::Error::msg(format!("error reading {file_name}: {e}")))
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name)
        .await
        .context(format!("error loading texture binary {file_name}"))?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub type ModelLoadingResources<'a> = (&'a wgpu::Device, &'a wgpu::Queue, &'a wgpu::BindGroupLayout);

pub async fn load_model(
    file_path: &str,
    resources: ModelLoadingResources<'_>,
) -> anyhow::Result<model::StaticModel> {
    println!("loading {file_path:?}");
    let (device, queue, layout) = resources;
    let obj_text = load_string(file_path).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p: String| async move {
            // p is relative to the obj file folder
            let p = std::path::Path::new(file_path)
                .parent()
                .unwrap()
                .join(p)
                .to_str()
                .unwrap()
                .to_string();
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await
    .context("error loading obj file")?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        // println!("Material: {m:?}");
        let phong_mtl = model::Phong::new(&m);
        let phong_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Phong VB"),
            contents: bytemuck::cast_slice(&[phong_mtl]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let mut flags: u32 = 0;
        let diffuse_texture = match m.diffuse_texture.as_str() {
            "" => texture::Texture::dummy(device),
            d => {
                flags |= model::ShaderFlags::HAS_DIFFUSE_TEXTURE;
                load_texture(d, device, queue).await?
            }
        };
        let normal_texture = match m.normal_texture.as_str() {
            "" => texture::Texture::dummy(device),
            d => {
                flags |= model::ShaderFlags::HAS_NORMAL_TEXTURE;
                load_texture(d, device, queue).await?
            }
        };
        let specular_texture = match m.specular_texture.as_str() {
            "" => texture::Texture::dummy(device),
            d => {
                flags |= model::ShaderFlags::HAS_SPECULAR_TEXTURE;
                load_texture(d, device, queue).await?
            }
        };
        let ambient_texture = match m.ambient_texture.as_str() {
            "" => texture::Texture::dummy(device),
            d => {
                flags |= model::ShaderFlags::HAS_AMBIENT_TEXTURE;
                load_texture(d, device, queue).await?
            }
        };
        let shininess_texture = match m.shininess_texture.as_str() {
            "" => texture::Texture::dummy(device),
            d => {
                flags |= model::ShaderFlags::HAS_SHININESS_TEXTURE;
                load_texture(d, device, queue).await?
            }
        };
        let flags_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Flags VB"),
            contents: bytemuck::cast_slice(&[flags]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: phong_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: flags_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&specular_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&specular_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&ambient_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&ambient_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&shininess_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::Sampler(&shininess_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            normal_texture,
            specular_texture,
            ambient_texture,
            shininess_texture,
            phong_mtl,
            flags: model::ShaderFlags::new(flags),
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // Placeholders
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: glm::Vec3 = v0.position.into();
                let pos1: glm::Vec3 = v1.position.into();
                let pos2: glm::Vec3 = v2.position.into();

                let uv0: glm::Vec2 = v0.tex_coords.into();
                let uv1: glm::Vec2 = v1.tex_coords.into();
                let uv2: glm::Vec2 = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + glm::Vec3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + glm::Vec3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + glm::Vec3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + glm::Vec3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + glm::Vec3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + glm::Vec3::from(vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (glm::Vec3::from(v.tangent) * denom).into();
                v.bitangent = (glm::Vec3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_path)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_path)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: m.name,
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::StaticModel {
        meshes: Arc::new(meshes),
        materials: Arc::new(materials),
        path: file_path.to_string(),
    })
}
