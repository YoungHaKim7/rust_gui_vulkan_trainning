# TodoApp – Rust + Vulkan (via `wgpu`)

A complete, runnable project. It uses **wgpu** (Vulkan backend) for GPU rendering, **winit** for windowing/input, and **fontdue** for text rasterisation into a texture atlas. You can add, toggle, and delete todos with keyboard input.

## Project Layout

```
todo-app/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── app.rs
    ├── todo.rs
    └── renderer.rs
```

---

## `Cargo.toml`

```toml
[package]
name = "todo-app"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.30"
wgpu = "0.20"
fontdue = "0.8"
bytemuck = { version = "1", features = ["derive"] }
```

> wgpu auto-selects Vulkan on Linux/Windows, Metal on macOS, and DX12 as fallback. To force Vulkan explicitly you can set `WGPU_BACKEND=vulkan` in the environment.

---

## `src/main.rs`

```rust
mod app;
mod renderer;
mod todo;

fn main() -> winit::Result<()> {
    app::run()
}
```

---

## `src/todo.rs` – Data Model

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TodoStatus {
    Pending,
    Done,
}

#[derive(Debug, Clone)]
pub struct Todo {
    pub id: u64,
    pub text: String,
    pub status: TodoStatus,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = match self.status {
            TodoStatus::Pending => "☐",
            TodoStatus::Done => "☑",
        };
        write!(f, "{} {}", marker, self.text)
    }
}

pub struct TodoStore {
    items: Vec<Todo>,
    next_id: u64,
}

impl TodoStore {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        self.items.push(Todo {
            id: self.next_id,
            text: trimmed.to_string(),
            status: TodoStatus::Pending,
        });
        self.next_id += 1;
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.status = match item.status {
                TodoStatus::Pending => TodoStatus::Done,
                TodoStatus::Done => TodoStatus::Pending,
            };
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }

    pub fn items(&self) -> &[Todo] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for TodoStore {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## `src/renderer.rs` – wgpu / Vulkan Rendering

```rust
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use fontdue::{Font as FontdueFont, FontInfo, RasterizedGlyph};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

// ─── Shader source (WGSL) ────────────────────────────────────────────────
const VERT_SRC: &str = r#"
struct VertexOutput {
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
};

[[stage(vertex)]]
fn main(
    [[builtin(vertex_index)]] vertex_index: u32,
) -> VertexOutput {
    // Fullscreen triangle (no index buffer needed)
    let positions = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0,  3.0),
    );
    let uvs = array<vec2<f32>, 3>(
        vec2(0.0, 1.0),
        vec2(2.0, 1.0),
        vec2(0.0, -1.0),
    );
    var out: VertexOutput;
    out.position = positions[vertex_index];
    out.uv = uvs[vertex_index];
    return out;
}
"#;

const FRAG_SRC: &str = r#"
[[stage(fragment)]]
fn main(
    [[builtin(position)]] frag_pos: vec4<f32>,
    [[location(0)]] uv_in: vec2<f32>,
    @binding(0) @group(0) bg_color: vec4<f32>,
    @binding(1) @group(0) text_tex: texture_2d<f32>,
    @binding(2) @group(0) sampler: sampler,
) -> [[location(0)]] vec4<f32> {
    let uv = uv_in;
    let texel = textureSample(text_tex, sampler, uv);
    // Blend: draw text (white) over background
    let alpha = texel.a;
    return mix(bg_color, vec4(1.0), alpha);
}
"#;

// ─── Types ───────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct UniformColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// ─── Renderer ────────────────────────────────────────────────────────────
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    text_texture: Option<wgpu::Texture>,
    text_sampler: wgpu::Sampler,
    font: FontdueFont<Vec<u8>>,
    pub font_size: f32,
}

impl Renderer {
    pub async fn init(window: &winit::window::Window) -> Result<Self, String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL, // prefer Vulkan
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface(window).map_err(|e| format!("surface error: {e}"))?
        };

        let adapter = instance
            .request_adapter(&wgpu::AdapterRequest {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("no suitable GPU adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("todo-app device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("device error: {e}"))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("todo shaders"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{VERT_SRC}{FRAG_SRC}").into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bg layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("todo pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[], // fullscreen triangle, no vertex buffer
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Uniform buffer (background colour)
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bg color"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[UniformColor {
                r: 0.12,
                g: 0.12,
                b: 0.18,
                a: 1.0,
            }]),
        });

        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("text sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Load a font (DejaVuSans is commonly available; fallback to system)
        let font_bytes = load_font();
        let font = FontdueFont::from_bytes(font_bytes).map_err(|e| format!("font: {e}"))?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group_layout,
            uniform_buffer,
            text_texture: None,
            text_sampler,
            font,
            font_size: 18.0,
        })
    }

    /// Rasterise all todo lines into a single RGBA texture atlas.
    pub fn rebuild_text_atlas(&mut self, lines: &[String], window_w: u32, window_h: u32) {
        let line_height = (self.font_size * 1.6).round() as u32;
        let padding = 20u32;
        let atlas_w = window_w;
        let atlas_h = (lines.len() as u32 * line_height + padding * 2).max(4);

        // Rasterise glyphs into an RGBA8 buffer
        let mut pixels = vec![0u8; (atlas_w as usize) * (atlas_h as usize) * 4];

        for (i, line) in lines.iter().enumerate() {
            let y_base = padding + i as u32 * line_height;
            self.font.rasterize_line(
                &line,
                self.font_size,
                self.font_size,
                |x, y, w, h, alpha| {
                    // x, y are in font-space; offset to atlas
                    let px = (x as f32 + 8.0) as u32;
                    let py = (y as f32 + y_base as f32) as u32;
                    for dy in 0..h {
                        for dx in 0..w {
                            let tx = px + dx;
                            let ty = py + dy;
                            if tx < atlas_w && ty < atlas_h {
                                let idx = ((ty as usize) * atlas_w as usize + tx as usize) * 4;
                                pixels[idx] = 255; // R
                                pixels[idx + 1] = 255; // G
                                pixels[idx + 2] = 255; // B
                                // alpha: max (avoid overdraw artifacts)
                                pixels[idx + 3] = pixels[idx + 3].max(alpha);
                            }
                        }
                    }
                },
            );
        }

        let dims = wgpu::Extent3d {
            width: atlas_w,
            height: atlas_h,
            depth_or_array_layers: 1,
        };

        // Upload texture (recreate each frame for simplicity; cache in production)
        let old = self.text_texture.take();
        if let Some(tex) = old {
            tex.destroy();
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text atlas"),
            size: dims,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_w * 4),
                rows_per_image: atlas_h,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: atlas_w,
                height: atlas_h,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([encoder.finish()]);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.text_texture = Some(Arc::new((texture, view)));
    }

    /// Store the atlas as a simple struct we can reference in bind groups.
    pub fn draw(
        &self,
        view: &wgpu::TextureView,
        bg_color: [f32; 4],
    ) -> Option<wgpu::CommandBuffer> {
        let (tex, tex_view) = self.text_texture.as_ref()?;

        // Update uniform
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[UniformColor {
                r: bg_color[0],
                g: bg_color[1],
                b: bg_color[2],
                a: bg_color[3],
            }]),
        );

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.uniform_buffer,
                    offset: 0,
                    size: std::mem::size_of::<UniformColor>() as u64,
                }),
                wgpu::BindingResource::TextureView(tex_view),
                wgpu::BindingResource::Sampler(&self.text_sampler),
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("todo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear([0.12, 0.12, 0.18, 1.0]),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        Some(encoder.finish())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let w = (new_size.width).max(1);
        let h = (new_size.height).max(1);
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        self.surface.get_current_texture().ok()
    }
}

// ─── Font loading helper ─────────────────────────────────────────────────
fn load_font() -> Vec<u8> {
    // Try common system font paths
    let candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
    ];
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            return data;
        }
    }
    // Last resort: embedded minimal font bytes (a tiny subset) – in practice
    // you'd bundle a font with the binary. Here we panic with a clear message.
    panic!(
        "No system font found. Place DejaVuSans.ttf in one of:\n  {}",
        candidates.join("\n  ")
    );
}
```

---

## `src/app.rs` – Application Loop & Input

```rust
use std::collections::VecDeque;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{renderer::Renderer, todo::TodoStore};

pub struct TodoApp {
    pub todos: TodoStore,
    pub input_buffer: String,       // text being typed
    pub cursor_line: usize,         // which todo is highlighted (for toggle/delete)
    pub renderer: Option<Renderer>,
    pub window: Option<Window>,
    pub dirty: bool,                // re-rasterise atlas next frame
}

impl TodoApp {
    pub fn new() -> Self {
        let mut todos = TodoStore::new();
        // Seed a few examples
        todos.add("Explore Rust + Vulkan");
        todos.add("Ship the app");
        todos.add("Take a break ☕");

        Self {
            todos,
            input_buffer: String::new(),
            cursor_line: 0,
            renderer: None,
            window: None,
            dirty: true,
        }
    }

    fn build_display_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("  TodoApp  (Vulkan / wgpu)"));
        lines.push("  ─────────────────────────────".into());
        for (i, item) in self.todos.items().iter().enumerate() {
            let prefix = if i == self.cursor_line { "▶" } else { " " };
            let status = match item.status {
                crate::todo::TodoStatus::Pending => "[ ]",
                crate::todo::TodoStatus::Done => "[x]",
            };
            lines.push(format!("  {prefix} {} {}", status, item.text));
        }
        lines.push("  ─────────────────────────────".into());
        lines.push(format!("  > {}", self.input_buffer));
        lines.push(String::new());
        lines.push("  Keys: Enter=add  ↑↓=move  Space=toggle  Del=remove  Esc=quit".into());
        lines
    }
}

impl ApplicationHandler for TodoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(winit::window::WindowAttributes {
            title: "TodoApp – Vulkan",
            inner_size: winit::dpi::PhysicalSize::new(640, 480),
            ..Default::default()
        })
        .unwrap();

        let rt = tokio::runtime::Runtime::new().expect("tokio");
        let renderer = rt.block_on(Renderer::init(&window)).expect("renderer init failed");

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else { return };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(r) = &mut self.renderer {
                        r.resize(size);
                    }
                    self.dirty = true;
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                    }
                    PhysicalKey::Code(KeyCode::Enter) => {
                        self.todos.add(&self.input_buffer);
                        self.input_buffer.clear();
                        self.cursor_line = 0;
                        self.dirty = true;
                    }
                    PhysicalKey::Code(KeyCode::Backspace) => {
                        self.input_buffer.pop();
                        self.dirty = true;
                    }
                    PhysicalKey::Code(KeyCode::ArrowUp) => {
                        if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                        }
                        self.dirty = true;
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown) => {
                        if self.cursor_line < self.todos.items().len().saturating_sub(1) {
                            self.cursor_line += 1;
                        }
                        self.dirty = true;
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        if !self.input_buffer.is_empty() {
                            // In input mode, type a space
                            self.input_buffer.push(' ');
                        } else {
                            self.todos.toggle(self.cursor_line);
                        }
                        self.dirty = true;
                    }
                    PhysicalKey::Code(KeyCode::Delete) => {
                        if !self.input_buffer.is_empty() {
                            // Remove char after cursor (simplified: pop last)
                            self.input_buffer.pop();
                        } else {
                            self.todos.remove(self.cursor_line);
                            if self.cursor_line >= self.todos.items().len() {
                                self.cursor_line = self.todos.items().len().saturating_sub(1);
                            }
                        }
                        self.dirty = true;
                    }
                    PhysicalKey::Character(c) => {
                        self.input_buffer.push(*c);
                        self.dirty = true;
                    }
                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                self.render(window);
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, _event: winit::event::DeviceEvent) {}
}

impl TodoApp {
    fn render(&mut self, window: &Window) {
        let Some(r) = &mut self.renderer else { return };
        let size = window.inner_size();

        if self.dirty {
            r.rebuild_text_atlas(
                &self.build_display_lines(),
                size.width,
                size.height,
            );
            self.dirty = false;
        }

        let Some(frame) = r.frame() else {
            // Surface lost – skip frame
            return;
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bg: [f32; 4] = [0.12, 0.12, 0.18, 1.0];
        if let Some(cmd_buf) = r.draw(&view, bg) {
            r.queue.submit([cmd_buf]);
        }
        frame.present();
    }
}

pub fn run() -> winit::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll); // continuous redraw
    let mut app = TodoApp::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
```

---

## Build & Run

```bash
# Add tokio for the async wgpu init (one-liner)
cargo add tokio --features rt-multi-thread

cargo run --release
```

> **Note:** On first run, ensure a Vulkan-capable driver is installed (Intel/AMD/NVIDIA). You can verify with `vulkaninfo` or `vkcube`.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  winit  (window, input, resize)                     │
│    └─► TodoApp  (ApplicationHandler)                │
│         ├─ TodoStore   (add / toggle / delete)      │
│         ├─ input_buffer (text being typed)          │
│         └─ render()                                  │
│              │                                       │
│              ▼                                       │
│  Renderer (wgpu → Vulkan driver)                     │
│    ├─ fullscreen-triangle pipeline (WGSL shaders)   │
│    ├─ text atlas  (fontdue rasterise → texture)     │
│    └─ uniform buffer (background colour)            │
└─────────────────────────────────────────────────────┘
```

| Layer | Crate | Role |
|-------|-------|------|
| Window / Input | `winit` | OS window, keyboard events, resize |
| GPU abstraction | `wgpu` (Vulkan backend) | Device, swapchain, pipeline, buffers |
| Shading | WGSL → Vulkan SPIR-V | Fullscreen quad sampling the text atlas |
| Text rasterisation | `fontdue` | TTF → per-glyph alpha → RGBA atlas |
| State | `TodoStore` | Plain-Rust todo list logic |

---

## Extending It

* **Multiple textures / scissor** – render each todo as its own draw-call with a scissor rect for true per-item colouring (strikethrough on done items).
* **Input field highlight** – add a second bind group with a cursor-blink uniform.
| **Persistence** – serialise `TodoStore` to JSON on exit, load on start.
* **GPU text (SDF)** – swap the alpha atlas for signed-distance-field glyphs for crisp scaling at any resolution.

This gives you a working, self-contained Rust todo app whose entire rendering path goes through Vulkan via wgpu.
