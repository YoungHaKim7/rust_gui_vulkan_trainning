# fix (final)

- The user has committed the work themselves (`17b61ba` — "slang viewer complete, triangle blinking solved"), working tree clean, zero build warnings.

## Summary

**Cause of the blink:** the command buffer was recorded once at startup against framebuffer 0 only, but the swapchain cycles through 4 images. Whenever `acquire_next_image` returned an index ≠ 0, the app presented an image that was never rendered → the triangle appeared only on a fraction of frames.

**Fix** (`src/main.rs`):
- New `record_command_buffer(image_index)` method records the render pass against `self.framebuffers[image_index]` — the framebuffer for the image about to be presented.
- `draw()` now calls it every frame after acquiring. The explicit `vkResetCommandBuffer` isn't needed: the pool is created with `RESET_COMMAND_BUFFER`, so `begin_command_buffer` implicitly resets.
- Stored `swapchain_extent` on the struct for the per-frame render area.

**Verification:**
- 8 consecutive screenshots taken 250 ms apart: **exactly 15,492 red pixels inside the window in every frame** — a blink would show alternating ~15k/0.
- Soak tests: 3×20 s runs + 1×20 s with the Khronos validation layer — clean, zero validation messages.

**One honest caveat:** during testing I hit intermittent segfaults — crash inside the driver's `vkWaitForFences` jumping to unmapped memory (NVIDIA proprietary 595.84, RTX 3060 Ti), after 87–223 frames, with the crash appearing and later disappearing across runs of *identical* binaries. I verified no objects are destroyed early, handles stay valid, and validation reports nothing — it behaves like a race inside the brand-new 595 driver branch rather than an app bug. The final code (canonical `wait fence → acquire → record → submit → present` structure) survived every soak without reproducing it, but if you see it on your machine, a driver update would be the first thing to try.

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
