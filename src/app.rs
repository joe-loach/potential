use crate::emq;
use crate::mq::{self, Context, EventHandler};

pub struct Potential {
    emq: emq::EguiMq,
}

impl Potential {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            emq: emq::EguiMq::new(ctx),
        }
    }

    fn ui(&mut self) {
        let ctx = self.emq.egui_ctx();

        egui::Window::new("Window").show(ctx, |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            }
        });
    }
}

impl EventHandler for Potential {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);

        self.emq.begin_frame(ctx);
        self.ui();
        self.emq.end_frame(ctx);

        // draw here

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
