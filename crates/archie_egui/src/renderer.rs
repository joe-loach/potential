use std::collections::HashMap;
use std::num::NonZeroU32;

use archie::wgpu::util::{BufferInitDescriptor, DeviceExt};
use archie::wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Transform {
    scale: (f32, f32),
    translation: (f32, f32),
}

impl Transform {
    fn new(window_size: (f32, f32), scale_factor: f32) -> Self {
        Self {
            scale: (
                2.0 / (window_size.0 / scale_factor),
                -2.0 / (window_size.1 / scale_factor),
            ),
            translation: (-1.0, 1.0),
        }
    }
}

unsafe impl bytemuck::Pod for Transform {}
unsafe impl bytemuck::Zeroable for Transform {}

struct DrawCommand {
    vertices: usize,
    texture_id: egui::TextureId,
    clip: (u32, u32, u32, u32),
}

pub struct Renderer {
    pipeline: Pipeline,
    textures: HashMap<egui::TextureId, Texture>,
    vertex_data: Vec<u8>,
    vertex_buffer: Buffer,
    vertex_buffer_capacity: usize,
    index_data: Vec<u8>,
    index_buffer: Buffer,
    index_buffer_capacity: usize,
}

impl Renderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let pipeline = Pipeline::new(device, format);
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("egui vertex buffer"),
            size: 0,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("egui index buffer"),
            size: 0,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            textures: HashMap::default(),
            vertex_data: Vec::new(),
            vertex_buffer,
            vertex_buffer_capacity: 0,
            index_data: Vec::new(),
            index_buffer,
            index_buffer_capacity: 0,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &archie::Context,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        meshes: Vec<egui::ClippedMesh>,
        textures_delta: egui::TexturesDelta,
    ) {
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.surface_format();
        let scale_factor = ctx.window().scale_factor() as f32;
        let window_size = ctx.window().inner_size();

        for (id, delta) in textures_delta.set {
            self.update_texture(id, device, queue, format, delta);
        }

        let mut index_offset = 0;

        let mut draw_commands = Vec::new();

        self.vertex_data.clear();
        self.index_data.clear();

        for egui::ClippedMesh(rect, mesh) in meshes {
            debug_assert!(mesh.is_valid());

            let (x, y, w, h) = (
                (rect.min.x * scale_factor).round() as u32,
                (rect.min.y * scale_factor).round() as u32,
                (rect.width() * scale_factor).round() as u32,
                (rect.height() * scale_factor).round() as u32,
            );

            if w < 1 || h < 1 || x >= window_size.width || y >= window_size.height {
                continue;
            }

            for vertex in &mesh.vertices {
                self.vertex_data
                    .extend_from_slice(bytemuck::bytes_of(&[vertex.pos.x, vertex.pos.y]));
                self.vertex_data
                    .extend_from_slice(bytemuck::bytes_of(&[vertex.uv.x, vertex.uv.y]));
                self.vertex_data.extend_from_slice(bytemuck::bytes_of(
                    &vertex.color.to_array().map(|c| c as f32),
                ));
            }

            let indices_with_offset = mesh
                .indices
                .iter()
                .map(|i| i + index_offset)
                .collect::<Vec<_>>();
            self.index_data
                .extend_from_slice(bytemuck::cast_slice(indices_with_offset.as_slice()));
            index_offset += mesh.vertices.len() as u32;

            let x_viewport_clamp = (x + w).saturating_sub(window_size.width);
            let y_viewport_clamp = (y + h).saturating_sub(window_size.height);

            draw_commands.push(DrawCommand {
                vertices: mesh.indices.len(),
                texture_id: mesh.texture_id,
                clip: (
                    x,
                    y,
                    w.saturating_sub(x_viewport_clamp).max(1),
                    h.saturating_sub(y_viewport_clamp).max(1),
                ),
            });
        }

        self.resize_buffers(device);

        self.render(
            device,
            queue,
            encoder,
            view,
            Transform::new(
                (window_size.width as f32, window_size.height as f32),
                scale_factor,
            ),
            draw_commands,
        );

        for &id in &textures_delta.free {
            self.free_texture(id);
        }
    }

    fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        transform: Transform,
        draw_commands: Vec<DrawCommand>,
    ) {
        if draw_commands.is_empty() {
            return;
        }

        let transform = {
            let contents = bytemuck::bytes_of(&transform);
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents,
                usage: BufferUsages::UNIFORM | BufferUsages::VERTEX,
            })
        };

        queue.write_buffer(&self.vertex_buffer, 0, &self.vertex_data);
        queue.write_buffer(&self.index_buffer, 0, &self.index_data);

        let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("egui transform bind group"),
            layout: &self.pipeline.transform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transform.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let mut texure_bind_groups = HashMap::new();

        for (id, texture) in &self.textures {
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.pipeline.texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &texture.create_view(&TextureViewDescriptor::default()),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            });

            texure_bind_groups.insert(*id, bind_group);
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            pass.set_bind_group(0, &transform_bind_group, &[]);

            let mut vertex_offset: u32 = 0;
            for draw_command in draw_commands {
                if let Some(texture_bind_group) = texure_bind_groups.get(&draw_command.texture_id) {
                    pass.set_bind_group(1, texture_bind_group, &[]);
                    pass.set_scissor_rect(
                        draw_command.clip.0,
                        draw_command.clip.1,
                        draw_command.clip.2,
                        draw_command.clip.3,
                    );
                    pass.draw_indexed(
                        vertex_offset..(vertex_offset + draw_command.vertices as u32),
                        0,
                        0..1,
                    );
                };

                vertex_offset += draw_command.vertices as u32;
            }
        }
    }

    fn update_texture(
        &mut self,
        id: egui::TextureId,
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        delta: egui::epaint::ImageDelta,
    ) {
        let image = match delta.image {
            egui::ImageData::Alpha(alpha) => {
                let pixels = alpha.srgba_pixels(1.0).collect::<Vec<_>>();
                egui::ColorImage {
                    size: alpha.size,
                    pixels,
                }
            }
            egui::ImageData::Color(color) => color,
        };

        let [w, h] = image.size;
        let data = bytemuck::cast_slice(image.pixels.as_slice());

        let size = Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        };

        let bytes_per_row = NonZeroU32::new(4 * size.width);
        let rows_per_image = NonZeroU32::new(size.height);

        if let Some([x, y]) = delta.pos {
            // partially update an existing texture
            if let Some(texture) = self.textures.get(&id) {
                queue.write_texture(
                    ImageCopyTextureBase {
                        texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: x as u32,
                            y: y as u32,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row,
                        rows_per_image,
                    },
                    size,
                );
            }
        } else {
            // create a new texture
            let texture = device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            });

            queue.write_texture(
                texture.as_image_copy(),
                data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row,
                    rows_per_image,
                },
                size,
            );

            if let Some(old) = self.textures.insert(id, texture) {
                old.destroy();
            }
        }
    }

    fn resize_buffers(&mut self, device: &Device) {
        if self.vertex_data.len() > self.vertex_buffer_capacity {
            self.vertex_buffer_capacity = if self.vertex_data.len().is_power_of_two() {
                self.vertex_data.len()
            } else {
                self.vertex_data.len().next_power_of_two()
            };
            self.vertex_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("egui vertex buffer"),
                size: self.vertex_buffer_capacity as BufferAddress,
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        }
        if self.index_data.len() > self.index_buffer_capacity {
            self.index_buffer_capacity = if self.index_data.len().is_power_of_two() {
                self.index_data.len()
            } else {
                self.index_data.len().next_power_of_two()
            };
            self.index_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("egui index buffer"),
                size: self.index_buffer_capacity as BufferAddress,
                usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
                mapped_at_creation: false,
            });
        }
    }

    fn free_texture(&mut self, id: egui::TextureId) {
        if let Some(t) = self.textures.remove(&id) {
            t.destroy();
        }
    }
}

struct Pipeline {
    pipeline: RenderPipeline,

    transform_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
}

impl Pipeline {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader_desc = include_wgsl!("egui.wgsl");
        let shader_module = device.create_shader_module(&shader_desc);

        let transform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui transform bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("egui pipeline layout"),
            bind_group_layouts: &[&transform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("egui render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 32,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Cw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            transform_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}

impl std::ops::Deref for Pipeline {
    type Target = RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
