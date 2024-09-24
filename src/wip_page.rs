use crate::camera;
use crate::grapics_context::GraphicsContext;
use std::time::Duration;
use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, One, Quaternion, Rotation3, Vector3};
use wgpu::util::DeviceExt;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::model::{DrawModel, Instance, Vertex};
use crate::texture::Texture;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct LightUniform {
    position: [f32; 3],
    _padding: f32,
    color: [f32; 3],
    _padding2: f32,
}

pub struct WipPage<'a> {
    graphics_context: GraphicsContext<'a>,
    render_pipeline: wgpu::RenderPipeline,
    camera: crate::camera::Camera,
    projection: camera::Projection,
    camera_uniform: crate::camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    obj_model: crate::model::Model,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl<'a> crate::runnable::Runnable<'a> for WipPage<'a> {
    async fn new(window: &'a Window) -> Self {
        // graphics basics
        let graphics_context = GraphicsContext::new(window).await;

        // texture setup
        let texture_bind_group_layout = graphics_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        // camera setup
        let camera = camera::Camera::new((0.0, 1.0, 2.5), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(graphics_context.config.width, graphics_context.config.height, cgmath::Deg(45.0), 0.1, 100.0);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = graphics_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = graphics_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera bing group layout"),
        });

        let camera_bind_group = graphics_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera bind group"),
        });

        // light setup
        let light_uniform = LightUniform {
            position: [-5.0, 0.0, -5.0],
            _padding: 0.0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
        };

        let light_buffer = graphics_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_bind_group_layout = graphics_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: None,
            }
        );

        let light_bind_group = graphics_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                }
            ],
            label: None,
        });


        // render pipeline
        let render_pipeline_layout = graphics_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/wip.wgsl").into()),
            };
            crate::wgpu_helpers::create_render_pipeline(
                &graphics_context.device,
                &render_pipeline_layout,
                graphics_context.config.format,
                Some(Texture::DEPTH_FORMAT),
                &[crate::model::ModelVertex::desc(), crate::model::InstanceRaw::desc()],
                shader,
            )
        };

        // Depth texture
        let depth_texture = Texture::create_depth_texture(&graphics_context.device, &graphics_context.config, "depth_texture");

        // Model
        let obj_model = crate::resources::load_model("WIP.obj", &graphics_context.device, &graphics_context.queue, &texture_bind_group_layout).await.unwrap();

        // instances
        let instances = vec![Instance{
            position: Vector3{x: 0.0, y: 0.0, z: 0.0},
            rotation: Quaternion::one()
        }];
        let instances_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = graphics_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instances_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );


        Self {
            graphics_context,
            render_pipeline,
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            light_uniform,
            light_buffer,
            light_bind_group,
            obj_model,
            instances,
            instance_buffer,
        }
    }

    fn update(&mut self, dt: Duration) {
        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle(Vector3::unit_x(), cgmath::Deg(60.0 * dt.as_secs_f32()))
                * old_position)
                .into();
        self.graphics_context.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.graphics_context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.graphics_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.012,
                            g: 0.627,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment:  Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });


            use crate::model::DrawModel;
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(&self.obj_model, &self.camera_bind_group, &self.light_bind_group, 0..1);
        }

        self.graphics_context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.graphics_context.resize(new_size);
        self.depth_texture = Texture::create_depth_texture(&self.graphics_context.device, &self.graphics_context.config, "depth_texture");
        self.projection.resize(self.graphics_context.config.width, self.graphics_context.config.height);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.graphics_context.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        self.graphics_context.size
    }

    fn window(&self) -> &Window {
        self.graphics_context.window
    }
}