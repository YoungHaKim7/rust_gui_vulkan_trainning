# link

<hr />


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


# Figure 1 - Vulkan API
- https://gpuopen.com/news/v-ez-brings-easy-mode-vulkan/


# Figure 2- V-EZ middleware layer
- https://gpuopen.com/news/v-ez-brings-easy-mode-vulkan/
  

# Vulkan is a layered architecture, made up of the following elements:
- https://vulkan.lunarg.com/doc/view/1.3.280.0/windows/LoaderInterfaceArchitecture.html
- The Vulkan Application
- The Vulkan Loader
- Vulkan Layers
- Drivers
- VkConfig


# Graphics pipeline basics
- https://vulkan.lunarg.com/doc/view/1.4.321.0/mac/antora/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/00_Introduction.html

<img width="403" height="643" alt="Image" src="https://github.com/user-attachments/assets/106becb5-d168-4f5d-8280-f3eb95ba0ed8" />


# Descriptor(디스크립터)
- Descriptor(디스크립터)는 어떤 대상의 성질, 특징, 정보 등을 서술하거나 식별하는 말(설명어) 또는 컴퓨터 분야에서 특정 데이터나 자원의 속성을 정의하고 관리하는 제어 정보(기술자)를 뜻합니다. 
- https://m.blog.naver.com/PostView.naver?blogId=lifeeconom&logNo=30144266880

- 일반 영어 단어로는 '기술하는 것', '표현하는 말'을 의미하며, 문맥에 따라 다음과 같이 다르게 쓰입니다.
- 일반 의미 (언어·정보)
- 설명어 / 색인어: 문서나 데이터의 주제, 개념을 나타내기 위해 붙이는 핵심 단어나 키워드.
- 특징 표현: 어떤 사물이나 대상의 상태·특성을 나타내는 정보 조각.

## 컴퓨터·기술 분야 의미

### 파일 디스크립터 (File Descriptor):
- 파일 디스크립터 (File Descriptor): 유닉스/리눅스 계열 운영체제에서 프로세스가 파일이나 입력·출력 리소스(소켓 등)에 접근할 때 사용하는 음수가 아닌 정수 번호. 
  - https://code4human.tistory.com/m/123


### 특징 디스크립터 (Feature Descriptor):
- 특징 디스크립터 (Feature Descriptor): 이미지 처리(OpenCV 등)에서 영상이나 객체의 특징점(Keypoint) 주변 픽셀 정보를 수치로 변환하여 표현한 데이터. 
  - https://dsbook.tistory.com/147


### 파이썬 디스크립터 (Python Descriptor)
- 파이썬 디스크립터 (Python Descriptor): 파이썬에서 속성(Attribute) 접근을 제어하기 위해 특정 메서드(__get__, __set__ 등)를 구현한 특별한 객체. 
  - https://wikidocs.net/168363


### 함수 디스크립터 (Function Descriptor)
- 함수 디스크립터 (Function Descriptor): 프로그래밍에서 함수의 입력값과 반환 타입 등을 간략하게 설명하는 시그니처 형태.
  - https://inpa.tistory.com/entry/%F0%9F%9A%80-Function-Descriptor
