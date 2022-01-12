use anyhow::Result;
use archie::wgpu;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let device = ctx.device();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = pipeline(device, &pipeline_layout, ctx.surface_format());

        Ok(Self {
            pipeline,
            bind_group_layout,
        })
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let desc = {
        let spirv = include_bytes!("shaders/compute.spv");
        let source = wgpu::util::make_spirv(spirv);
        wgpu::ShaderModuleDescriptor {
            label: None,
            source,
        }
    };
    let module = { device.create_shader_module(&desc) };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: "vert",
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
            clamp_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: "frag",
            targets: &[wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::all(),
            }],
        }),
    })
}

// fn load_shaders() -> Result<impl Iterator<Item = Shader>> {
//     use common::*;
//     let config = std::fs::read_to_string("shaders.toml")?;
//     let config: Config = toml::from_str(&config)?;
//     Ok(config.shaders.into_iter().map(|(name, info)| {
//         let data = std::fs::read(&info.module).unwrap();
//         let name: &'static str = Box::leak(name.into_boxed_str());
//         let spirv = Cow::Owned(wgpu::util::make_spirv_raw(&data).into_owned());
//         Shader {
//             entries: info.entries,
//             desc: wgpu::ShaderModuleDescriptorSpirV {
//                 label: Some(name),
//                 source: spirv,
//             },
//         }
//     }))
// }
