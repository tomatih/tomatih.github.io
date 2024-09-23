use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    let base = reqwest::Url::parse(&format!("{}/assets/", origin)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let url = format_url(file_name);
    let text = reqwest::get(url)
        .await?
        .text()
        .await?;
    Ok(text)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let url = format_url(file_name);
    let data = reqwest::get(url)
        .await?
        .bytes()
        .await?
        .to_vec();
    Ok(data)
}

pub async fn load_texture(file_name: &str, is_normal_map: bool, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<crate::texture::Texture> {
    let data = load_binary(file_name).await?;
    crate::texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<crate::model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);


    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let texture_name = m.diffuse_texture.unwrap_or("default_diffuse.qoi".to_string());
        let diffuse_texture = load_texture(&texture_name, true, device, queue).await?;
        let texture_name = m.normal_texture.unwrap_or("default_normal.qoi".to_string());
        let normal_texture = load_texture(&texture_name, true, device, queue).await?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(crate::model::Material {
            name: m.name,
            diffuse_texture,
            normal_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    crate::model::ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [
                            m.mesh.texcoords[i * 2],
                            1.0 - m.mesh.texcoords[i * 2 + 1]
                        ],
                        normal: if m.mesh.normals.is_empty() {
                            [0.0, 0.0, 0.0]
                        } else {
                            [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ]
                        },
                        tangent: [0.0; 3],
                        bitangent: [0.0; 3],
                    }
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets.
            for c in indices.chunks_exact(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                vertices[c[0] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // average tangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut vertices[i];
                v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            crate::model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(crate::model::Model { meshes, materials })
}