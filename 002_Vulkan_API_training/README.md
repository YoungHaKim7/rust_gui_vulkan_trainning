# c++은 glfw가 유명하고 Rust의 첫시작은 vulkano

# Vulkan개념잡기(잘 정리됨)
- https://namu.wiki/w/Vulkan(API)
  - 내가 만든 프로그램 문서로 설명 하기 좋다.
    - https://docs.rs/slang_files_viewer_shaders/latest/slang_files_viewer_shaders/#2-the-vulkan-object-model

# vulkan의 핵심 개념
- OpenGL은 함수마다 Command Buffer를 작성하고 Queue에 제출하는 방식을 사용하는데, Queue에 제출을 할 때 시간이 조금 걸린다. Vulkan은 일일이 Command Buffer를 작성할 때마다 Queue에 제출하지 않고, Command Buffer를 따로 다 작성 후 마지막에 한번에 다 제출하는 방식을 쓴다. 이 때문에 싱글스레드로 사용하여 멀티코어를 활용하는 병렬 처리가 아니더라도 성능은 OpenGL보다 뛰어나다.
- https://namu.wiki/w/Vulkan(API)

<img src="https://github.com/YoungHaKim7/vulkan-tutorial-rust_cpp/blob/main/assets/gallery.jpg" />

# Vulkano-Shaders
- https://docs.rs/vulkano-shaders/latest/vulkano_shaders/

# (Unity 지만 잘 정리됨)버텍스 및 프래그먼트 셰이더 예제(Vertex and fragment shader examples)
https://docs.unity3d.com/kr/2017.1/Manual/SL-VertexFragmentShaderExamples.html

