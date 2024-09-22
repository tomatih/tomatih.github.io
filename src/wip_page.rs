use std::time::Duration;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct WipPage<'a> {
    window: &'a Window,
}

impl<'a> crate::runnable::Runnable<'a> for WipPage<'a> {
    async fn new(window: &'a Window) -> Self{
        Self{
            window
        }
    }

    fn update(&mut self, dt: Duration) {
        todo!()
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        todo!()
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        todo!()
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize{
            width: 0,
            height: 0,
        }
    }

    fn window(&self) -> &Window {
        self.window
    }
}