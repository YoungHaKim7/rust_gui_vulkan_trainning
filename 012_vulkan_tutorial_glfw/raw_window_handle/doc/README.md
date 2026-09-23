Fixed and verified — the app now prints `raw handle: XlibWindowHandle { window: 58720267, visual_id: 0 }` and runs its event loop normally.

## What was wrong

`glfw = { version = "0.62.0", features = ["raw-window-handle-v0-5"] }` — features are **additive** on top of defaults. glfw 0.62.0's `default` → `all` → already includes `raw-window-handle-v0-6`, so your explicit `raw-window-handle-v0-5` meant *both* RWH versions were compiled in. glfw's source isn't written to handle that combination (both code paths declare the same `raw_window_handle` items), hence the 11 errors.

Also, the `#[cfg(feature = "raw-window-handle-v0-6")]` gates in main.rs were copied from glfw's own example crate — those features don't exist in your package, so the cfg always took the `raw_window_handle_0_5` branch, which isn't a dependency of your crate anyway.

## Changes

**Cargo.toml** — drop the v0-5 feature (defaults give you RWH 0.6), and add the RWH 0.6 crate directly so your code can `use` it:
```toml
[dependencies]
glfw = "0.62.0"
raw-window-handle = "0.6"
```

**src/main.rs:16** — removed all the cfg juggling, keeping only the 0.6 API:
```rust
use glfw::{Action, Context, Key};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
```
and src/main.rs:31 — single version of the handle lookup:
```rust
let raw = window.window_handle().unwrap().as_raw();
```

(In RWH 0.6, `HasRawWindowHandle::raw_window_handle()` was replaced by the safe `HasWindowHandle::window_handle()` → `.as_raw()`.)
