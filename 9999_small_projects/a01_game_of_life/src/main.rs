// A multi windowed game of life application. You could use this to learn:
//
// - how to handle multiple window inputs,
// - how to draw on a canvas,
// - how to organize compute shader with graphics,
// - how to do a cellular automata simulation using compute shaders.
//
// The possibilities are limitless. ;)

mod app;
mod game_of_life;
mod pixels_draw;
mod render_pass;

use crate::app::{App, RenderPipeline};
use glam::{f32::Vec2, IVec2};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 1024.0;
pub const WINDOW2_WIDTH: f32 = 512.0;
pub const WINDOW2_HEIGHT: f32 = 512.0;
pub const SCALING: f32 = 2.0;

/// Application state driven by the event loop.
struct Application {
    app: App,
    cursor_pos: Vec2,
    // An extremely crude way to handle input state... but works for this example.
    mouse_is_pressed_w1: bool,
    mouse_is_pressed_w2: bool,
    time: Instant,
}

impl Application {
    fn new() -> Application {
        Application {
            app: App::default(),
            cursor_pos: Vec2::ZERO,
            mouse_is_pressed_w1: false,
            mouse_is_pressed_w2: false,
            time: Instant::now(),
        }
    }

    /// Runs a single iteration of the main loop.
    fn tick(&mut self) {
        // Draw life on windows if mouse is down.
        draw_life(
            &mut self.app,
            self.cursor_pos,
            self.mouse_is_pressed_w1,
            self.mouse_is_pressed_w2,
        );

        // Compute life & render 60fps.
        if (Instant::now() - self.time).as_secs_f64() > 1.0 / 60.0 {
            compute_then_render_per_window(&mut self.app);
            self.time = Instant::now();
        }
    }
}

impl ApplicationHandler for Application {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create windows & pipelines.
        self.app.open(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if process_event(
            &mut self.app,
            window_id,
            &event,
            &mut self.cursor_pos,
            &mut self.mouse_is_pressed_w1,
            &mut self.mouse_is_pressed_w2,
        ) {
            event_loop.exit();
            return;
        }

        self.tick();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for (_, renderer) in self.app.windows.iter() {
            renderer.window().request_redraw();
        }

        self.tick();
    }
}

fn main() -> Result<(), impl Error> {
    println!("Welcome to Vulkano Game of Life\nUse the mouse to draw life on the grid(s)\n");

    // Create event loop.
    let event_loop = EventLoop::new().unwrap();

    // Create app with vulkano context.
    let mut application = Application::new();

    event_loop.run_app(&mut application)
}

/// Processes a single window event.
/// Returns true only if the window is to be closed.
pub fn process_event(
    app: &mut App,
    window_id: WindowId,
    event: &WindowEvent,
    cursor_pos: &mut Vec2,
    mouse_pressed_w1: &mut bool,
    mouse_pressed_w2: &mut bool,
) -> bool {
    match event {
        WindowEvent::CloseRequested => {
            if window_id == app.windows.primary_window_id().unwrap() {
                return true;
            } else {
                // Destroy window by removing its renderer.
                app.windows.remove_renderer(window_id);
                app.pipelines.remove(&window_id);
            }
        }
        // Resize window and its images.
        WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
            let vulkano_window = app.windows.get_renderer_mut(window_id).unwrap();
            vulkano_window.resize();
        }
        // Handle mouse position events.
        WindowEvent::CursorMoved { position, .. } => {
            *cursor_pos = Vec2::new(position.x as f32, position.y as f32)
        }
        // Handle mouse button events.
        WindowEvent::MouseInput { state, button, .. } => {
            let mut mouse_pressed = false;
            if button == &MouseButton::Left && state == &ElementState::Pressed {
                mouse_pressed = true;
            }
            if button == &MouseButton::Left && state == &ElementState::Released {
                mouse_pressed = false;
            }
            if window_id == app.windows.primary_window_id().unwrap() {
                *mouse_pressed_w1 = mouse_pressed;
            } else {
                *mouse_pressed_w2 = mouse_pressed;
            }
        }
        _ => (),
    }
    false
}

fn draw_life(
    app: &mut App,
    cursor_pos: Vec2,
    mouse_is_pressed_w1: bool,
    mouse_is_pressed_w2: bool,
) {
    let primary_window_id = app.windows.primary_window_id().unwrap();
    for (id, window) in app.windows.iter_mut() {
        if id == &primary_window_id && !mouse_is_pressed_w1 {
            continue;
        }
        if id != &primary_window_id && !mouse_is_pressed_w2 {
            continue;
        }

        let window_size = window.window_size();
        let compute_pipeline = &mut app.pipelines.get_mut(id).unwrap().compute;
        let mut normalized_pos = Vec2::new(
            (cursor_pos.x / window_size[0]).clamp(0.0, 1.0),
            (cursor_pos.y / window_size[1]).clamp(0.0, 1.0),
        );

        // Flip y.
        normalized_pos.y = 1.0 - normalized_pos.y;
        let image_extent = compute_pipeline.color_image().image().extent();
        compute_pipeline.draw_life(IVec2::new(
            (image_extent[0] as f32 * normalized_pos.x) as i32,
            (image_extent[1] as f32 * normalized_pos.y) as i32,
        ))
    }
}

/// Compute and render per window.
fn compute_then_render_per_window(app: &mut App) {
    let primary_window_id = app.windows.primary_window_id().unwrap();
    for (window_id, window_renderer) in app.windows.iter_mut() {
        let pipeline = app.pipelines.get_mut(window_id).unwrap();
        if *window_id == primary_window_id {
            compute_then_render(window_renderer, pipeline, [1.0, 0.0, 0.0, 1.0], [0.0; 4]);
        } else {
            compute_then_render(window_renderer, pipeline, [0.0, 0.0, 0.0, 1.0], [1.0; 4]);
        }
    }
}

/// Compute game of life, then display result on target image.
fn compute_then_render(
    window_renderer: &mut VulkanoWindowRenderer,
    pipeline: &mut RenderPipeline,
    life_color: [f32; 4],
    dead_color: [f32; 4],
) {
    // Skip this window when minimized.
    match window_renderer.window_size() {
        [w, h] => {
            if w == 0.0 || h == 0.0 {
                return;
            }
        }
    }

    // Start the frame.
    let before_pipeline_future =
        match window_renderer.acquire(Some(Duration::from_millis(1)), |swapchain_image_views| {
            pipeline
                .place_over_frame
                .recreate_framebuffers(swapchain_image_views)
        }) {
            Err(e) => {
                println!("{e}");
                return;
            }
            Ok(future) => future,
        };

    // Compute.
    let after_compute = pipeline
        .compute
        .compute(before_pipeline_future, life_color, dead_color);

    // Render.
    let color_image = pipeline.compute.color_image();
    let target_image = window_renderer.swapchain_image_view();

    let after_render = pipeline.place_over_frame.render(
        after_compute,
        color_image,
        target_image,
        window_renderer.image_index(),
    );

    // Finish the frame. Wait for the future so resources are not in use when we render.
    window_renderer.present(after_render, true);
}
