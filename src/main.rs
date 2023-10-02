// TODO: remove SpriteOption if it doesn't do anything

use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, mem};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod input;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct GPUCamera {
    screen_pos: [f32; 2],
    screen_size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct GPUSprite {
    screen_region: [f32; 4],
    sheet_region: [f32; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SpriteOption {
    Storage,
    Uniform,
    VertexBuffer,
}

#[cfg(all(not(feature = "uniforms"), not(feature = "vbuf")))]
const SPRITES: SpriteOption = SpriteOption::Storage;
#[cfg(feature = "uniforms")]
const SPRITES: SpriteOption = SpriteOption::Uniform;
#[cfg(feature = "vbuf")]
const SPRITES: SpriteOption = SpriteOption::VertexBuffer;
#[cfg(all(feature = "vbuf", feature = "uniform"))]
compile_error!("Can't choose both vbuf and uniform sprite features");

// #[cfg(not(feature = "webgl"))]
// const USE_STORAGE: bool = true;
// #[cfg(feature = "webgl")]
// const USE_STORAGE: bool = false;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    log::info!("Use sprite mode {:?}", SPRITES);

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if SPRITES == SpriteOption::Storage {
                    wgpu::Limits::downlevel_defaults()
                } else {
                    wgpu::Limits::downlevel_webgl2_defaults()
                }
                .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    if SPRITES == SpriteOption::Storage {
        let supports_storage_resources = adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::VERTEX_STORAGE)
            && device.limits().max_storage_buffers_per_shader_stage > 0;
        assert!(supports_storage_resources, "Storage buffers not supported");
    }
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            // It needs the first entry for the texture and the second for the sampler.
            // This is like defining a type signature.
            entries: &[
                // The texture binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 0,
                    // Only available in the fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // It's a texture binding
                    ty: wgpu::BindingType::Texture {
                        // We can use it with float samplers
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        // It's being used as a 2D texture
                        view_dimension: wgpu::TextureViewDimension::D2,
                        // This is not a multisampled texture
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 1,
                    // Only available in the fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // It's a sampler
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    // No count
                    count: None,
                },
            ],
        });
    // The camera binding
    let camera_layout_entry = wgpu::BindGroupLayoutEntry {
        // This matches the binding in the shader
        binding: 0,
        // Available in vertex shader
        visibility: wgpu::ShaderStages::VERTEX,
        // It's a buffer
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        // No count, not a buffer array binding
        count: None,
    };
    let sprite_bind_group_layout = match SPRITES {
        SpriteOption::Storage => {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    camera_layout_entry,
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding in the shader
                        binding: 1,
                        // Available in vertex shader
                        visibility: wgpu::ShaderStages::VERTEX,
                        // It's a buffer
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        // No count, not a buffer array binding
                        count: None,
                    },
                ],
            })
        }
        SpriteOption::Uniform => {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    camera_layout_entry,
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding in the shader
                        binding: 1,
                        // Available in vertex shader
                        visibility: wgpu::ShaderStages::VERTEX,
                        // It's a buffer
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(SPRITE_UNIFORM_SIZE),
                        },
                        // No count, not a buffer array binding
                        count: None,
                    },
                ],
            })
        }
        SpriteOption::VertexBuffer => {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[camera_layout_entry],
            })
        }
    };
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&sprite_bind_group_layout, &texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: match SPRITES {
                SpriteOption::Storage => "vs_storage_main",
                SpriteOption::Uniform => "vs_uniform_main",
                SpriteOption::VertexBuffer => "vs_vbuf_main",
            },
            buffers: match SPRITES {
                SpriteOption::VertexBuffer => &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GPUSprite>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[f32; 4]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
                _ => &[],
            },
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let (sprite_tex, _sprite_img) = load_texture("content/blocks.png", None, &device, &queue)
        .await
        .expect("Couldn't load spritesheet texture");
    let view_sprite = sprite_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler_sprite = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_bind_group_layout,
        entries: &[
            // One for the texture, one for the sampler
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view_sprite),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler_sprite),
            },
        ],
    });
    let camera = GPUCamera {
        screen_pos: [0.0, 0.0],
        screen_size: [256.0, 192.0],
    };
    let buffer_camera = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytemuck::bytes_of(&camera).len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut sprites: Vec<GPUSprite> = vec![
        // these sprites initial locations are determined by sprite_position_x
        // screen_region [x,y,z,w] = top left corner x, top left corner y, width, height
        // sheet_region [x,y,z,w] = divided by spritesheet width, divided by spritesheet height, divided by spritesheet width, divided by spritesheet height, divided by spritesheet width, divided by spritesheet height,
        // T1
        GPUSprite {
            screen_region: [0.0, 4.0, 24.0, 16.0],
            sheet_region: [0.0, 4.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        }, 
        // T2
        GPUSprite {
            screen_region: [36.0, 0.0, 16.0, 24.0],
            sheet_region: [36.0 / 123.0, 0.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // T3
        GPUSprite {
            screen_region: [64.0, 4.0, 24.0, 16.0],
            sheet_region: [64.0 / 123.0, 4.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // T4
        GPUSprite {
            screen_region: [100.0, 0.0, 16.0, 24.0],
            sheet_region: [100.0 / 123.0, 0.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // sideways L0
        GPUSprite {
            screen_region: [0.0, 36.0, 24.0, 16.0],
            sheet_region: [0.0, 36.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // sideways L1
        GPUSprite {
            screen_region: [36.0, 32.0, 16.0, 24.0],
            sheet_region: [36.0 / 123.0, 32.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // sideways L2
        GPUSprite {
            screen_region: [64.0, 36.0, 24.0, 16.0],
            sheet_region: [64.0 / 123.0, 36.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // sideways L3
        GPUSprite {
            screen_region: [100.0, 32.0, 16.0, 24.0],
            sheet_region: [100.0 / 123.0, 32.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // squiggle 0
        GPUSprite {
            screen_region: [0.0, 68.0, 24.0, 16.0],
            sheet_region: [0.0, 68.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // squiggle 1
        GPUSprite {
            screen_region: [36.0, 64.0, 16.0, 24.0],
            sheet_region: [0.0 / 123.0, 64.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // squiggle 2
        GPUSprite {
            screen_region: [64.0, 68.0, 24.0, 16.0],
            sheet_region: [64.0 /123.0, 68.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // squiggle 3
        GPUSprite {
            screen_region: [100.0, 64.0, 24.0, 16.0],
            sheet_region: [100.0 / 123.0, 64.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // square 0
        GPUSprite {
            screen_region: [4.0, 100.0, 16.0, 16.0],
            sheet_region: [4.0 / 123.0, 100.0 / 224.0, 16.0 / 123.0, 16.0 / 224.0],
        },
        // square 1
        GPUSprite {
            screen_region: [36.0, 100.0, 16.0, 16.0],
            sheet_region: [36.0 / 123.0, 100.0 / 224.0, 16.0 / 123.0, 16.0 / 224.0],
        },
        // square 2
        GPUSprite {
            screen_region: [68.0, 100.0, 16.0, 16.0],
            sheet_region: [68.0 / 123.0, 100.0 / 224.0, 16.0 / 123.0, 16.0 / 224.0],
        },
        // square 3
        GPUSprite {
            screen_region: [100.0, 100.0, 16.0, 16.0],
            sheet_region: [100.0 / 123.0, 100.0 / 224.0, 16.0 / 123.0, 16.0 / 224.0],
        },
        // dark squiggle 0
        GPUSprite {
            screen_region: [0.0, 132.0, 24.0, 16.0],
            sheet_region: [0.0 / 123.0, 132.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // dark squiggle 1
        GPUSprite {
            screen_region: [36.0, 128.0, 16.0, 24.0],
            sheet_region: [36.0 / 123.0, 128.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // dark squiggle 2
        GPUSprite {
            screen_region: [64.0, 132.0, 24.0, 16.0],
            sheet_region: [64.0 /123.0, 132.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // dark squiggle 3
        GPUSprite {
            screen_region: [100.0, 128.0, 24.0, 16.0],
            sheet_region: [100.0 / 123.0, 128.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // light sideways L0
        GPUSprite {
            screen_region: [0.0, 164.0, 24.0, 16.0],
            sheet_region: [0.0, 164.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // light sideways L1
        GPUSprite {
            screen_region: [36.0, 160.0, 16.0, 24.0],
            sheet_region: [36.0 / 123.0, 160.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // light sideways L2
        GPUSprite {
            screen_region: [64.0, 164.0, 24.0, 16.0],
            sheet_region: [64.0 / 123.0, 164.0 / 224.0, 24.0 / 123.0, 16.0 / 224.0],
        },
        // light sideways L3
        GPUSprite {
            screen_region: [100.0, 160.0, 16.0, 24.0],
            sheet_region: [100.0 / 123.0, 160.0 / 224.0, 16.0 / 123.0, 24.0 / 224.0],
        },
        // straight 0
        GPUSprite {
            screen_region: [8.0, 188.0, 8.0, 32.0],
            sheet_region: [8.0 / 123.0, 188.0 / 224.0, 8.0 / 123.0, 32.0 / 224.0],
        },
        // straight 1
        GPUSprite {
            screen_region: [28.0, 200.0, 32.0, 8.0],
            sheet_region: [28.0 / 123.0, 200.0 / 224.0, 32.0 / 123.0, 8.0 / 224.0],
        },
        // straight 2
        GPUSprite {
            screen_region: [72.0, 188.0, 8.0, 32.0],
            sheet_region: [72.0 / 123.0, 188.0 / 224.0, 8.0 / 123.0, 32.0 / 224.0],
        },
        // straight 3
        GPUSprite {
            screen_region: [91.0, 200.0, 32.0, 8.0],
            sheet_region: [91.0 / 123.0, 200.0 / 224.0, 32.0 / 123.0, 8.0 / 224.0],
        },
    ];

    let window_width = config.width as f32;
    let window_height = config.height as f32;

    // here divide by a number to create the number of grids
    let cell_width = window_width / 80.0;
    let cell_height = window_height / 80.0;

    // Initialize sprite positions within the grid
    let mut sprite_position: [f32; 2] = [75.0, 192.0];

    // current sprite
    let mut curr_sprite_index = 0;

    // initialize vertical position
    let mut vertical_position: f32 = 0.0;
    let scroll_speed: f32 = 0.2;

    const SPRITE_UNIFORM_SIZE: u64 = 512 * mem::size_of::<GPUSprite>() as u64;

    let buffer_sprite = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: if SPRITES == SpriteOption::Uniform {
            SPRITE_UNIFORM_SIZE
        } else {
            sprites.len() as u64 * std::mem::size_of::<GPUSprite>() as u64
        },
        usage: match SPRITES {
            SpriteOption::Storage => wgpu::BufferUsages::STORAGE,
            SpriteOption::Uniform => wgpu::BufferUsages::UNIFORM,
            SpriteOption::VertexBuffer => wgpu::BufferUsages::VERTEX,
        } | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let sprite_bind_group = match SPRITES {
        SpriteOption::VertexBuffer => device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_camera.as_entire_binding(),
            }],
        }),
        _ => device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_camera.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_sprite.as_entire_binding(),
                },
            ],
        }),
    };

    queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
    queue.write_buffer(&buffer_sprite, 0, bytemuck::cast_slice(&sprites));
    let mut input = input::Input::default();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if input.is_key_pressed(winit::event::VirtualKeyCode::Space) {
                    curr_sprite_index += 1;
                    if curr_sprite_index % 4 == 0 {
                        curr_sprite_index -= 4
                    }

                }
                if input.is_key_pressed(winit::event::VirtualKeyCode::Left) {
                    sprite_position[0] -= cell_width;

                    if sprite_position[0] < 0.0 {
                        sprite_position[0] = 0.0;
                    }
                }

                if input.is_key_pressed(winit::event::VirtualKeyCode::Right) {
                    sprite_position[0] += cell_width;

                    if sprite_position[0] < window_width {
                        sprite_position[0] += cell_width;
                    }
                }

                //update sprite position
                sprites[curr_sprite_index].screen_region[0] = sprite_position[0];
                sprites[curr_sprite_index].screen_region[1] = sprite_position[1];

                vertical_position += scroll_speed; // You can adjust the scroll speed as needed

                // check if the piece has hit the bottom of the screen
                if vertical_position > camera.screen_size[1] {
                    let difference = 4 - (curr_sprite_index % 4);
                    curr_sprite_index += difference;

                    if curr_sprite_index >= sprites.len() {
                        curr_sprite_index = 0; // Wrap around to the first sprite
                    }
                    sprites[curr_sprite_index].screen_region[1] = camera.screen_size[1];
                    vertical_position = 0.0;
                }
                // Update the Y-coordinate of each sprite
                sprites[curr_sprite_index].screen_region[1] = sprites[curr_sprite_index].screen_region[1] - vertical_position;
                
                // Then send the data to the GPU!
                input.next_frame();

                queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
                queue.write_buffer(&buffer_sprite, 0, bytemuck::cast_slice(&sprites));

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    if SPRITES == SpriteOption::VertexBuffer {
                        rpass.set_vertex_buffer(0, buffer_sprite.slice(..));
                    }
                    rpass.set_bind_group(0, &sprite_bind_group, &[]);
                    rpass.set_bind_group(1, &texture_bind_group, &[]);
                    // draw two triangles per sprite, and sprite at the current index.
                    // this uses instanced drawing, but it would also be okay
                    // to draw 6 * sprites.len() vertices and use modular arithmetic
                    // to figure out which sprite we're drawing.
                    rpass.draw(0..6, (curr_sprite_index as u32)..(curr_sprite_index as u32)+1);
                }
                queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // WindowEvent->KeyboardInput: Keyboard input!
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event: WindowEvent::KeyboardInput { input: key_ev, .. },
                ..
            } => {
                input.handle_key_event(key_ev);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                input.handle_mouse_button(state, button);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input.handle_mouse_move(position);
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}

async fn load_texture(
    path: impl AsRef<std::path::Path>,
    label: Option<&str>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<(wgpu::Texture, image::RgbaImage), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    let img = {
        let fetch = web_sys::window()
            .map(|win| win.fetch_with_str(path.as_ref().to_str().unwrap()))
            .unwrap();
        let resp: web_sys::Response = wasm_bindgen_futures::JsFuture::from(fetch)
            .await
            .unwrap()
            .into();
        log::debug!("{:?} {:?}", &resp, resp.status());
        let buf: js_sys::ArrayBuffer =
            wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap())
                .await
                .unwrap()
                .into();
        log::debug!("{:?} {:?}", &buf, buf.byte_length());
        let u8arr = js_sys::Uint8Array::new(&buf);
        log::debug!("{:?}, {:?}", &u8arr, u8arr.length());
        let mut bytes = vec![0; u8arr.length() as usize];
        log::debug!("{:?}", &bytes);
        u8arr.copy_to(&mut bytes);
        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?
            .to_rgba8()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let img = image::open(path.as_ref())?.to_rgba8();
    let (width, height) = img.dimensions();
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );
    Ok((texture, img))
}
