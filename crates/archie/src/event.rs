use winit::{
    event::{
        self, ElementState, Event, KeyboardInput, ModifiersState, MouseButton, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
};

use crate::Context;

#[allow(unused_variables)]
pub trait EventHandler<E = ()> {
    fn update(&mut self, ctx: &Context, dt: f32);
    fn draw(
        &mut self,
        ctx: &Context,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );

    fn ui(&mut self, ctx: &egui::CtxRef) {}

    fn key_up(&mut self, key: VirtualKeyCode, modifiers: &ModifiersState) {}
    fn key_down(&mut self, key: VirtualKeyCode, modifiers: &ModifiersState) {}

    fn mouse_up(&mut self, key: MouseButton) {}
    fn mouse_down(&mut self, key: MouseButton) {}

    fn mouse_moved(&mut self, x: f64, y: f64) {}
    fn wheel_moved(&mut self, dx: f32, dy: f32) {}

    fn raw_event(&mut self, event: &Event<E>) {}
}

pub fn run<S, E>(mut ctx: Context, event_loop: EventLoop<E>, mut state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    ctx.window.set_visible(true);
    let mut last = instant::Instant::now();
    let mut modifiers = ModifiersState::empty();
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

                // try our best to make sure we have a texture to draw to
                let frame = ctx.surface.get_current_texture().or_else(|e| {
                    if let wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost = e {
                        ctx.surface.configure(&ctx.device, &ctx.surface_config);
                        ctx.surface.get_current_texture()
                    } else {
                        Err(e)
                    }
                });
                // if we fail, stop the program
                let frame = match frame {
                    Ok(frame) => frame,
                    Err(err) => {
                        log::error!("get_current_texture failed: {:?}", err);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                };

                let mut encoder = ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let current = instant::Instant::now();
                let elapsed = current - last;
                state.update(&ctx, elapsed.as_secs_f32());
                state.draw(&ctx, &mut encoder, &view);

                egui_render(&mut ctx, &mut encoder, &view, |ctx| state.ui(ctx));

                // Submit the commands.
                ctx.queue.submit(std::iter::once(encoder.finish()));

                frame.present();
                last = current;
            }

            event::Event::MainEventsCleared => {
                ctx.window.request_redraw();
            }

            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        return;
                    }
                    ctx.surface_config.width = size.width;
                    ctx.surface_config.height = size.height;
                    ctx.surface.configure(&ctx.device, &ctx.surface_config);
                }
                event::WindowEvent::CursorMoved { position, .. } => {
                    state.mouse_moved(position.x, position.y);
                }
                event::WindowEvent::MouseWheel {
                    delta: event::MouseScrollDelta::LineDelta(dx, dy),
                    ..
                } => {
                    state.wheel_moved(dx, dy);
                }
                event::WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => match mouse_state {
                    ElementState::Pressed => state.mouse_down(button),
                    ElementState::Released => state.mouse_up(button),
                },
                event::WindowEvent::ModifiersChanged(input) => {
                    modifiers = input;
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
                    ElementState::Pressed => state.key_down(key, &modifiers),
                    ElementState::Released => state.key_up(key, &modifiers),
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

    pass.execute(encoder, target, &paint_jobs, &screen_descriptor, None)
        .unwrap();
}
