Runs clean — it survived 6 seconds of full-speed rendering (60fps GPU compute+draw across both windows) with no panics or vulkano validation errors, until `timeout` killed it (SIGTERM), which is expected for a GUI app.

## What was wrong

The code was written against **vulkano 0.34 / winit 0.29** APIs. Beyond cosmetic signature changes, there was one structural issue: in vulkano 0.35, the raw `CommandBuffer` returned by `RecordingCommandBuffer::end()` **doesn't implement `PrimaryCommandBufferAbstract`**, so it can't be submitted through `GpuFuture::then_execute` at all. The supported path is `AutoCommandBufferBuilder`.

## Changes

**`src/main.rs`** — `App::open` needs an `&ActiveEventLoop`, which only exists *inside* the loop. Ported to the `ApplicationHandler` trait + `EventLoop::run_app` (also fixes the deprecated `run` warning):
- Windows/pipelines are created in `resumed()`, control flow set on `StartCause::Init`
- `process_event` now takes `(window_id, &WindowEvent)` since the trait splits those apart
- Per-event logic (mouse-draw + 60fps compute/render) lives in a `tick()` method called from `window_event` and `about_to_wait`

**`src/game_of_life.rs`** — `RecordingCommandBuffer::new(...)` → `AutoCommandBufferBuilder::primary(...)`, `.end()` → `.build()`. The builder methods keep the 0.34-style signatures: `bind_pipeline_compute(Arc<_>)`, `bind_descriptor_sets(point, Arc<PipelineLayout>, 0, set)` (no dynamic-offsets arg), `push_constants(layout, 0, pc)` takes push constants **by value**.

**`src/pixels_draw.rs`** — same switch to `AutoCommandBufferBuilder::secondary(...)` (inheritance info moves from `CommandBufferBeginInfo` into the constructor). Returns `Arc<SecondaryAutoCommandBuffer>`. The `set_viewport` collect now works because the auto builder's parameter type (`SmallVec`) is inferable — no `&[Viewport]` target anymore. Removed the erroneous `&` on `bind_pipeline_graphics`.

**`src/render_pass.rs`** — `AutoCommandBufferBuilder::primary` + `build()` produces an `Arc<PrimaryAutoCommandBuffer>` that `then_execute` accepts directly (no `.into()`). `begin_render_pass`/`end_render_pass` take their infos by value, and `execute_commands` takes the secondary buffer's `Arc` as `Arc<dyn SecondaryCommandBufferAbstract>`.

A side benefit: the auto builder tracks resource usage, so it inserts the needed barriers between your two dispatches (compute step → color step) automatically.
