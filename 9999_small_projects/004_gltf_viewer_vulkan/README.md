- https://github.com/YoungHaKim7/gltf_viewer_vulkan

# 코드분할

Everything checks out — `cargo check`, `clippy`, and the `--print` path all pass. The crate now follows the same lib/bin split as your `slang_files_viewer_shaders` sibling:

```
src/
├── lib.rs      (new, 31 lines — minimal)  docs + module decls + pub use app::run
├── main.rs     (107 → 5 lines)            one-liner: gltf_viewer_vulkan::run()
├── app.rs      +run() entry, DEFAULT_MODEL (arg parsing, --print dispatch, event loop)
├── debug.rs    +print_gltf()/print_node() (the --print scene-graph diagnostic)
├── renderer.rs  (unchanged — swapchain, shadow map, pipelines)
├── model.rs     (unchanged — glTF loading)
├── gpu.rs       (unchanged — device/queue/allocators)
├── scene.rs, camera.rs, texture.rs, shaders.rs  (unchanged)
```

**What moved where:**
- `main.rs` held the entry logic + ~50 lines of glTF printing — that went to `app.rs` (entry point, matches its "winit event handling + frame loop" role) and `debug.rs` (diagnostics, alongside `dump_frame`).
- `lib.rs` carries only the crate docs (updated to list `app` as the entry point and `debug` as both diagnostics) and module declarations.

**Why it works with no other edits:** all cross-module paths were already `crate::…`, which resolve identically with `lib.rs` as the crate root; `main.rs` just consumes the `pub use app::run` re-export.

One note: `run()` is `pub` (it's the crate's public API), while `App`, `print_gltf`, and internals stay `pub(crate)` — nothing else leaked.


# 파일 글씨로 보는거(초기버젼)

- https://github.com/YoungHaKim7/gltf_viewer_vulkan/commit/db1f640c66eb63e01fbe5fcd3619cf4bdbbffa2e

```bash
=== assets/BlenderUntitled.glb ===
node "Cube"
  mesh "Cube"
  primitive 0: mode=Triangles
    Positions: 24xVec3 (buffer view 0)
    Normals: 24xVec3 (buffer view 1)
    TexCoords(0): 24xVec2 (buffer view 2)
    material "Material": base_color=[0.8, 0.8, 0.8, 1.0], metallic=0, roughness=0.5

buffer[0]: 840 bytes
```
