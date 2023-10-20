#![allow(dead_code)]
use wgpu::{IndexFormat, PrimitiveTopology, ShaderSource};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use wgpu_simplified as ws;

pub struct Inputs<'a> {
    pub source: ShaderSource<'a>,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
}

impl Inputs<'_> {
    pub async fn new(&mut self, event_loop: EventLoop<()>, window: Window, num_vertices: u32) {   
        let mut init = ws::IWgpuInit::new(&window, 1, None).await;
    
        let shader = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: self.source.clone(),
        });
        
        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let mut ppl = ws::IRenderPipeline{
            shader: Some(&shader), 
            pipeline_layout: Some(&pipeline_layout), 
            is_depth_stencil: false, 
            topology: self.topology,
            strip_index_format:self.strip_index_format,
            ..Default::default()
        };    
        let render_pipeline = ppl.new(&init);

        event_loop.run(move |event, _, control_flow| {       
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Recreate the surface with the new size
                    init.config.width = size.width;
                    init.config.height = size.height;
                    init.surface.configure(&init.device, &init.config);
                }
                Event::RedrawRequested(_) => {
                    let frame = init.surface.get_current_texture().unwrap();
                    
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
                            { label: None });
                    {
                        let color_attachment = ws::create_color_attachment(&view);
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(color_attachment)],
                            depth_stencil_attachment: None,
                        });
                        rpass.set_pipeline(&render_pipeline);
                        rpass.draw(0..num_vertices, 0..1);
                    }

                    init.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }
}
