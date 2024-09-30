use wgpu::RenderPass;
use crate::grapics_context::GraphicsContext;
use crate::texture::Texture;

pub trait AssetBundle{

    fn new() -> Self;
    fn fully_loaded(&self) -> bool;

    fn start_loading(&mut self);
}


pub struct AssetManager<A: AssetBundle>{
    loaded_cache: bool,
    asset_bundle: A,
    loading_pipeline: wgpu::RenderPipeline
}

impl<A:AssetBundle> AssetManager<A> {
    pub fn new(graphics_context: &GraphicsContext) -> Self<>{
        // asset loading
        let mut asset_bundle = A::new();
        asset_bundle.start_loading();

        // define pipeline
        let loading_pipeline_layout = graphics_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Loading pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let loading_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/loading.wgsl").into()),
            };
            crate::wgpu_helpers::create_render_pipeline(
                &graphics_context.device,
                &loading_pipeline_layout,
                graphics_context.config.format,
                Some(Texture::DEPTH_FORMAT),
                &[],
                shader,
            )
        };

        Self{
            loaded_cache: false,
            asset_bundle,
            loading_pipeline,
        }
    }

    pub fn loaded(&mut self) -> bool{
        if !self.loaded_cache{
            self.loaded_cache = self.asset_bundle.fully_loaded();
        }
        self.loaded_cache
    }

    pub fn render_loading(&mut self, render_pass: &mut RenderPass){
        if self.loaded() {return;}

        render_pass.set_pipeline(&self.loading_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}




