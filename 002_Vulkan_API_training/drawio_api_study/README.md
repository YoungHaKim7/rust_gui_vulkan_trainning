# 그림 출처 
- `vulkan_api.dio` 여기서 수정하면 됨(drawio 익스텐스받아서 수정하기(VSCode))

- https://gpuopen.com/news/v-ez-brings-easy-mode-vulkan/


<img src="./vulkan_api.svg" />


# vulkan의 핵심 개념
- OpenGL은 함수마다 Command Buffer를 작성하고 Queue에 제출하는 방식을 사용하는데, Queue에 제출을 할 때 시간이 조금 걸린다. Vulkan은 일일이 Command Buffer를 작성할 때마다 Queue에 제출하지 않고, Command Buffer를 따로 다 작성 후 마지막에 한번에 다 제출하는 방식을 쓴다. 이 때문에 싱글스레드로 사용하여 멀티코어를 활용하는 병렬 처리가 아니더라도 성능은 OpenGL보다 뛰어나다.
- https://namu.wiki/w/Vulkan(API)

<img src="https://github.com/YoungHaKim7/vulkan-tutorial-rust_cpp/blob/main/assets/gallery.jpg" />
