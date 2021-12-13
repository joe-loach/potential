use winit::{
    event::{self, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::Context;

#[allow(unused_variables)]
pub trait EventHandler<E = ()> {
    fn update(&mut self);
    fn draw(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);

    fn ui(&mut self, ctx: &egui::CtxRef) {}

    fn key_up(&mut self, key: VirtualKeyCode) {}
    fn key_down(&mut self, key: VirtualKeyCode) {}

    fn raw_event(&mut self, event: &Event<E>) {}
}

pub fn run<S, E>(mut ctx: Context, event_loop: EventLoop<E>, mut state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    let start = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        state.raw_event(&event);
        ctx.egui_platform.handle_event(&event);

        match event {
            event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == ctx.window.id() => *control_flow = ControlFlow::Exit,

            event::Event::RedrawRequested(_) => {
                ctx.egui_platform.update_time(start.elapsed().as_secs_f64());

                let frame = ctx
                    .surface
                    .get_current_texture()
                    .or_else(|e| {
                        if let wgpu::SurfaceError::Outdated = e {
                            ctx.surface.configure(&ctx.device, &ctx.surface_config);
                            ctx.surface.get_current_texture()
                        } else {
                            Err(e)
                        }
                    })
                    .unwrap();

                let mut encoder = ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                egui_render(&mut ctx, &mut encoder, &view, |ctx| state.ui(ctx));
                state.draw(&mut encoder, &view);

                // Submit the commands.
                ctx.queue.submit(std::iter::once(encoder.finish()));

                frame.present();
            }

            event::Event::MainEventsCleared => {
                ctx.window.request_redraw();
            }

            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::Resized(size) => {
                    ctx.surface_config.width = size.width;
                    ctx.surface_config.height = size.height;
                    ctx.surface.configure(&ctx.device, &ctx.surface_config);
                }
                event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: key_state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => match key_state {
                    ElementState::Pressed => state.key_down(key),
                    ElementState::Released => state.key_up(key),
                },
                _ => (),
            },
            _ => (),
        }
    });
}

/// Computes and renders **egui** to the [`wgpu::TextureView`].
fn egui_render<F>(
    ctx: &mut Context,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    ui: F,
) where
    F: FnOnce(&egui::CtxRef),
{
    let platform = &mut ctx.egui_platform;
    let pass = &mut ctx.egui_render_pass;

    let (output, paint_commands) = {
        platform.begin_frame();
        ui(&platform.context()); // create the UI
        platform.end_frame(Some(&ctx.window))
    };

    // handle the egui's frame output
    // TODO: handle more *stuff*
    if let Some(url) = output.open_url {
        crate::helper::open_url(&url.url, url.new_tab);
    }

    let paint_jobs = platform.context().tessellate(paint_commands);

    let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
        physical_width: ctx.surface_config.width,
        physical_height: ctx.surface_config.height,
        scale_factor: ctx.window.scale_factor() as f32,
    };
    pass.update_texture(&ctx.device, &ctx.queue, &platform.context().texture());
    pass.update_user_textures(&ctx.device, &ctx.queue);
    pass.update_buffers(&ctx.device, &ctx.queue, &paint_jobs, &screen_descriptor);

    pass.execute(
        encoder,
        &target,
        &paint_jobs,
        &screen_descriptor,
        Some(wgpu::Color::BLACK),
    )
    .unwrap();
}
