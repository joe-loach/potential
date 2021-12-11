use std::sync::Arc;

use winit::{
    event::{self, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::Context;

#[allow(unused_variables)]
pub trait EventHandler<E = ()> {
    fn update(&mut self);
    fn draw(&mut self);

    fn ui(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {}

    fn key_up(&mut self, key: VirtualKeyCode) {}
    fn key_down(&mut self, key: VirtualKeyCode) {}

    fn raw_event(&mut self, event: &Event<E>) {}
}

pub fn run<S, E>(mut ctx: Context, event_loop: EventLoop<E>, mut state: S) -> !
where
    S: EventHandler<E> + 'static,
{
    let start_time = instant::Instant::now();
    let mut previous_frame_time = None;
    event_loop.run(move |event, _, control_flow| {
        state.raw_event(&event);
        ctx.egui_platform.handle_event(&event);

        match event {
            event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == ctx.window.id() => *control_flow = ControlFlow::Exit,

            event::Event::RedrawRequested(_) => {
                ctx.egui_platform
                    .update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match ctx.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                // Begin to draw the UI frame.
                let egui_start = instant::Instant::now();
                ctx.egui_platform.begin_frame();
                let mut app_output = epi::backend::AppOutput::default();
                let mut frame = epi::backend::FrameBuilder {
                    info: epi::IntegrationInfo {
                        name: "potential",
                        web_info: web_info(),
                        cpu_usage: previous_frame_time,
                        native_pixels_per_point: Some(ctx.window.scale_factor() as _),
                        prefer_dark_mode: None,
                    },
                    tex_allocator: &mut ctx.egui_render_pass,
                    output: &mut app_output,
                    repaint_signal: Arc::new({
                        struct DummyRepaintSignal;
                        impl epi::RepaintSignal for DummyRepaintSignal {
                            fn request_repaint(&self) {}
                        }
                        DummyRepaintSignal
                    }),
                }
                .build();

                // Draw the demo application.
                state.ui(&ctx.egui_platform.context(), &mut frame);

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let (_output, paint_commands) = ctx.egui_platform.end_frame(Some(&ctx.window));
                let paint_jobs = ctx.egui_platform.context().tessellate(paint_commands);

                let frame_time = egui_start.elapsed().as_secs_f64() as f32;
                previous_frame_time = Some(frame_time);

                let mut encoder =
                    ctx.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("encoder"),
                        });

                // Upload all resources for the GPU.
                let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                    physical_width: ctx.surface_config.width,
                    physical_height: ctx.surface_config.height,
                    scale_factor: ctx.window.scale_factor() as f32,
                };
                ctx.egui_render_pass.update_texture(
                    &ctx.device,
                    &ctx.queue,
                    &ctx.egui_platform.context().texture(),
                );
                ctx.egui_render_pass
                    .update_user_textures(&ctx.device, &ctx.queue);
                ctx.egui_render_pass.update_buffers(
                    &ctx.device,
                    &ctx.queue,
                    &paint_jobs,
                    &screen_descriptor,
                );

                // Record all render passes.
                ctx.egui_render_pass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();
                // Submit the commands.
                ctx.queue.submit(std::iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();
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

    #[cfg(not(target_arch = "wasm32"))]
    fn web_info() -> Option<epi::WebInfo> {
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn web_info() -> Option<epi::WebInfo> {
        let web_location_hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_else(|| String::from(""));
        Some(epi::WebInfo { web_location_hash })
    }
}
