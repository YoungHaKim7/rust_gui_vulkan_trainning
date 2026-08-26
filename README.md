# C++이랑 다른언어와 함께 정리중..
- https://github.com/YoungHaKim7/vulkan-tutorial-rust_cpp

<hr />

<p align="center">
  <img width=50px src="https://user-images.githubusercontent.com/67513038/213436632-820a1675-98d9-4626-979d-be63c60cdcb7.png" />
  <img width=35px src="https://user-images.githubusercontent.com/67513038/213403213-1b1b3efc-ce53-4825-9dfc-e9bf2956a7f4.svg" />
  <img width=70px src="https://github.com/YoungHaKim7/Cpp_Training/assets/67513038/1599aaad-3821-4abe-b40b-f7000f5ab0b7" />
</p>

<p align="center">
<!-- Rust version -->
<a href="https://www.rust-lang.org/" rel="nofollow noopener noreferrer">
  <img src="https://img.shields.io/badge/Rust-1.98+-orange.svg" alt="Rust">
</a>
<!-- Vulkan version -->
<a href="https://www.vulkan.org/" rel="nofollow noopener noreferrer">
  <img src="https://img.shields.io/badge/Vulkan-1.4-red.svg" alt="Vulkan">
</a>
</p>

<hr />

# link

- Vulkan공부하기 전 OpenGL기초 지식이 필요하다
- Learn OpenGL
  - https://learnopengl.com/PBR/Theory

<hr />

# rust_gui_vulkan_trainning

- https://github.com/vulkano-rs/vulkano

<hr />

# WebGPU headers
- https://webgpu-native.github.io/webgpu-headers/

<hr />

# Vulkan vs OpenGL


|-|OpenGL | Vulkan|
|-|-|-|
|Thread|Single-threading|Multi-threading|
|global<br> state<br> machine|One single global state machine |	Object-based with no global state|
|state<br> concepts|State is tied to a single context |	All state concepts are localized to a command buffer|
||Operations can only be executed sequentially |	Multi-threaded programming is possible|
|memory management|GPU memory and synchronization are usually hidden |	Explicit control over memory management and synchronization|
|checking at runtime|Extensive error checking |	Vulkan drivers do no error checking at runtime;<br> there is a validation layer for developers |

- Vulkan설명(나무위키) https://namu.wiki/w/Vulkan(API)

- https://en.wikipedia.org/wiki/Vulkan

- 그림으로 이해
  - OpenGL and Vulkan are both rendering APIs. In both cases, the GPU executes shaders, while the CPU executes everything else.

<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/e6/Division_of_labor_cpu_and_gpu.svg/500px-Division_of_labor_cpu_and_gpu.svg.png" />

- Vulkan
  - https://vkguide.dev/docs/extra-chapter/multithreading/
  - https://en.wikipedia.org/wiki/Vulkan
    - 한글 설명서
      - https://vkguide.dev/docs/ko

- OpenGL
  - https://en.wikipedia.org/wiki/OpenGL

# rust로 OpenGL공부하기
- https://github.com/bwasty/learn-opengl-rs
