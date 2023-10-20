use bytemuck::{cast_slice, Pod, Zeroable};
use cgmath::Matrix4;
use rand;
use std::{iter, mem};
use wgpu::{util::DeviceExt, VertexBufferLayout};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wgpu_simplified as ws;
use wgpu_fundamentals::vertex_data as vd;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
}

fn create_vertices(r: f32, u: u16, v: u16) -> (Vec<Vertex>, Vec<u16>, Vec<u16>) {
    let (pos, _, _, ind, ind2) = vd::create_sphere_data(r, u, v);
    let mut data: Vec<Vertex> = vec![];
    for i in 0..pos.len() {
        data.push(Vertex { position: pos[i] });
    }
    (data.to_vec(), ind, ind2)
}

struct State {
    init: ws::IWgpuInit,
    pipelines: [wgpu::RenderPipeline; 2],
    vertex_buffer: wgpu::Buffer,
    index_buffers: [wgpu::Buffer; 2],
    uniform_bind_groups: [wgpu::BindGroup; 2],
    uniform_buffers: [wgpu::Buffer; 3],
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    msaa_texture_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,
    indices_lens: [u32; 2],
    plot_type: u32,

    radius: f32,
    u_segments: u16,
    v_segments: u16,
    update_buffers: bool,
    recreate_buffers: bool,
    rotation_speed: f32,
}

impl State {
    async fn new(window: &Window, sample_count: u32) -> Self {
        let init = ws::IWgpuInit::new(&window, sample_count, None).await;

        let shader = init
            .device
            .create_shader_module(wgpu::include_wgsl!("unlit.wgsl"));

        // uniform data
        let camera_position = (3.0, 1.5, 3.0).into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = cgmath::Vector3::unit_y();

        let model_mat = ws::create_model_mat([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let (view_mat, project_mat, vp_mat) = ws::create_vp_mat(
            camera_position,
            look_direction,
            up_direction,
            init.config.width as f32 / init.config.height as f32,
        );
        let mvp_mat = vp_mat * model_mat;

        let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
        let uniform_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(mvp_ref),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // color uniform buffer for object and wireframe
        let color_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice([1.0 as f32, 0.0, 0.0].as_ref()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let color_buffer2 = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice([1.0 as f32, 1.0, 0.0].as_ref()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let (layout, uniform_bind_group) = ws::create_bind_group(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::FRAGMENT],
            &[
                uniform_buffer.as_entire_binding(),
                color_buffer.as_entire_binding(),
            ],
        );

        let (layout2, uniform_bind_group2) = ws::create_bind_group(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::FRAGMENT],
            &[
                uniform_buffer.as_entire_binding(),
                color_buffer2.as_entire_binding(),
            ],
        );

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };

        let mut ppl = ws::IRenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[vertex_buffer_layout],
            ..Default::default()
        };
        let pipeline = ppl.new(&init);

        let pipeline_layout2 =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout 2"),
                    bind_group_layouts: &[&layout2],
                    push_constant_ranges: &[],
                });

        let vertex_buffer_layout2 = VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };

        let mut ppl2 = ws::IRenderPipeline {
            topology: wgpu::PrimitiveTopology::LineList,
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout2),
            vertex_buffer_layout: &[vertex_buffer_layout2],
            ..Default::default()
        };
        let pipeline2 = ppl2.new(&init);

        let msaa_texture_view = ws::create_msaa_texture_view(&init);
        let depth_texture_view = ws::create_depth_view(&init);

        let (vertex_data, index_data, index_data2) = create_vertices(2.0, 20, 30);
        let vertex_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer2 = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data2),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            init,
            pipelines: [pipeline, pipeline2],
            vertex_buffer,
            index_buffers: [index_buffer, index_buffer2],
            uniform_bind_groups: [uniform_bind_group, uniform_bind_group2],
            uniform_buffers: [uniform_buffer, color_buffer, color_buffer2],
            view_mat,
            project_mat,
            msaa_texture_view,
            depth_texture_view,
            indices_lens: [index_data.len() as u32, index_data2.len() as u32],
            plot_type: 0,

            radius: 2.0,
            u_segments: 20,
            v_segments: 30,
            update_buffers: false,
            recreate_buffers: false,
            rotation_speed: 1.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init
                .surface
                .configure(&self.init.device, &self.init.config);

            self.project_mat =
                ws::create_projection_mat(new_size.width as f32 / new_size.height as f32, true);
            self.depth_texture_view = ws::create_depth_view(&self.init);
            if self.init.sample_count > 1 {
                self.msaa_texture_view = ws::create_msaa_texture_view(&self.init);
            }
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match keycode {
                VirtualKeyCode::LControl => {
                    let scolor: [f32; 3] = [rand::random(), rand::random(), rand::random()];
                    self.init.queue.write_buffer(
                        &self.uniform_buffers[1],
                        0,
                        cast_slice(scolor.as_ref()),
                    );
                    true
                }
                VirtualKeyCode::LAlt => {
                    let wcolor: [f32; 3] = [rand::random(), rand::random(), rand::random()];
                    self.init.queue.write_buffer(
                        &self.uniform_buffers[2],
                        0,
                        cast_slice(wcolor.as_ref()),
                    );
                    true
                }
                VirtualKeyCode::Space => {
                    self.plot_type = (self.plot_type + 1) % 3;
                    true
                }
                VirtualKeyCode::Q => {
                    self.radius += 0.1;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::A => {
                    self.radius -= 0.1;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::W => {
                    self.u_segments += 1;
                    self.recreate_buffers = true;
                    true
                }
                VirtualKeyCode::S => {
                    self.u_segments -= 1;
                    self.recreate_buffers = true;
                    true
                }
                VirtualKeyCode::E => {
                    self.v_segments += 1;
                    self.recreate_buffers = true;
                    true
                }
                VirtualKeyCode::D => {
                    self.v_segments -= 1;
                    self.recreate_buffers = true;
                    true
                }
                VirtualKeyCode::R => {
                    self.rotation_speed += 0.1;
                    true
                }
                VirtualKeyCode::F => {
                    self.rotation_speed -= 0.1;
                    if self.rotation_speed < 0.0 {
                        self.rotation_speed = 0.0;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // update uniform buffer
        let dt = self.rotation_speed * dt.as_secs_f32();
        let model_mat =
            ws::create_model_mat([0.0, 0.0, 0.0], [dt.sin(), dt.cos(), 0.0], [1.0, 1.0, 1.0]);
        let mvp_mat = self.project_mat * self.view_mat * model_mat;
        let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
        self.init
            .queue
            .write_buffer(&self.uniform_buffers[0], 0, bytemuck::cast_slice(mvp_ref));

        // update vertex buffer
        if self.update_buffers {
            let (pos, _ind, _ind2) = create_vertices(self.radius, self.u_segments, self.v_segments);
            self.init
                .queue
                .write_buffer(&self.vertex_buffer, 0, cast_slice(&pos));
            self.update_buffers = false;
        }

        // recreate vertex and index buffers
        if self.recreate_buffers {
            let (pos, ind, ind2) = create_vertices(self.radius, self.u_segments, self.v_segments);
            self.indices_lens = [ind.len() as u32, ind2.len() as u32];

            self.vertex_buffer.destroy();
            self.vertex_buffer =
                self.init
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: cast_slice(&pos),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });

            let indices_data = [ind, ind2];
            for i in 0..2 {
                self.index_buffers[i].destroy();
                self.index_buffers[i] =
                    self.init
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: cast_slice(&indices_data[i]),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
            }
            self.recreate_buffers = false;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let color_attach = ws::create_color_attachment(&view);
            let msaa_attach = ws::create_msaa_color_attachment(&view, &self.msaa_texture_view);
            let color_attachment = if self.init.sample_count == 1 {
                color_attach
            } else {
                msaa_attach
            };
            let depth_attachment = ws::create_depth_stencil_attachment(&self.depth_texture_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_attachment),
            });

            let plot_type = if self.plot_type == 1 {
                "shape_only"
            } else if self.plot_type == 2 {
                "wireframe_only"
            } else {
                "both"
            };

            if plot_type == "shape_only" || plot_type == "both" {
                render_pass.set_pipeline(&self.pipelines[0]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffers[0].slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.uniform_bind_groups[0], &[]);
                render_pass.draw_indexed(0..self.indices_lens[0], 0, 0..1);
            }

            if plot_type == "wireframe_only" || plot_type == "both" {
                render_pass.set_pipeline(&self.pipelines[1]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffers[1].slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.uniform_bind_groups[1], &[]);
                render_pass.draw_indexed(0..self.indices_lens[1], 0, 0..1);
            }
        }

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let mut sample_count = 1 as u32;
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        sample_count = args[1].parse::<u32>().unwrap();
    }

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(&*format!("{}_{}", "sphere_wireframe", sample_count));
    let mut state = pollster::block_on(State::new(&window, sample_count));

    let render_start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
            let dt = now - render_start_time;
            state.update(dt);

            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}