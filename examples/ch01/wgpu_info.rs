use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use wgpu_simplified as ws;

async fn get_wgpu_info(window: &Window) {

    let init = ws::IWgpuInit::new(&window, 1, None).await;

    println!("{:#?}", init.adapter.get_info());
    println!("Adapter{:#?}", init.adapter.limits());
    println!("Device{:#?}", init.device.limits());
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("wgpu_info");
    env_logger::init();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_) => {
                pollster::block_on(get_wgpu_info(&window));
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
