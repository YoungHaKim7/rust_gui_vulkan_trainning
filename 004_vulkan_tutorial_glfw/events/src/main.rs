// https://github.com/PistonDevelopers/glfw-rs/blob/master/examples/events.rs
// Copyright 2013 The GLFW-RS Developers. For a full listing of the authors,
// refer to the AUTHORS file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
        let (mut window, events) = glfw
            .create_window(WIDTH, HEIGHT, "Vulkan", WindowMode::Windowed)
            .ok_or("failed to create GLFW window")?;

        window.set_sticky_keys(true);

        // Polling of events can be turned on and off by the specific event type
        window.set_pos_polling(true);
        window.set_size_polling(true);
        window.set_close_polling(true);
        window.set_refresh_polling(true);
        window.set_focus_polling(true);
        window.set_iconify_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);
        window.set_char_polling(true);
        window.set_char_mods_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_cursor_enter_polling(true);
        window.set_scroll_polling(true);
        window.set_maximize_polling(true);
        window.set_content_scale_polling(true);

        // Alternatively, all event types may be set to poll at once. Note that
        // in this example, this call is redundant as all events have been set
        // to poll in the above code.
        window.set_all_polling(true);

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

            for (time, event) in glfw::flush_messages(&self.events) {
                handle_window_event(&mut self.window, (time, event));
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

fn handle_window_event(window: &mut glfw::Window, (time, event): (f64, glfw::WindowEvent)) {
    match event {
        glfw::WindowEvent::Pos(x, y) => {
            window.set_title(&format!("Time: {:?}, Window pos: ({:?}, {:?})", time, x, y))
        }
        glfw::WindowEvent::Size(w, h) => window.set_title(&format!(
            "Time: {:?}, Window size: ({:?}, {:?})",
            time, w, h
        )),
        glfw::WindowEvent::Close => println!("Time: {:?}, Window close requested.", time),
        glfw::WindowEvent::Refresh => {
            println!("Time: {:?}, Window refresh callback triggered.", time)
        }
        glfw::WindowEvent::Focus(true) => println!("Time: {:?}, Window focus gained.", time),
        glfw::WindowEvent::Focus(false) => println!("Time: {:?}, Window focus lost.", time),
        glfw::WindowEvent::Iconify(true) => println!("Time: {:?}, Window was minimised", time),
        glfw::WindowEvent::Iconify(false) => println!("Time: {:?}, Window was maximised.", time),
        glfw::WindowEvent::FramebufferSize(w, h) => {
            println!("Time: {:?}, Framebuffer size: ({:?}, {:?})", time, w, h)
        }
        glfw::WindowEvent::Char(character) => {
            println!("Time: {:?}, Character: {:?}", time, character)
        }
        glfw::WindowEvent::CharModifiers(character, mods) => println!(
            "Time: {:?}, Character: {:?}, Modifiers: [{:?}]",
            time, character, mods
        ),
        glfw::WindowEvent::MouseButton(btn, action, mods) => println!(
            "Time: {:?}, Button: {:?}, Action: {:?}, Modifiers: [{:?}]",
            time,
            glfw::DebugAliases(btn),
            action,
            mods
        ),
        glfw::WindowEvent::CursorPos(xpos, ypos) => window.set_title(&format!(
            "Time: {:?}, Cursor position: ({:?}, {:?})",
            time, xpos, ypos
        )),
        glfw::WindowEvent::CursorEnter(true) => {
            println!("Time: {:?}, Cursor entered window.", time)
        }
        glfw::WindowEvent::CursorEnter(false) => println!("Time: {:?}, Cursor left window.", time),
        glfw::WindowEvent::Scroll(x, y) => window.set_title(&format!(
            "Time: {:?}, Scroll offset: ({:?}, {:?})",
            time, x, y
        )),
        glfw::WindowEvent::Key(key, scancode, action, mods) => {
            println!(
                "Time: {:?}, Key: {:?}, ScanCode: {:?}, Action: {:?}, Modifiers: [{:?}]",
                time, key, scancode, action, mods
            );
            match (key, action) {
                (Key::Escape, Action::Press) => window.set_should_close(true),
                (Key::R, Action::Press) => {
                    // Resize should cause the window to "refresh"
                    let (window_width, window_height) = window.get_size();
                    window.set_size(window_width + 1, window_height);
                    window.set_size(window_width, window_height);
                }
                _ => {}
            }
        }
        glfw::WindowEvent::FileDrop(paths) => {
            println!("Time: {:?}, Files dropped: {:?}", time, paths)
        }
        glfw::WindowEvent::Maximize(maximized) => {
            println!("Time: {:?}, Window maximized: {:?}.", time, maximized)
        }
        glfw::WindowEvent::ContentScale(xscale, yscale) => println!(
            "Time: {:?}, Content scale x: {:?}, Content scale y: {:?}",
            time, xscale, yscale
        ),
    }
}
