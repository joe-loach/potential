use std::borrow::Cow;

use anyhow::Result;
use archie::wgpu;

struct Shader {
    entries: Vec<String>,
    desc: wgpu::ShaderModuleDescriptorSpirV<'static>,
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let mut shaders = load_shaders()?;

        let device = ctx.device();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = pipeline(
            device,
            &pipeline_layout,
            ctx.surface_format(),
            shaders.next().unwrap(),
        );

        Ok(Self { pipeline })
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }
            }],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.pipeline);
        // pass.set_push_constants(stages, offset, data)
        pass.draw(0..3, 0..1);
    }
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    shader: Shader,
) -> wgpu::RenderPipeline {
    let module = unsafe { device.create_shader_module_spirv(&shader.desc) };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: &shader.entries[1],
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            clamp_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: &shader.entries[0],
            targets: &[wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::all(),
            }],
        }),
    })
}

fn load_shaders() -> Result<impl Iterator<Item = Shader>> {
    use common::*;
    let config = std::fs::read_to_string("shaders.toml")?;
    let config: Config = toml::from_str(&config)?;
    Ok(config.shaders.into_iter().map(|(name, info)| {
        let data = std::fs::read(&info.module).unwrap();
        let name: &'static str = Box::leak(name.clone().into_boxed_str());
        let spirv = Cow::Owned(wgpu::util::make_spirv_raw(&data).into_owned());
        Shader {
            entries: info.entries,
            desc: wgpu::ShaderModuleDescriptorSpirV {
                label: Some(name),
                source: spirv,
            },
        }
    }))
}
