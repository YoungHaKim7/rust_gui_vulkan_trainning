Done — the port builds clean (no warnings) and runs: exit 124 means it was killed by my 3-second timeout while sitting happily in the event loop with the window open.

## What changed

- **`src/main.cpp` → `src/main.rs`** — a 1:1 port of the [tutorial base code](https://github.com/Overv/VulkanTutorial/blob/main/code/00_base_code.cpp), keeping the same structure: `HelloTriangleApplication` with `new()` (constructor + `initWindow`), `run()`, `init_vulkan()` (empty, ready for the next chapters), `main_loop()`, `cleanup()`.
- **`Cargo.toml`** — uses the `glfw` crate (0.59) so the API maps directly onto the C++ (`glfwInit` → `glfw::init`, `glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API)` → `window_hint(ClientApiHint::NoApi)`, etc.). `ash` (Vulkan bindings) gets added in the next chapter when `initVulkan()` does real work.
- Deleted `src/main.cpp` and the stale CMake artifacts in `target/` (Cargo owns that directory now).

## Notable C++ → Rust differences

- **`cleanup()` is a no-op by design**: `glfwDestroyWindow`/`glfwTerminate` happen automatically via `Drop` (`PWindow` destroys the window, `Glfw` terminates the library) — Rust's RAII replaces the manual teardown.
- **`try/catch` in `main`** became a `Result` from `new()`; errors print to stderr and exit with code 1 (`EXIT_FAILURE`).
- Two small deviations from the original, both marked with comments: Escape also closes the window (convenient for testing), and the event channel is drained each frame (required with the `glfw` crate's message-passing API).

Run it with `cargo run`. Two leftovers you may want to deal with: `CMakeLists.txt`, `.clang-format`, and the C++ recipes in `justfile` are now dead tooling — say the word and I'll delete them or convert the justfile to cargo recipes (`r` → `cargo run`, etc.).
