use crate::buffer_primitives::Vertex;
use crate::properties::Color;
use crate::properties::ColorBytes;
use crate::texture;
use std::convert::TryInto;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

static COLOR_BYTE_SIZE: usize = 4; //Color is comprised of 4 bytes, rgba all in u8 form

pub trait Surface {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Color<u8>);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    //vertex buffer and numvertices should be abstracted out
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    pub pixel_surface: PixelSurface,
    pub texture: texture::Texture,
}

pub struct PixelSurface {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl PixelSurface {
    fn new(width: u32, height: u32) -> Self {
        let pixels = vec![
            0;
            (width * height * COLOR_BYTE_SIZE as u32) // 4 color channels
                .try_into()
                .unwrap()
        ];

        Self {
            width,
            height,
            pixels,
        }
    }

    fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl Surface for PixelSurface {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Color<u8>) {
        let color_bytes: ColorBytes = color.get_bytes().unwrap();
        let row_size = self.width * COLOR_BYTE_SIZE as u32;
        let index: usize = (x * COLOR_BYTE_SIZE as u32 + y * row_size)
            .try_into()
            .unwrap();
        let color_slice = self.pixels.get_mut(index..index + COLOR_BYTE_SIZE);
        color_slice
            .unwrap()
            .copy_from_slice(&bytemuck::bytes_of(&color_bytes)[0..COLOR_BYTE_SIZE]);
        // turn Color struct into byte array and copy over
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        //Instance is a handle to our GPU
        //BackendBit is a bitmask defining which backens wgpu will use, PRIMARY enables all of them
        // (Vulkan + Metal + DX12 + Browser WebGPU)
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        //Need to replace this texture loading with a blank texture, which is exposed to be filled by others
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, Some("happy-tree.png"))
                .unwrap();

        let pixel_surface = PixelSurface::new(texture.size.width, texture.size.height);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                //Sampled texture
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    //texture Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        //Splitting bind groups and bind group layouts let's us swap out specific bind groups that are compatible with the layouts we determine
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swap_chain_desc.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        //One triangle that will automatically get clipped to screen size, with a texture stretched to the internal "square" that is actually displayed
        const VERTICES: &[Vertex] = &[
            Vertex {
                tex_coords: [0.0, 0.0],
                position: [-1.0, -1.0, 0.0],
            },
            Vertex {
                tex_coords: [2.0, 0.0],
                position: [3.0, -1.0, 0.0],
            },
            Vertex {
                tex_coords: [0.0, 2.0],
                position: [-1.0, 3.0, 0.0],
            },
        ];

        const INDICES: &[u16] = &[0, 1, 2, 0]; //WGPU needs buffers to be aligned to 4 bytes

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            pixel_surface,
            texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    pub fn update(&mut self) {
        //Copy pixels to texture
        self.texture
            .fill_texture(self.pixel_surface.get_pixels(), &self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            };

            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        } //Scoping for render_pass borrowing encoder mutably with begin_render_pass
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
