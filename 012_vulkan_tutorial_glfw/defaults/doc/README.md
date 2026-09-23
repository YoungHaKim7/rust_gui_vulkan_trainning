Fixed — compiles clean now, no warnings (verified with `cargo check --all-features`, which also exercises the `vulkan` branch):

1. **`extern "C"` → `unsafe extern "C"`** (src/main.rs:84, src/main.rs:88, src/main.rs:112) — required by edition 2024.
2. **Wrapped the call in `unsafe { }`** inside the `unsafe fn` (src/main.rs:109) — edition 2024's `unsafe_op_in_unsafe_fn` no longer lets an unsafe fn body call unsafe code implicitly.
3. **Declared the `vulkan` feature in Cargo.toml** — the `#[cfg(feature = "vulkan")]` gates refer to *this crate's* features, not glfw's (which is what `features = ["vulkan"]` on the dependency enables). Now `cargo run --features vulkan` will print the Vulkan support info; plain `cargo run` skips it.
