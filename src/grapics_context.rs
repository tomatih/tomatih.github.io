use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::Window;

pub struct GraphicsContext<'a>{
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: &'a Window,
}

impl<'a> GraphicsContext<'a>{
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device,&config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let window_browser = web_sys::window().unwrap();
            let scaling = window_browser.device_pixel_ratio();

            let limits = self.device.limits();
            let scaled_size = new_size.to_logical(scaling).clamp(
                LogicalSize::new(1, 1),
                LogicalSize::new(limits.max_texture_dimension_2d, limits.max_texture_dimension_2d),
            );

            self.size = new_size;
            self.config.width = scaled_size.width;
            self.config.height = scaled_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

}