use winit::{
    dpi::PhysicalSize,
    event::{
        ElementState, Event, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
};

use crate::Context;

#[allow(unused_variables)]
pub trait EventHandler<E = ()> {
    fn update(&mut self, ctx: &Context);
    fn draw(
        &mut self,
        ctx: &mut Context,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );

    fn key_up(&mut self, key: VirtualKeyCode, modifiers: &ModifiersState) {}
    fn key_down(&mut self, key: VirtualKeyCode, modifiers: &ModifiersState) {}

    fn mouse_up(&mut self, key: MouseButton) {}
    fn mouse_down(&mut self, key: MouseButton) {}

    fn mouse_moved(&mut self, x: f64, y: f64) {}
    fn wheel_moved(&mut self, dx: f32, dy: f32) {}

    fn raw_event(&mut self, ctx: &mut Context, event: &Event<E>) {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run<S, E>(ctx: Context, event_loop: EventLoop<E>, state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    start(ctx, event_loop, state)
}

#[cfg(target_arch = "wasm32")]
pub fn run<S, E>(ctx: Context, event_loop: EventLoop<E>, state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    use wasm_bindgen::{prelude::*, JsCast};

    let start_closure = Closure::once_into_js(move || start(ctx, event_loop, state));

    if let Err(error) = call_catch(&start_closure) {
        let is_control_flow_exception = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
            e.message().includes("Using exceptions for control flow", 0)
        });

        if !is_control_flow_exception {
            web_sys::console::error_1(&error);
        }
    }

    std::process::exit(0);

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
        fn call_catch(this: &JsValue) -> Result<(), JsValue>;
    }
}

fn start<S, E>(mut ctx: Context, event_loop: EventLoop<E>, mut state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    ctx.window.set_visible(true);
    ctx.timer.start();

    let mut modifiers = ModifiersState::empty();

    fn reconfigure_surface(ctx: &mut Context, width: u32, height: u32) {
        ctx.surface_config.width = width;
        ctx.surface_config.height = height;
        ctx.surface.configure(&ctx.device, &ctx.surface_config);
    }

    let mut scale_factor = ctx.window.scale_factor();

    event_loop.run(move |event, _, control_flow| {
        state.raw_event(&mut ctx, &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == ctx.window.id() => *control_flow = ControlFlow::Exit,

            Event::RedrawRequested(_) => {
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

                // update the timer
                ctx.timer.tick();

                // update and fill the encoder
                state.update(&ctx);
                state.draw(&mut ctx, &mut encoder, &view);

                // Submit the commands.
                ctx.queue.submit(std::iter::once(encoder.finish()));

                frame.present();
            }

            Event::MainEventsCleared => {
                ctx.window.request_redraw();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ScaleFactorChanged {
                    scale_factor: sf,
                    new_inner_size: PhysicalSize { width, height },
                } => {
                    // only change scale factor if it's valid
                    // if not, it might cause panics elsewhere
                    if winit::dpi::validate_scale_factor(sf) {
                        scale_factor = sf;
                    }
                    reconfigure_surface(&mut ctx, *width, *height);
                }
                // note: on windows, width and height are set to 0 when minimised.
                // the surface cannot be resized to 0, do nothing.
                WindowEvent::Resized(PhysicalSize {
                    width: 0,
                    height: 0,
                }) => {}
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    reconfigure_surface(&mut ctx, width, height);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = position.to_logical(scale_factor);
                    state.mouse_moved(position.x, position.y);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (dx, dy) = match delta {
                        MouseScrollDelta::LineDelta(dx, dy) => (dx, dy),
                        MouseScrollDelta::PixelDelta(delta) => {
                            (delta.x as f32 / 32.0, delta.y as f32 / 32.0)
                        }
                    };
                    state.wheel_moved(dx, dy);
                }
                WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => match mouse_state {
                    ElementState::Pressed => state.mouse_down(button),
                    ElementState::Released => state.mouse_up(button),
                },
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::ModifiersChanged(input) => {
                    modifiers = input;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: key_state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    #[cfg(target_arch = "wasm32")]
                    match key {
                        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => {
                            modifiers.toggle(ModifiersState::ALT)
                        }
                        VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
                            modifiers.toggle(ModifiersState::CTRL)
                        }
                        VirtualKeyCode::LWin => modifiers.toggle(ModifiersState::LOGO),
                        _ => (),
                    }
                    match key_state {
                        ElementState::Pressed => state.key_down(key, &modifiers),
                        ElementState::Released => state.key_up(key, &modifiers),
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });
}
