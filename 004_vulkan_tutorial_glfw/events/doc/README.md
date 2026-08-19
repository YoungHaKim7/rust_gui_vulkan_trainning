Builds clean, no warnings. What changed in `src/main.rs`:

**Root cause:** `glfw.window_hint(...)` in GLFW sets *process-global* state, not per-window options. Your `HelloTriangleApplication` set `ClientApiHint::NoApi` (correct for Vulkan), and the events-example code you'd pasted into `main()` then created a second window that silently inherited it — so that window had no OpenGL context and `make_current()` panicked. On top of that, `make_current()` is inherently wrong for a `NoApi` Vulkan window.

**Fix — merged everything into the single Vulkan app window:**
- `main()` now just constructs the app and calls `app.run()` (src/main.rs:116) — no second `glfw::init_no_callbacks()`, no second window, no `make_current()`.
- All the `set_*_polling(true)` calls + `set_sticky_keys(true)` moved into `HelloTriangleApplication::new()` (src/main.rs:47) so every event type is deliverable.
- `main_loop` now routes every event through the full `handle_window_event` (src/main.rs:101) instead of the minimal Escape-only match — you get the whole events demo (pos/size/char/cursor/scroll/key/file-drop…) on the Vulkan window. Escape to quit still works via the handler.

If you ever *do* want a second OpenGL window alongside a Vulkan one, the trick is resetting the leaked hint before creating it: `glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGl))` — otherwise it inherits `NoApi` from the previous creation.

`cargo r` now opens one window that logs every event; close with Escape.
