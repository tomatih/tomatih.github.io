mod runnable;
mod wip_page;
mod grapics_context;
mod wgpu_helpers;
mod texture;
mod resources;
mod model;
mod camera;
mod asset_loader;

use wasm_bindgen::prelude::wasm_bindgen;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

fn create_window(event_loop: &EventLoop<()>) -> Window {
    let window_browser = web_sys::window().unwrap();
    let scaling = window_browser.device_pixel_ratio();
    let window_height = window_browser.inner_height().unwrap().as_f64().unwrap() * scaling;
    let window_width = window_browser.inner_width().unwrap().as_f64().unwrap() * scaling;

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(window_width as u32, window_height as u32))
        .with_append(true)
        .build(event_loop)
        .unwrap();

    let _ = window.request_inner_size(PhysicalSize::new(window_width as u32, window_height as u32));

    window
}

#[wasm_bindgen(start)]
fn main() {
    // logging setup
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    wasm_bindgen_futures::spawn_local(async {
        // window setup
        let event_loop = EventLoop::new().unwrap();
        let window = create_window(&event_loop);
        runnable::run::<wip_page::WipPage>(event_loop, &window).await
    });
}