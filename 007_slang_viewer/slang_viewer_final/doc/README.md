# fix 1
- Fixed and verified — the app now runs cleanly with the Khronos validation layer reporting zero errors (exit 124 is just the timeout killing the GUI app; previously it panicked instantly).

## Root cause

The `ERROR_UNKNOWN` came from the shader side — two problems, both visible in the disassembled SPIR-V slangc produced:

**1. Entry point name mismatch (the panic trigger).** slangc renames SPIR-V entry points to `"main"`:

```
OpEntryPoint Vertex %vertMain "main" ...
OpEntryPoint Fragment %fragMain "main" ...
```

But `src/main.rs` requested `PName = "vertMain"` / `"fragMain"`. A missing entry point name is undefined behavior per the spec — the driver returned `VK_ERROR_UNKNOWN`.

**2. Illegal `DrawParameters` capability.** slangc emits an *unused* `BuiltIn BaseVertex` input variable (plus `OpCapability DrawParameters`) for any shader using `SV_VertexID`. That capability is only legal if the `shaderDrawParameters` device feature is enabled — the code enabled no features.

## Changes

- `build.rs`: added `-fvk-use-entrypoint-name` to both slangc invocations, so the SPIR-V entry names stay `vertMain`/`fragMain` and match what the pipeline requests.
- `src/main.rs`:
  - Bumped instance `api_version` to 1.1 (`PhysicalDeviceVulkan11Features` lives in the Vulkan 1.1 feature chain, and `shaderDrawParameters` was removed from the core `VkPhysicalDeviceFeatures` struct in the headers ash 0.38 tracks).
  - Asserts the device supports Vulkan 1.1 and that `shader_draw_parameters` is supported (queried via `get_physical_device_features2`).
  - Enables it through `push_next(&mut enabled_features)` on `DeviceCreateInfo`.

One latent issue I left alone since it's explicitly commented as intentional: the command buffer is recorded once against framebuffer 0 only (`src/main.rs:521`), so whenever `acquire_next_image` returns an index ≠ 0 you present an unrendered image — the triangle will flicker or show garbage under the surface. The standard fix is to reset and re-record the command buffer each frame for the acquired framebuffer; happy to do that if you want.

<hr />

# fix 2

Builds clean now — zero errors, zero warnings. Three things were fixed:

**1. The compile error** (`src/main.rs:110`)
```rust
vk::ImageViewType::_2D  →  vk::ImageViewType::TYPE_2D
```
Your ash version uses the Vulkan C-style constant names (same convention as the `SampleCountFlags::TYPE_1` / `Format::B8G8R8A8_UNORM` you already use).

**2. Unused imports** — removed `ext::debug_utils` and `CStr`.

**3. `unsafe_op_in_unsafe_fn` warnings** — this project uses the Rust 2024 edition, where unsafe operations are no longer allowed bare inside an `unsafe fn` body; they need an explicit `unsafe { }` block. The original error was hiding most of these in `new()` — once fixed, ~30 more surfaced there. I wrapped each unsafe fn body (`new`, `draw`, `destroy`) in a single explicit `unsafe { }` block, which keeps the behavior identical (the functions were already declared `unsafe`) while satisfying the lint.

I also silenced the `dead_code` warning on `entry`, `physical_device`, and `swapchain_images` with `#[allow(dead_code)]` + a keep-alive comment — they're held intentionally to keep those Vulkan objects/loader alive for the app's lifetime.
