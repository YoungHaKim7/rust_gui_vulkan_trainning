Build succeeds. Two fixes:

1. **`Cargo.toml`** — added `image = "0.25"` as a direct dependency. The `features = ["image"]` on glfw only enables glfw's own cursor-from-image support; it doesn't make the `image` crate nameable from your code. `0.25` matches the version glfw 0.62 already pulls in, so no duplicate compilation.

2. **`examples/icon.png`** — the file didn't exist, so `open_image("examples/icon.png").unwrap()` at `src/main.rs:43` would have panicked on the next run. I generated a simple 64x64 RGBA icon there.

The stale `#[cfg(...)]` comments at the top of main.rs are leftovers from the original glfw-rs example (where the feature gate lived in the example crate itself) — they're harmless, but you can delete them.
