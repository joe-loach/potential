mod renderer;

use archie::wgpu;
use renderer::Renderer;

pub struct Egui {
    context: egui::Context,
    renderer: Renderer,
    winit_state: egui_winit::State,
    output: Option<egui::FullOutput>,
}

impl Egui {
    pub fn new(ctx: &archie::Context) -> Self {
        let device = ctx.device();

        let egui_renderer = Renderer::new(device, ctx.surface_format());
        let egui_winit_state = egui_winit::State::new(
            device.limits().max_texture_dimension_2d as usize,
            ctx.window(),
        );
        let egui_context = egui::Context::default();

        Egui {
            context: egui_context,
            winit_state: egui_winit_state,
            renderer: egui_renderer,
            output: None,
        }
    }

    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    pub fn update(&mut self, ctx: &archie::Context, mut ui: impl FnMut(&egui::Context)) {
        let mut output = {
            let input = self.winit_state.take_egui_input(ctx.window());
            self.context.begin_frame(input);
            ui(&self.context);
            self.context.end_frame()
        };

        self.winit_state.handle_platform_output(
            ctx.window(),
            &self.context,
            output.platform_output.take(),
        );

        self.output = Some(output);
    }

    pub fn draw(
        &mut self,
        ctx: &mut archie::Context,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if let Some(output) = self.output.take() {
            let meshes = self.context.tessellate(output.shapes);
            self.renderer
                .draw(ctx, encoder, view, meshes, &output.textures_delta);
        }
    }

    pub fn handle_event(&mut self, event: &archie::winit::event::Event<()>) {
        if let archie::winit::event::Event::WindowEvent { event, .. } = event {
            self.winit_state.on_event(&self.context, event);
        }
    }
}
