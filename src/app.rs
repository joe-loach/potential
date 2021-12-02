mod gui;
mod shader;

use crate::emq;
use crate::mq;

use glam::*;
use mq::Bindings;
use mq::Buffer;
use mq::BufferLayout;
use mq::BufferType;

pub struct App {
    // gui
    emq: emq::EguiMq,
    // shader
    pipeline: Pipeline,
    bindings: Bindings,
}

impl App {
    pub fn new(ctx: &mut mq::Context) -> Self {
        #[rustfmt::skip]
        let screen = [
            Vec2::new(-1.0, -1.0),
            Vec2::new( 1.0, -1.0),
            Vec2::new( 1.0,  1.0),
            Vec2::new(-1.0,  1.0),
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &screen);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &[0_u16, 1, 2, 0, 2, 3]);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: Vec::new(),
        };

        let shader = shader::shader(ctx).expect("couldn't compile shader");
        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("pos", VertexFormat::Float2)],
            shader,
        );
        
        Self {
            emq: emq::EguiMq::new(ctx),
            pipeline,
            bindings,
        }
    }
}

use mq::Pipeline;
use mq::VertexAttribute;
use mq::VertexFormat;
use mq::{Context, EventHandler};

impl EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear((1.0, 1.0, 1.0, 1.0).into(), None, None);

        self.emq.begin_frame(ctx);
        self.ui();
        self.emq.end_frame(ctx);

        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        let screen_size = ctx.screen_size();
        let ratio = screen_size.1 / screen_size.0;
        let (scale_x, scale_y) = if ratio <= 1.0 {
            (ratio, 1.0)
        } else {
            (1.0, 1.0 / ratio)
        };

        #[rustfmt::skip]
        let transform = Mat4::from_cols_array(&[
            scale_x, 0.0, 0.0, 0.0,
            0.0, scale_y, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        ctx.apply_uniforms(&transform);
        ctx.draw(0, 6, 1);

        ctx.end_render_pass();

        self.emq.draw(ctx);

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.emq.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.emq.mouse_wheel_event(ctx, dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.emq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.emq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.emq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.emq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.emq.key_up_event(keycode, keymods);
    }
}
