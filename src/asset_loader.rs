use crate::grapics_context::GraphicsContext;
use crate::texture::Texture;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::Worker;
use wgpu::RenderPass;

pub trait AssetBundle{

    fn new(loader: Rc<RefCell<Worker>>) -> Self;
    fn fully_loaded(&mut self) -> bool;

    fn start_loading(&mut self, graphics_context: &GraphicsContext);
}


pub struct AssetManager<A: AssetBundle>{
    loaded_cache: bool,
    asset_bundle: A,
    loading_pipeline: wgpu::RenderPipeline,
    loader: Rc<RefCell<Worker>>
}

impl<A:AssetBundle> AssetManager<A> {
    pub fn new(graphics_context: &GraphicsContext) -> Self<>{
        // create worker
        let loader = Rc::new(RefCell::new(Worker::new("./load_worker.js").unwrap()));
        // asset loading
        let mut asset_bundle = A::new(loader.clone());
        asset_bundle.start_loading(graphics_context);

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
            loader
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




