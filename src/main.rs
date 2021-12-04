#![deny(clippy::all)]

use std::rc::Rc;

use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

/// A custom event type for the winit app.
// enum Event {
//     RequestRedraw,
// }

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal;

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        // self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
    }
}

fn demo(ctx: &egui::CtxRef, frame: &mut epi::Frame) {
    egui::Window::new("demo").show(&ctx, |ui| {
        if ui.button("Click").clicked() {

        }
    });
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello world")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        use wasm_bindgen::JsCast;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        
        // Retrieve current width and height dimensions of browser client window
        let get_window_size = || {
            let client_window = web_sys::window().unwrap();
            LogicalSize::new(
                client_window.inner_width().unwrap().as_f64().unwrap(),
                client_window.inner_height().unwrap().as_f64().unwrap(),
            )
        };

        let window = Rc::clone(&window);

        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let size = get_window_size();
            window.set_inner_size(size);
        }) as Box<dyn FnMut(_)>);
        web_sys::window().unwrap()
             .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
             .unwrap();
        closure.forget();
    }

    let backends = {
        if cfg!(target_arch = "wasm32") {
            wgpu::Backends::all()
        } else {
            wgpu::Backends::PRIMARY
        }
    };
    let instance = wgpu::Instance::new(backends);
    let surface = unsafe { instance.create_surface(window.as_ref()) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (mut device, mut queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                limits: adapter.limits(),
                features: wgpu::Features::empty(),
                ..Default::default()
            },
            None,
        )
        .await
        .expect("couldn't find a device");

    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal);

    let mut platform =
        egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        });

    let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

    let start_time = instant::Instant::now();
    let mut previous_frame_time = None;
    event_loop.run(move |event, _, control_flow| {
        platform.handle_event(&event);
        
        match event {
            winit::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,

            winit::event::Event::RedrawRequested(_) => {
                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
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
                platform.begin_frame();
                let mut app_output = epi::backend::AppOutput::default();
                let mut frame = epi::backend::FrameBuilder {
                    info: epi::IntegrationInfo {
                        name: "",
                        web_info: None,
                        cpu_usage: previous_frame_time,
                        native_pixels_per_point: Some(window.scale_factor() as _),
                        prefer_dark_mode: None,
                    },
                    tex_allocator: &mut egui_rpass,
                    output: &mut app_output,
                    repaint_signal: repaint_signal.clone(),
                }
                .build();

                // Draw the demo application.
                demo(&platform.context(), &mut frame);

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let (_output, paint_commands) = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(paint_commands);

                let frame_time = egui_start.elapsed().as_secs_f64() as f32;
                previous_frame_time = Some(frame_time);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };
                egui_rpass.update_texture(&device, &queue, &platform.context().texture());
                egui_rpass.update_user_textures(&device, &queue);
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();
                // Submit the commands.
                queue.submit(std::iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();
            }

            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }

            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);
                }
                _ => (),
            },
            _ => (),
        }
    });
}
