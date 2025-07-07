# C++이랑 다른언어와 함께 정리중..
- https://github.com/YoungHaKim7/vulkan-tutorial-rust_cpp

<hr />

# link

<hr />

# rust_gui_vulkan_trainning

- https://github.com/vulkano-rs/vulkano


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
