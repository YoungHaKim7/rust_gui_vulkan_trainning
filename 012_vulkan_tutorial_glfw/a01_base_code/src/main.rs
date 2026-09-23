//! Rust port of the Vulkan tutorial base code:
//! https://github.com/Overv/VulkanTutorial/blob/main/code/00_base_code.cpp
//! https://crates.io/crates/glfw(rust)

use std::error::Error;

use glfw::{
    Action, ClientApiHint, Glfw, GlfwReceiver, Key, PWindow, WindowEvent, WindowHint, WindowMode,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct HelloTriangleApplication {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

impl HelloTriangleApplication {
    fn new() -> Result<Self, Box<dyn Error>> {
        // glfwInit()
        let mut glfw = glfw::init(glfw::fail_on_errors)?;

        // initWindow()
        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Resizable(false));
        let (window, events) = glfw
            .create_window(WIDTH, HEIGHT, "Vulkan", WindowMode::Windowed)
            .ok_or("failed to create GLFW window")?;

        Ok(Self {
            glfw,
            window,
            events,
        })
    }

    fn run(&mut self) {
        self.init_vulkan();
        self.main_loop();
        self.cleanup();
    }

    fn init_vulkan(&self) {
        // Will be filled in the next chapters.
    }

    fn main_loop(&mut self) {
        while !self.window.should_close() {
            self.glfw.poll_events();

            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    // Not in the C++ original, but convenient during development.
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        self.window.set_should_close(true)
                    }
                    _ => {}
                }
            }
        }
    }

    fn cleanup(&mut self) {
        // glfwDestroyWindow() and glfwTerminate() are handled automatically:
        // the window is destroyed when `PWindow` drops, and `Glfw` calls
        // glfwTerminate() when it drops at the end of `main`.
    }
}

fn main() {
    let mut app = match HelloTriangleApplication::new() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1); // EXIT_FAILURE
        }
    };

    app.run();
}
