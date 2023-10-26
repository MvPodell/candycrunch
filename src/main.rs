// TODO: remove SpriteOption if it doesn't do anything

use bytemuck::{Pod, Zeroable};
use rand::Rng;
// use image::math::Rect;
use image::math::Rect;
use std::{borrow::Cow, mem};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use num::abs;

mod input;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Occupation {
    Empty = 0,
    // White = 1,
    // Light = 2,
    // Dark = 3,
}

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
    //cell_region:[f32; 16],
}

#[derive(Copy, Clone)]
struct Space {
    x_space: f32,
    y_space: f32,
    color: &'static str, // Use a string slice for color
    filled: bool,
}

impl Space {
    fn new(x: f32, y: f32, color: &'static str) -> Self {
        Space {
            x_space: x,
            y_space: y,
            color,
            filled: false,
        }
    }
}

struct GameGrid {
    grid: [[Space; 10]; 20],
}

fn screen_to_grid(x: f32, y: f32) -> (usize, usize) {
    let grid_x = (x as usize - 80) / 8;
    let grid_y = 19 - (y as usize) / 8;
    (grid_x, grid_y)
}

impl GameGrid {
    fn new() -> Self {
        let mut grid = [[Space::new(0.0, 0.0, "empty"); 10]; 20];

        // Initialize the grid with space objects
        let mut y_cord = 518.0;
        let mut x_cord = 98.0;
        for row in 0..20 {
            for col in 0..10 {
                let x = x_cord; // Adjust X coordinate as needed
                let y = y_cord; // Adjust Y coordinate as needed
                grid[row][col] = Space::new(x, y, "nah");
                y_cord -= 88.0;
            }
            x_cord += 98.0;
            y_cord = 518.0;
        }

        GameGrid { grid }
    }

    fn print_grid(&self) {
        for row in &self.grid {
            for space in row {
                if space.filled {
                    print!("1 "); // You can change this to any character or representation for filled spaces
                } else {
                    print!("0 "); // You can change this to any character or representation for empty spaces
                }
            }
            println!();
        }
    }

    fn fill_space(&mut self, x: usize, y: usize, color: &'static str) {
        if x < 10 && y < 20 {
            self.grid[y][x].filled = true;
            self.grid[y][x].color = color;
        }
    }

    fn print_space(&self, x: usize, y: usize) {
        if x < 10 && y < 20 {
            let space = &self.grid[y][x];
            println!(
                "x: {}, y: {}, color: {}, filled: {}",
                x, y, space.color, space.filled
            );
        } else {
            println!("Invalid indices");
        }
    }

    fn point_color(&self, grid_x: usize, grid_y: usize) -> &'static str {
        self.grid[grid_y][grid_x].color
    }

    fn swap_colors(&mut self, x_coord: f32, y_coord: f32, last_clicked: (f32, f32)) {
        let (last_x, last_y) = last_clicked;
        
        let color1 = self.grid[y_coord as usize][x_coord as usize].color;
        let color2 = self.grid[last_y as usize][last_x as usize].color;
        // swap the colors
        self.grid[y_coord as usize][x_coord as usize].color = color2;
        self.grid[last_y as usize][last_x as usize].color = color1;
    }

    fn get_color_coords(&self, sprite_col: usize, sprite_row: usize)-> [f32; 4] {
        let color = match self.grid[sprite_col][sprite_row].color {
            "white" => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0],
            "dark blue" => [0.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "light blue" => [0.0 / 23.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "light orange" => [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "dark orange" => [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "white orange" => [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "black" => [0.0 / 80.0, 96.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            // Add cases for other colors
            _ => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0], // Default to a color
        };
        color
    }

    fn color_is_black(&self, x: usize, y: usize) -> bool {
        if x < 10 && y < 20 {
            let color = self.grid[y][x].color;
            return color == "black";
        } else {
            false
        }
    }

    fn set_black(&mut self, x: usize, y:usize) {
        let color = "black";
        self.grid[x][y].color = color;
        // println!("color after {} for ({}, {})", self.grid[x][y].color, x, y);
    }

    fn check_blackout_horiz(&self) -> (usize, usize, usize) {
        for row in 0..20 {
            let mut consecutive_count = 1;
            let mut last_color = "black";

            for index in 0..10 {
                let space = &self.grid[row][index];
                // println!("{last_color}");
                if space.filled && space.color == last_color && last_color != "black"{
                    consecutive_count += 1;
                    if consecutive_count == 4 {
                        // println!("row{row} index{index}");
                        if index < 4 {
                            return (row + 1, row, index);
                        } else {
                            return (((index-3)*20) + row+1, row, index-3);
                        }  
                    }
                } else {
                    consecutive_count = 1;
                    last_color = space.color;
                }
            }
        }
        // return 202 to signify that there are not 4 in a row
        return (202, 0, 0);
    }
    fn check_blackout_vert(&self) -> (usize, usize, usize) {
        // Check vertically
        for col in 0..10 {
            let mut consecutive_count = 1;
            let mut last_color = "black";

            for row in 0..20 {
                let space = &self.grid[row][col];

                if space.filled && space.color == last_color && last_color != "black"{
                    consecutive_count += 1;
                    if consecutive_count == 4 {
                        // return the first index of the four in a column 
                        return ((row-2) + (20*col), row-3, col);
                    }
                } else {
                    consecutive_count = 1;
                    last_color = space.color;
                }
            }
        }
        // return 202 to signify that there are not 4 in a column
        return (202, 0, 0) // No four consecutive spaces found
    }
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

    let (sprite_tex, _sprite_img) = load_texture("content/blocks7.png", None, &device, &queue)
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
        screen_pos: [80.0, 0.0],
        screen_size: [80.0, 160.0],
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
        GPUSprite {
            screen_region: [0.0, 0.0, 8.0, 8.0],
            sheet_region: [0.0 / 80.0, 0.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
        },
    ];
    // sprite template
    let sprite1 = GPUSprite {
        screen_region: [0.0, 0.0, 8.0, 8.0],
        sheet_region: [0.0 / 80.0, 0.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
    };

    let mut input = input::Input::default();
    let mut game_grid = GameGrid::new();
    // Initialize color counters
    let mut color_counters: [u32; 6] = [0; 6];

    let mut x: f32 = 80.0;
    let mut y: f32 = 152.0;

    // loop through every slot in the grid
    for _col in 0..10 {
        for _row in 0..20 {
            // generate a random number between 0 and 2. This number will be used to select the color of the sprite
            let mut rng = rand::thread_rng();
            let mut random_number: u32 = rng.gen_range(0..=5);
            let mut sprite: GPUSprite = sprite1;

            // convert pixels to grid units
            let (grid_x, grid_y) = screen_to_grid(x, y);

            // println!("{}", random_number);

            // Check if placing more than three of the same color in a row
            if color_counters[random_number as usize] >= 3 {
                // If more than three, select a different color
                let mut new_random_number = random_number;
                while new_random_number == random_number {
                    new_random_number = rng.gen_range(0..=5);
                }
                random_number = new_random_number;
            }

            // Increment the counter for the chosen color
            color_counters[random_number as usize] += 1;

            // Reset the counters for other colors
            for i in 0..6 {
                if i != random_number as usize {
                    color_counters[i] = 0;
                }
            }

            // set the color of the sprite according to the random number generated
            match random_number {
                0 => {
                    // set the color of the sprite to white
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 0.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "white");
                }
                1 => {
                    // set the color of the sprite to dark blue
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "dark blue");
                }
                2 => {
                    // set the color of the sprite to light blue
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "light blue");
                }
                3 => {
                    // set the color of the sprite to light orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "light orange");
                }
                4 => {
                    // set the color of the sprite to dark orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "dark orange");
                }
                5 => {
                    // set the color of the sprite to white with orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "white orange");
                }
                _ => println!("Random number is out of range"),
            }

            // game_grid.print_space(grid_x, grid_y);
            sprites.push(sprite);
            y -= 8.0;
        }

        x += 8.0;
        y = 152.0;
    }

    let mut counter = 0;
    let mut last_clicked = (x, y);
    let mut last_cell_clicked = 0;
    let mut last_y = 0.0;
    let mut last_x = 0.0;
    let mut color1 = [0.0,0.0,0.0,0.0];
    let mut color2 = [0.0,0.0,0.0,0.0];
    let mut score = 0;

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
                if input.is_key_pressed(winit::event::VirtualKeyCode::Down) {
                    game_grid.print_space(0, 0);
                }

                if input.is_key_pressed(winit::event::VirtualKeyCode::Up) {
                    game_grid.print_grid();
                }

                if input.is_mouse_released(winit::event::MouseButton::Left) {
                    let mouse_pos = input.mouse_pos();
                    let (mouse_x_norm, mouse_y_norm) = ((mouse_pos.x / 32.0),
                                                        (mouse_pos.y / 32.0));  
                    

                    // in these calculations, mouse_x_norm is the column, mouse_y_norm is the row
                    let row = mouse_y_norm.floor() as usize;
                    let column = mouse_x_norm.floor() as usize;
                    // println!("Current color: {}", game_grid.point_color(column, row));

                    // game_grid.print_space(1, 1);
                    // check for swap
                    // if the counter is even, then save the clicked coords
                    if counter % 2 == 0 {
                        last_clicked = (mouse_x_norm.floor() as f32, mouse_y_norm.floor() as f32);
                        last_cell_clicked = column * 20 + row + 1;
                    } 
                    // if counter is odd, then swap current click with saved coords
                    else {
                        let (last_x, last_y) = last_clicked;
                        if !game_grid.color_is_black(last_x as usize, last_y as usize) && !game_grid.color_is_black(column, row){
                            println!("swap colors!");
                            // swap the sprite locations
                            let curr_cell = column * 20 + row + 1;
                            let diff = curr_cell as f32 - last_cell_clicked as f32;

                            // only swap if they are one apart
                            if abs(diff) == 1.0 || abs(diff) == 20.0 {
                                // get colors
                                // (last_x, last_y) = last_clicked;
                                color1 = game_grid.get_color_coords(mouse_y_norm.floor() as usize, mouse_x_norm.floor() as usize,);
                                color2 = game_grid.get_color_coords(last_y as usize, last_x as usize);
                                
                                // update the colors in the sprites vec
                                sprites[curr_cell as usize].sheet_region = color2;
                                sprites[last_cell_clicked as usize].sheet_region = color1;

                                // update the colors in the grid
                                game_grid.swap_colors(mouse_x_norm.floor() as f32, mouse_y_norm.floor() as f32, last_clicked);
                            } else {
                                println!("Invalid click! Can only click tiles one apart.");
                            }
                        } else {
                            println!("Invalid click! Cannot swap a tile with nothing!");
                        }
                        


                        let (blackout_horiz, start_x, mut start_y) = game_grid.check_blackout_horiz();
                        if blackout_horiz != 202 {
                            // println!("Four in a row starting at {blackout_horiz}!");
                            // set the four in a row to black
                            // println!("Black region starts at {}", blackout_horiz);
                            for i in (blackout_horiz..(blackout_horiz + 80)).step_by(20) {
                                // set the sprite in the vec to black
                                sprites[i].sheet_region = [0.0 / 80.0, 96.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0];
                                // set the color of the sprite in the grid to black
                                game_grid.set_black(start_x, start_y);
                                start_y+=1;
                                // println!("Black? {}", game_grid.color_is_black(start_x, start_y));
                                
                            }
                            score+=4;
                        } 
                        let (blackout_vert, mut start_x, start_y) = game_grid.check_blackout_vert();
                        if blackout_vert != 202 {
                            // println!("Four in a column starting at {blackout_vert}!");
                            // set the four in a column to black 
                            for i in blackout_vert..(blackout_vert + 4) {
                                // set the sprite in the vec to black
                                sprites[i].sheet_region = [0.0 / 80.0, 96.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0];
                                // set the color of the sprite in the grid to black
                                // println!("{start_x}     {start_y}");
                                game_grid.set_black(start_x, start_y);
                                start_x += 1;
                            }  
                            score+=4; 
                        }
                    }
                    counter+=1;
                    println!("Current score: {score}");
                }
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
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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

                    rpass.draw(0..6, 0..201);
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

    // Define the desired aspect ratio
    let aspect_ratio = 80.0 / 160.0;

    // Set a fixed width for the window
    let window_width = 320.0; // For example, you can use 320 pixels width

    // Calculate the corresponding height based on the aspect ratio
    let window_height = window_width / aspect_ratio;

    // Create the window with the calculated dimensions
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
        .build(&event_loop)
        .unwrap();

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

// fn update_colors(sprite_1_index: usize, sprite_1_row: usize, sprite_1_col: usize, sprite_2_index: usize, sprite_2_row: usize, sprite_2_col: usize, game_grid: GameGrid, sprites: &mut Vec<GPUSprite>) {
//     let color1 = match game_grid.grid[sprite_1_row][sprite_1_col].color {
//         "white" => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0],
//         "dark blue" => [0.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "light blue" => [0.0 / 23.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "light orange" => [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "dark orange" => [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "white orange" => [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         // Add cases for other colors
//         _ => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0], // Default to a color
//     };

//     let color2 = match game_grid.grid[sprite_2_row][sprite_2_col].color {
//         "white" => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0],
//         "dark blue" => [0.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "light blue" => [0.0 / 23.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "light orange" => [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "dark orange" => [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         "white orange" => [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
//         // Add cases for other colors
//         _ => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0], // Default to a color
//     };

//     // Update the color of the first sprite
//     sprites[sprite_1_index].sheet_region = color1;
//     // Update the color of the second sprite
//     sprites[sprite_2_index].sheet_region = color2;
// }