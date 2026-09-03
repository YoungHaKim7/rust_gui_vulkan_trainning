# 유튜브 영상
- [240419) 엄청나게 빠른 탐욕스러운 메셔 - Voxel Engine 최적화 | Tantan](https://youtu.be/qnGoGq7DWMc?si=JY0ijmBB-ibsi9WD)
  - github code 
    - https://github.com/TanTanDev/binary_greedy_mesher_demo


# Convert Bevy voxel demo → Vulkan (vulkano) engine in vulkan_voxel_engine/


 - The repo root is a Bevy 0.13 voxel-engine demo (src/): chunked 32³ voxel terrain (bracket-noise), two meshing algorithms (binary greedy + culled-AO), a scanner that streams chunks in/out around the camera, async data/mesh tasks, a fly camera, a day/night sun cycle, packed-u32 vertices, wireframe toggle, and block carving.

 - The user wants this demo rebuilt with raw Vulkan via vulkano in vulkan_voxel_engine/ (Cargo.toml already stubbed with the same vulkano git rev `fb4cfdb` + winit 0.30 used by the two reference projects), following the architecture of:
 - solar_system_simulation_vulkan_rust/ — 3D app skeleton: Gpu (instance/device/queue/allocators), RenderContext (swapchain, pipelines, per-frame
   record), App (winit ApplicationHandler, input, redraw), GLSL shaders in assets/ loaded via vulkano_shaders::shader!, push constants, debug_dump_frame PPM path.
 - todo_app_vulkan/ — same skeleton plus: wait_for_frame() fence-wait pattern before overwriting per-swapchain-image buffers,
   - fill_mode_non_solid-style feature enabling, offscreen render + readback.

 - Environment verified: Rust 1.98, display available, NVIDIA/Intel/lavapipe Vulkan ICDs — we can build and run.

 - What gets ported / substituted

 
 |               Bevy concept                |                                            Vulkan port                                            |
 |-|-|
 | Bevy ECS plugins/systems                  | Plain structs + functions called in a fixed order in App::redraw                                  |
 | WGPU renderer + WGSL PBR material         | vulkano dynamic-rendering pipeline + GLSL chunk.vert/frag (Lambert sun + ambient + AO — Bevy<br /> PBR/cascade shadows not ported)  |
 | bevy_flycam                               | Custom fly camera (camera.rs): mouse-capture look, WASD + Space/Shift, speed 128, sensitivity <br /> 0.00015 (values from src/main.rs:69) |
 |  AsyncComputeTaskPool + Task + poll_once  | `std::thread::spawn` + `JoinHandle::is_finished()` polled each frame (identical spawn/poll/join <br /> semantics, no new dep)  |
 | bevy_screen_diagnostics (on-screen text)  | Window-title stats (pattern: solar set_title), updated ~2×/s                                      |
 | egui inspectors <br />(WorldInspector/AssetInspector)   | Dropped (editor-only plugins, no meaningful analog)                                |
 | Sun DirectionalLight day/night cycle      | Same cycle math; result is a sun-direction + intensity vec pushed to the shader                   |
 | Green circle base mesh (radius 22)        | Same disc rendered as a static packed-vertex mesh through the chunk pipeline                      |
 | Wireframe via 2nd material <br />(src/rendering.rs)   | 2nd pipeline: same shaders, PolygonMode::Line, device feature fill_mode_non_solid   |
 | Mesh entities / despawn                   | `HashMap<IVec3, ChunkGpuMesh>` (vertex+index Subbuffers) in RenderContext                          |


 - Kept 1:1 (engine-agnostic, just swap bevy::math → glam, bevy::utils::HashMap → std): voxel.rs, constants.rs, utils.rs, lod.rs, face_direction.rs, quad.rs, chunk.rs (terrain gen, same bracket-noise params), chunk_mesh.rs, chunks_refs.rs, ulled_mesher.rs, greedy_mesher_optimized.rs, scanner.rs, voxel_engine.rs (queues + modifications), sun.rs.

 - Files to create in vulkan_voxel_engine/

 - `Cargo.toml` — remove unused ab_glyph, copypasta, raw-window-handle (todo-app leftovers); add glam (vector math, API-compatible with bevy's re-exported glam: IVec3, ivec3, Vec3, Mat4, Quat), bracket-noise = "0.8.7", rand = "0.8".

 - src/main.rs — EventLoop + App::new + run_app (solar main.rs pattern).
 - src/gpu.rs — copy of solar Gpu (device pick, dynamic_rendering feature) plus fill_mode_non_solid: true in DeviceFeatures for wireframe.
 - src/camera.rs — fly camera: position: Vec3 (start (0, 2, 0.5)), yaw/pitch, update(dt, keys), view_matrix(), proj(aspect) via
   Mat4::perspective_rh_zo + Vulkan Y-flip; forward() helper for the carve ray.
 - src/app.rs — App { gpu, camera, sun, engine, scanner, rcx, input_state, ... }; ApplicationHandler (resumed/window_event/about_to_wait). Keys: R
   swap mesher, T toggle wireframe, N carve 32×32 random air blocks in the chunk 64 units ahead (port of modify_current_terrain, src/main.rs:78), I
   speed up sun, Esc quit, click capture/release mouse look. Mouse capture via set_cursor_grab(Confined→Locked fallback) + set_cursor_visible(false).
 - src/renderer.rs — RenderContext: window, swapchain, depth attachment (Format::D32_SFLOAT, one image per swapchain image, recreated in
   refresh_swapchain — neither reference has depth; standard vulkano RenderingInfo.depth_attachment pattern), fill + wireframe GraphicsPipelines
   (solar create_graphics_pipeline pattern + DepthStencilState default), ground-disc mesh, chunk_meshes: HashMap<IVec3, ChunkGpuMesh> with
   upload_chunk(pos, ChunkMesh) / remove_chunk(pos) / remove_all(), record_scene(...) (bind pipeline → push constants → per-chunk bind+draw indexed;
   disc first), wait_for_frame fence pattern from todo_app before any buffer overwrite/drop.
 - src/shaders.rs — vulkano_shaders::shader! modules for ../assets/chunk.vert|frag (solar shaders.rs pattern), push-constant struct derived from
   stages.
 - assets/chunk.vert / assets/chunk.frag — GLSL port of assets/shaders/chunk.wgsl: decode packed u32 (x|y|z 6 bits each, ao@18 (3b), normal@21 (3b),
   block@25 (7b) — same tables: 6 normals, block colors air/grass/dirt, AO lerp [1.0, 0.7, 0.5, 0.15]). Push constants (96 B < 128 B limit): mat4
   view_proj; vec4 chunk_origin; vec4 sun_dir_intensity;. Fragment: base_color * ao * (sky_ambient + sun_color * max(dot(n, sun), 0) * intensity),
   gamma-ish output matching the demo's look.
 - src/voxel.rs, src/constants.rs, src/utils.rs, src/lod.rs, src/face_direction.rs, src/quad.rs, src/chunk.rs, src/chunk_mesh.rs, src/chunks_refs.rs
   — direct ports; Quad::color/Color param dropped or replaced with ().
 - src/culled_mesher.rs, src/greedy_mesher_optimized.rs — direct ports (std HashMap).
 - src/scanner.rs — port: Scanner::new(12), functions become methods taking (&mut Scanner, cam_chunk_pos, &mut VoxelEngine); same spherical offset
   sets, same queue choreography.
 - src/voxel_engine.rs — port: VoxelEngine struct with same queues/maps; data_tasks: HashMap<IVec3, JoinHandle<ChunkData>>, mesh_tasks: Vec<(IVec3,
   JoinHandle<Option<ChunkMesh>>)>; spawn/join/unload functions; mesher toggle calls unload_all_meshes. App feeds finished meshes to
   renderer.upload_chunk and removes despawned ones.
 - src/sun.rs — cycle math from src/sun.rs (DAY 60 s / NIGHT 1 s, 450 ms ticks, I×6): produces sun_dir: Vec3 (direction toward sun) + intensity =
   sin²·1.0; shader adds a floor ambient so night stays visible.
 - src/debug.rs — offscreen 1280×800 render → PPM via VOXEL_DUMP_FRAME env var (solar debug.rs pattern, incl. depth image) for verification without
   interactive screenshots.

 Frame order in App::redraw (mirrors Bevy system ordering): sun tick → camera update → scanner (detect_move → scan_data/unloads → scan_mesh) →
 start_modifications → join data/mesh (upload finished GPU meshes, drop replaced) → unload data/mesh (drop GPU meshes) →
 start_data_tasks/start_mesh_tasks → acquire → record → present. Buffer drops/uploads ordered behind wait_for_frame().

 Technical notes / gotchas

 - Clip-space: Mat4::perspective_rh_zo (Vulkan 0..1 depth) pre-multiplied by Y-flip scale(1,-1,1); pipelines use cull_mode: None (vulkano default) so
   quad winding differences can't cause missing faces.
 - Wireframe: requires fill_mode_non_solid device feature (analog of the demo's WgpuFeatures::POLYGON_MODE_LINE); line_width: 1.0 avoids needing
  - Clip-space: Mat4::perspective_rh_zo (Vulkan 0..1 depth) pre-multiplied by Y-flip scale(1,-1,1); pipelines use cull_mode: None (vulkano default) so quad winding differences can't cause missing faces.
  - Wireframe: requires fill_mode_non_solid device feature (analog of the demo's WgpuFeatures::POLYGON_MODE_LINE); line_width: 1.0 avoids needing strict_features.
  - glam integer vectors: replace IVec3::distance_squared with (a - b).length_squared() helper to avoid API uncertainty.
  - Vertex data stays Vec<u32> (packed) → Buffer::from_iter with BufferUsage::VERTEX_BUFFER|INDEX_BUFFER per chunk; indices u32.
  - Start position/scanner distance from src/main.rs: camera (0.0, 2.0, 0.5), Scanner::new(12), chunk world offset = chunk_pos * 32.
  - Window title shows: FPS, total vertices, loaded chunks, queue lengths, mesher mode, wireframe state.

- Verification
  - 1. cargo build in vulkan_voxel_engine/ (must compile clean; fix as needed).
  - 2. cargo run — window opens, terrain streams in around spawn, WASD+mouse fly, day/night cycles.
  - 3. Interactive: R (mesher swap → meshes rebuild, vertex count in title drops for greedy), T (wireframe), N (blocks carved ahead), I (sun fast-forward), resize window (swapchain rebuild keeps working), Esc quits cleanly.
     4. Headless-ish check: VOXEL_DUMP_FRAME=/tmp/voxel_frame.ppm cargo run → inspect PPM shows lit terrain (solar-style debug path); also good
        regression check after pipeline changes.
     5. Compare against original: cargo run in repo root (Bevy) for visual reference.
