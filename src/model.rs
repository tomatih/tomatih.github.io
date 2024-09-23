use std::ops::Range;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{BindGroup, VertexBufferLayout};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: crate::texture::Texture,
    pub normal_texture: crate::texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material, camera_bind_group: &'a wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, material: &'a Material, camera_bind_group: &'a wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup, instances: Range<u32>);

    fn draw_model(&mut self, model: &'a Model, camera_bind_group: &'a wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup);
    fn draw_model_instanced(&mut self, model: &'a Model, camera_bind_group: &'a wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, camera_bind_group, light_bind_group, 0..1)
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'a wgpu::BindGroup, instances: Range<u32>) {
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &camera_bind_group, &[]);
        self.set_bind_group(2, &light_bind_group, &[]);
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b BindGroup, light_bind_group: &'a wgpu::BindGroup) {
        self.draw_model_instanced(model, camera_bind_group, light_bind_group, 0..1);
    }

    fn draw_model_instanced(&mut self, model: &'b Model, camera_bind_group: &'b BindGroup, light_bind_group: &'a wgpu::BindGroup, instances: Range<u32>) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, camera_bind_group, light_bind_group, instances.clone());
        }
    }
}


pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model 4x4
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // normal 3x3
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
