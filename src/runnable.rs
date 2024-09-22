use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;


pub trait Runnable<'a> {
    async fn new(window: &'a Window) -> Self;

    fn update(&mut self, dt: instant::Duration);

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    fn resize(&mut self, new_size: PhysicalSize<u32>);

    fn get_size(&self) -> PhysicalSize<u32>;

    fn window(&self) -> &Window;
}

pub async fn run<'a, R: Runnable<'a>>(event_loop: EventLoop<()>, window: &'a Window) {
    let mut app = R::new(window).await;

    let mut last_update_time = instant::Instant::now();
    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == app.window().id() => {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => app.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        let now = instant::Instant::now();
                        let dt = now - last_update_time;
                        last_update_time = now;
                        app.update(dt);
                        match app.render(){
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost) => app.resize(app.get_size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => log::error!("{:?}", e)
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => app.window().request_redraw(),
            _ => {}
        }
    }).unwrap();
}