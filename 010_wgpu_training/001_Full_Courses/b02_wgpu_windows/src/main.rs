use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct State {
    // Surface is declared before Window so that Surface is dropped
    // before the Window.
    surface: wgpu::Surface<'static>,

    device: wgpu::Device,
    queue: wgpu::Queue,

    config: wgpu::SurfaceConfiguration,

    size: winit::dpi::PhysicalSize<u32>,

    window: Arc<Window>,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // ------------------------------------------------------------
        // 1. Create wgpu instance
        // ------------------------------------------------------------

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            display: None,
        });

        // ------------------------------------------------------------
        // 2. Create rendering surface
        // ------------------------------------------------------------

        let surface = instance.create_surface(window.clone()).unwrap();

        // ------------------------------------------------------------
        // 3. Find GPU adapter
        // ------------------------------------------------------------

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: true,
            })
            .await
            .unwrap();

        println!("Using adapter: {:?}", adapter.get_info());

        // ------------------------------------------------------------
        // 4. Create device and queue
        // ------------------------------------------------------------

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Main Device"),

                required_features: wgpu::Features::empty(),

                required_limits: wgpu::Limits::default(),

                memory_hints: wgpu::MemoryHints::default(),

                experimental_features: wgpu::ExperimentalFeatures::disabled(),

                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        // ------------------------------------------------------------
        // 5. Get surface capabilities
        // ------------------------------------------------------------

        let surface_caps = surface.get_capabilities(&adapter);

        // Prefer an sRGB format.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // ------------------------------------------------------------
        // 6. Configure surface
        // ------------------------------------------------------------

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

            format: surface_format,

            color_space: wgpu::SurfaceColorSpace::Auto,

            width: size.width,
            height: size.height,

            present_mode: wgpu::PresentMode::Fifo,

            desired_maximum_frame_latency: 2,

            alpha_mode: surface_caps.alpha_modes[0],

            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    // ------------------------------------------------------------
    // Resize
    // ------------------------------------------------------------

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
    }

    // ------------------------------------------------------------
    // Render
    // ------------------------------------------------------------

    fn render(&mut self) -> Result<(), wgpu::CurrentSurfaceTexture> {
        // --------------------------------------------------------
        // Get the next surface texture.
        //
        // Since wgpu 30, get_current_texture() returns an enum
        // instead of a Result<SurfaceTexture, SurfaceError>:
        //
        // CurrentSurfaceTexture
        //      ├── Success(SurfaceTexture)
        //      │        │
        //      │        └── texture
        //      │                 │
        //      │                 └── create_view()
        //      ├── Suboptimal(SurfaceTexture) // usable, but reconfigure
        //      └── Timeout | Occluded | Outdated | Lost | Validation
        // --------------------------------------------------------

        let mut reconfigure = false;

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => output,

            // The texture is usable, but no longer matches the surface,
            // so reconfigure after presenting this frame.
            wgpu::CurrentSurfaceTexture::Suboptimal(output) => {
                reconfigure = true;
                output
            }

            error => return Err(error),
        };

        // --------------------------------------------------------
        // Create a texture view.
        // --------------------------------------------------------

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // --------------------------------------------------------
        // Create command encoder.
        // --------------------------------------------------------

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // --------------------------------------------------------
        // Render pass.
        // --------------------------------------------------------

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,

                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],

                depth_stencil_attachment: None,

                timestamp_writes: None,

                occlusion_query_set: None,

                multiview_mask: None,
            });
        }

        // --------------------------------------------------------
        // Submit commands to GPU.
        // --------------------------------------------------------

        self.queue.submit(Some(encoder.finish()));

        // --------------------------------------------------------
        // Present the rendered image.
        // Since wgpu 30, present() lives on Queue, not on the
        // surface texture: queue.present(surface_texture).
        // --------------------------------------------------------

        self.queue.present(output);

        if reconfigure {
            self.resize(self.size);
        }

        Ok(())
    }
}

// ====================================================================
// Application
// ====================================================================

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    // ----------------------------------------------------------------
    // Window becomes active.
    // ----------------------------------------------------------------

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Shader playground"))
            .unwrap();

        let window = Arc::new(window);

        // State::new() is async, but ApplicationHandler::resumed()
        // is not async, so block here.
        let state = pollster::block_on(State::new(window));

        self.state = Some(state);

        // Request the first frame.
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    // ----------------------------------------------------------------
    // Window events.
    // ----------------------------------------------------------------

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        // Ignore events belonging to another window.
        if window_id != state.window.id() {
            return;
        }

        match event {
            // --------------------------------------------------------
            // Close window.
            // --------------------------------------------------------
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            // --------------------------------------------------------
            // Window resized.
            // --------------------------------------------------------
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            }

            // --------------------------------------------------------
            // Render frame.
            // --------------------------------------------------------
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(()) => {}

                    Err(wgpu::CurrentSurfaceTexture::Lost) => {
                        state.resize(state.size);
                    }

                    Err(wgpu::CurrentSurfaceTexture::Outdated) => {
                        state.resize(state.size);
                    }

                    Err(wgpu::CurrentSurfaceTexture::Timeout) => {
                        eprintln!("Surface timeout");
                    }

                    Err(wgpu::CurrentSurfaceTexture::Occluded) => {
                        eprintln!("Surface occluded");
                    }

                    Err(wgpu::CurrentSurfaceTexture::Validation) => {
                        eprintln!("Surface validation error");
                    }

                    // render() only ever returns these two on success.
                    Err(wgpu::CurrentSurfaceTexture::Success(_))
                    | Err(wgpu::CurrentSurfaceTexture::Suboptimal(_)) => {}
                }

                // Request another frame.
                state.window.request_redraw();
            }

            _ => {}
        }
    }
}

// ====================================================================
// Main
// ====================================================================

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
