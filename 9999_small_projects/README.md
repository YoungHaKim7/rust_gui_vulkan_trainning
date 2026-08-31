- https://github.com/vulkano-rs/vulkano

# 내가 만든거 Vulkan 이 백엔드

- slang파일을 보기 위한 뷰어(시각화함)(slang파일만 가능)
  - https://github.com/YoungHaKim7/slang_files_viewer_shaders

- `vert` & `frag` shader file로 뷰어(시각화함)(slang, spv, / `frag` & `vert` 다 가능) 최종 완성본 260824
  - https://github.com/YoungHaKim7/vert_frag_viewer

- `glTF`파일 뷰어(`glb`파일) - Blender에서 glTF파일로 Export가능
  - https://github.com/YoungHaKim7/gltf_viewer_vulkan

## vulkan으로 과학 시뮬레이션 만들기 시리즈
- 나비에-스토크스 방정식(Navier-Stokes equations, NS equation)(유체역학)
  - https://github.com/YoungHaKim7/Navier-Stokes_equations_Vulkan/
- 태양계 시스템
  - https://github.com/YoungHaKim7/solar_system_simulation_vulkan_rust


## vulkan으로 GUI만들기 시리즈
- full Vulkan renderer + immediate-mode GUI + ToDo logic(성능은 구림. 교육용)
  - https://github.com/YoungHaKim7/todo_app_vulkan

## vulkan으로 게임 만들기 시리즈
- sdl3로 만든거 vulkan으로 변경(3d game기초)
  - https://github.com/YoungHaKim7/vulkan_woodeneye

<br />

<hr />

# SmallProject 분석
- The three reference projects (all vulkano + winit, pinned to the same git rev, same Gpu → RenderContext → App module skeleton):
  - Navier-Stokes — cleanest windowed skeleton: swapchain, dynamic-rendering pipelines, GpuFuture sync loop, headless PPM frame dump
    - https://github.com/YoungHaKim7/Navier-Stokes_equations_Vulkan
  - solar_system — vertex buffers, #[derive(Vertex)] pattern, graphics-pipeline factory, host-visible buffer writes
    - https://github.com/YoungHaKim7/solar_system_simulation_vulkan_rust 
  - vulkan_woodeneye — 3D camera math, mouse-look + WASD input handling, scissor/resize handling
    - https://github.com/YoungHaKim7/vulkan_woodeneye
    
