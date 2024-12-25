use crate::model::{Material, Model};
use crate::texture::Texture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use web_sys::Worker;
use wgpu::RenderPass;
use crate::grapics_context::GraphicsContext;

pub enum Asset {
    Texture(Texture),
    Material(Material),
    Model(Model),
}

pub struct AssetBundle {
    assets: HashMap<String, Asset>,
}

impl AssetBundle {
    pub fn new(assets: HashMap<String, Asset>) -> Self {
        Self { assets }
    }

    //TODO: Add a default missing texture
    pub fn get_texture(&self, name: &str) -> &Texture {
        match self.assets.get(name) {
            None => panic!("Could not find asset: {}", name),
            Some(asset) => match asset {
                Asset::Texture(tex) => tex,
                _ => panic!("{} is not a texture", name),
            },
        }
    }

    //TODO: Add a default missing model
    pub fn get_model(&self, name: &str) -> &Model {
        match self.assets.get(name) {
            None => panic!("Could not find asset: {}", name),
            Some(asset) => match asset {
                Asset::Model(model) => model,
                _ => panic!("{} is not a model", name),
            },
        }
    }

    //TODO: Add a default material
    pub fn get_material(&self, name: &str) -> &Material {
        match self.assets.get(name) {
            None => panic!("Could not find asset: {}", name),
            Some(asset) => match asset {
                Asset::Material(material) => material,
                _ => panic!("{} is not a material", name),
            },
        }
    }
}

pub enum AssetRequest {
    Texture(String),
    Material(String),
    Model(String),
}

struct AssetLoader {
    // Asset management
    asset_requests: HashMap<String, AssetRequest>,
    assets: HashMap<String, AssetBundle>,
    currently_loading: bool,
    // Loader architecture
    loader: Rc<RefCell<Worker>>,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    // Load screen rendering
    loading_pipeline: wgpu::RenderPipeline,
}

impl AssetLoader {
    fn new(asset_requests: HashMap<String, AssetRequest>, graphics_context: &GraphicsContext) -> Self {
        // create worker
        let (tx, rx) = channel::<Vec<u8>>();
        let loader = Rc::new(RefCell::new(Worker::new("./load_worker.js").unwrap()));

        // create loading pipeline
        let loading_pipeline_layout = graphics_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Loading pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let loading_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Loader Shader"), //TODO: add the actual loading shader
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/loading.wgsl").into()),
            };
            crate::wgpu_helpers::create_render_pipeline(
                &graphics_context.device,
                &loading_pipeline_layout,
                graphics_context.config.format,
                None,
                &[],
                shader,
            )
        };

        Self {
            asset_requests,
            assets: HashMap::new(),
            currently_loading: false,
            loader,
            tx,
            rx,
            loading_pipeline
        }
    }

    fn update(&mut self){

    }

    fn render(&mut self, render_pass: &mut RenderPass){
        // render the screen
        render_pass.set_pipeline(&self.loading_pipeline);
        render_pass.draw(0..3, 0..1);
    }

}
