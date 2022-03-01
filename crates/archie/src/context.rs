use std::rc::Rc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::timer::Timer;

pub struct Context {
    pub(crate) window: Rc<Window>,

    pub(crate) device: Device,
    pub(crate) queue: Queue,

    pub(crate) surface: Surface,
    pub(crate) surface_config: SurfaceConfiguration,

    pub(crate) timer: Timer,
}

impl Context {
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }
}

pub struct ContextBuilder {
    title: String,
    width: u32,
    height: u32,
    fullscreen: bool,
    vsync: bool,
    decorations: bool,
}

impl ContextBuilder {
    /// Creates a new [`ContextBuilder`].
    ///
    /// This is the same as calling `ContextBuilder::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn title<T: Into<String>>(self, title: T) -> Self {
        Self {
            title: title.into(),
            ..self
        }
    }

    #[must_use]
    pub fn width(self, width: u32) -> Self {
        Self { width, ..self }
    }

    #[must_use]
    pub fn height(self, height: u32) -> Self {
        Self { height, ..self }
    }

    #[must_use]
    pub fn fullscreen(self, fullscreen: bool) -> Self {
        Self { fullscreen, ..self }
    }

    #[must_use]
    pub fn vsync(self, vsync: bool) -> Self {
        Self { vsync, ..self }
    }

    #[must_use]
    pub fn decorations(self, decorations: bool) -> Self {
        Self { decorations, ..self }
    }
}

impl ContextBuilder {
    /// Consumes the builder and produces a [`Context`].
    pub async fn build(
        self,
        features: Option<wgpu::Features>,
    ) -> Result<(EventLoop<()>, Context), BuildError> {
        let Self {
            title,
            width,
            height,
            fullscreen,
            vsync,
            decorations,
            ..
        } = self;

        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(width, height);
            let builder = WindowBuilder::new()
                .with_visible(false)
                .with_title(title)
                .with_inner_size(size)
                .with_decorations(decorations)
                .with_fullscreen(if fullscreen {
                    Some(Fullscreen::Borderless(None))
                } else {
                    None
                });

            builder.build(&event_loop).map_err(BuildError::Window)?
        };

        let window = Rc::new(window);

        // On WASM, a canvas should be created for the Window.
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{closure::Closure, JsCast};
            use winit::platform::web::WindowExtWebSys;

            let window = Rc::clone(&window);
            let web_window = web_sys::window().ok_or(BuildError::WebWindow)?;

            web_window
                .document()
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .ok_or(BuildError::Canvas)?;

            // keep the canvas the size of the inner window
            if fullscreen {
                let get_window_size = || {
                    let client_window = web_sys::window().unwrap();
                    LogicalSize::new(
                        client_window
                            .inner_width()
                            .unwrap()
                            .as_f64()
                            .unwrap()
                            .ceil() as u32,
                        client_window
                            .inner_height()
                            .unwrap()
                            .as_f64()
                            .unwrap()
                            .ceil() as u32,
                    )
                };

                // Initialize winit window with current dimensions of browser client
                window.set_inner_size(get_window_size());

                // Listen for resize event on browser client. Adjust winit window dimensions
                // on event trigger
                let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let size = get_window_size();
                    window.set_inner_size(size);
                }) as Box<dyn FnMut(_)>);

                web_window
                    .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                    .unwrap();

                closure.forget();
            }
        }

        // choose appropriate backends based on the platform
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
            .ok_or(BuildError::AdapterNotFound)?;

        let required = wgpu::Features::empty();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    limits: adapter.limits(),
                    features: required | features.unwrap_or_else(wgpu::Features::empty),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(BuildError::DeviceNotFound)?;

        // we need to ask for the inner size again,
        // this is the *physical* size now, not *logical*
        let PhysicalSize { width, height } = window.inner_size();

        let format = surface
            .get_preferred_format(&adapter)
            .unwrap_or(TextureFormat::Rgba8UnormSrgb);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: if vsync {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Immediate
            },
        };
        surface.configure(&device, &surface_config);

        let timer = Timer::new();

        let ctx = Context {
            window,
            device,
            queue,
            surface,
            surface_config,
            timer,
        };

        Ok((event_loop, ctx))
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            title: String::from("Archie Window"),
            width: 600,
            height: 600,
            fullscreen: false,
            vsync: true,
            decorations: true,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("window couldn't be created")]
    Window(#[from] winit::error::OsError),
    #[error("failed to get web_sys window")]
    WebWindow,
    #[error("canvas couldn't be created")]
    Canvas,
    #[error("no adapter found")]
    AdapterNotFound,
    #[error("no device found")]
    DeviceNotFound(wgpu::RequestDeviceError),
}
