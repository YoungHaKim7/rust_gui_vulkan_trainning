# Result


```bash


```


# fix

- Compiles clean. The tutorial you're following (learn-wgpu) targets an older wgpu — you're on **wgpu 30.0.1**, which changed the surface API. Three fixes in `src/main.rs`:

**1. `get_current_texture()` now returns an enum, not a `Result`** (`src/main.rs:154`)
`wgpu::SurfaceError` no longer exists. `render()` now matches on `wgpu::CurrentSurfaceTexture`:
- `Success(texture)` → render normally
- `Suboptimal(texture)` → still render, but reconfigure the surface afterward
- anything else (`Timeout`/`Occluded`/`Outdated`/`Lost`/`Validation`) → returned as the new error type

**2. `present()` moved to `Queue`** (`src/main.rs:246`)
```rust
output.present();              // old
self.queue.present(output);    // wgpu 30 — Queue::present(SurfaceTexture)
```

**3. Error handling in the event loop** (`src/main.rs:335`)
The new enum has no `OutOfMemory` or `Other` variants — it now has `Occluded` (window minimized/hidden) and `Validation` instead. `Lost`/`Outdated` → reconfigure, the rest just log and skip the frame.
