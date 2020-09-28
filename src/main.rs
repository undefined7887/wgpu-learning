use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
use std::mem;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to build window 123");

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                println!("Redraw requested");
                state.update();
                state.render();
            }

            // Simple event for window
            Event::WindowEvent {
                ref event,
                window_id
            } if window.id() == window_id => {
                // Firstly sending event to state
                if state.input(event) {
                    return;
                }

                // Secondly handling event here
                match event {
                    // When window closes
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }

                    // When user press keyboard button
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        // Closing window if user pressed escape
                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input {
                            *control_flow = ControlFlow::Exit
                        }
                    }

                    // When window is resized
                    WindowEvent::Resized(new_size) => {
                        state.resize(*new_size);
                    }

                    // When user changed display resolution
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            _ => ()
        }
    })
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ]
        }
    }
}

struct State {
    surface: wgpu::Surface,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    num_vertices: u32
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        println!("{} {}", size.width, size.height);
        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
            .await
            .expect("Failed to request adapter");


        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false
                },
                limits: wgpu::Limits::default(),
            }
        ).await;

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let vertex_shader_src = include_str!("shader.vert");
        let fragment_shader_src = include_str!("shader.frag");

        let mut shader_compiler = shaderc::Compiler::new().unwrap();

        let vertex_shader_spirv = shader_compiler
            .compile_into_spirv(
                vertex_shader_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let fragment_shader_spirv = shader_compiler
            .compile_into_spirv(
                fragment_shader_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vertex_shader_data = wgpu::read_spirv(
            std::io::Cursor::new(vertex_shader_spirv.as_binary_u8())
        ).unwrap();

        let fragment_shader_data = wgpu::read_spirv(
            std::io::Cursor::new(fragment_shader_spirv.as_binary_u8())
        ).unwrap();

        let vertex_shader_module = device.create_shader_module(&vertex_shader_data);
        let fragment_shader_module = device.create_shader_module(&fragment_shader_data);

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[]
            }
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                layout: &render_pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vertex_shader_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fragment_shader_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_clamp: 0.0,
                    depth_bias_slope_scale: 0.0,
                }),
                color_states: &[
                    wgpu::ColorStateDescriptor {
                        format: swap_chain_desc.format,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[
                        Vertex::desc(),
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }
        );

        const VERTICES: &[Vertex] = &[
            Vertex { pos: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
        ];

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(VERTICES),
            wgpu::BufferUsage::VERTEX,
        );

        let num_vertices = VERTICES.len() as u32;

        State {
            surface,
            _adapter: adapter,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);

        // Redrawing after resize
        //self.update();
        //self.render();
    }

    // input() won't deal with GPU code, so it can be synchronous
    fn input(&mut self, _: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        let frame = self.swap_chain
            .get_next_texture()
            .expect("Failed to get next frame");

        let mut encoder = self.device
            .create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("My first command encoder")
                }
            );

        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            }
        );

        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..self.num_vertices, 0..1);

        drop(render_pass);

        self.queue.submit(
            &[
                encoder.finish()
            ]
        )
    }
}