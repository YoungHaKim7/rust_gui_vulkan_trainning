use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Window>,
}

impl App {
    fn new() -> Self {
        Self { window: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Shader playground"))
            .unwrap();

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                // wgpu rendering will go here.
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
