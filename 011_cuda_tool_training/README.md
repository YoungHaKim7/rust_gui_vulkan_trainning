# cuda관련 자료들 잘 분석하

```
cccl-13-4 cuda-command-line-tools-13-4 cuda-compiler-13-4 cuda-crt-13-4 cuda-ctadvisor-13-4 cuda-cudart-13-4 cuda-cudart-devel-13-4
  cuda-culibos-devel-13-4 cuda-cuobjdump-13-4 cuda-cupti-13-4 cuda-cuxxfilt-13-4 cuda-documentation-13-4 cuda-driver-devel-13-4 cuda-gdb-13-4
  cuda-libraries-13-4 cuda-libraries-devel-13-4 cuda-nsight-compute-13-4 cuda-nsight-systems-13-4 cuda-nvcc-13-4 cuda-nvdisasm-13-4
  cuda-nvml-devel-13-4 cuda-nvprune-13-4 cuda-nvrtc-13-4 cuda-nvrtc-devel-13-4 cuda-nvtx-13-4 cuda-opencl-13-4 cuda-opencl-devel-13-4
  cuda-profiler-api-13-4 cuda-sandbox-devel-13-4 cuda-sanitizer-13-4 cuda-tileiras-13-4 cuda-toolkit-13-4 cuda-toolkit-13-4-config-common
  cuda-tools-13-4 cuda-visual-tools-13-4 gds-tools-13-4 libcublas-13-4 libcublas-devel-13-4 libcufft-13-4 libcufft-devel-13-4 libcufile-13-4
  libcufile-devel-13-4 libcuobjclient-13-4 libcuobjclient-devel-13-4 libcurand-13-4 libcurand-devel-13-4 libcusolver-13-4 libcusolver-devel-13-4
  libcusparse-13-4 libcusparse-devel-13-4 libnpp-13-4 libnpp-devel-13-4 libnvfatbin-13-4 libnvfatbin-devel-13-4 libnvjitlink-13-4
  libnvjitlink-devel-13-4 libnvjpeg-13-4 libnvjpeg-devel-13-4 libnvptxcompiler-13-4 libnvvm-13-4 nsight-compute-2026.3.0 nsight-systems-2026.3.2
```

# 260917 **[NVIDIA, Rust 네이티브 GPU 프로그래밍 지원 발표](<https://news.hada.io/topic?id=33815&utm_source=discord&utm_medium=bot&utm_campaign=5116>)**

- **CUDA Rust**는 다른 언어로 작성한 커널을 호출하는 래퍼를 넘어, GPU 커널 자체를 Rust로 작성하고 네이티브 PTX로 컴파일하는 두 가지 개발 경로를 제공함  
- **cuda-oxide**는 스레드와 메모리를 직접 제어하는 SIMT 방식이며, **cutile-rs**는 데이터 타일 단위로 계산을 작성하고 실제 스레드 배치를 컴파일러에 맡기는 방식임  
- NVIDIA는 **Tile을 먼저 선택**하고, 아키텍처별 제어나 직접적인 메모리 및 스레드 관리가 필요할 때 SIMT를 사용할 것을 권장함  
- 두 경로 모두 Rust의 **빌림과 소유권**으로 입력과 출력…

# Why cuda-oxide?#

### 🦀 Rust on the GPU
- Write GPU kernels with Rust’s type system and ownership model. Safety is a first-class goal, but GPUs have subtleties — read about the safety model.
- https://nvlabs.github.io/cuda-oxide/

# 260917 **[Windows에서 AMD GPU로 CUDA 실행하기](<https://news.hada.io/topic?id=33649&utm_source=discord&utm_medium=bot&utm_campaign=5116>)**
- **ZLUDA와 AMD HIP/ROCm을 조합**해, NVIDIA CUDA용 Windows 프로그램을 AMD GPU에서 실행할 수 있도록 설치/진단/실행 스크립트를 제공하는 프로젝트  
- GPU와 드라이버를 확인하고 필요한 구성 요소를 내려받은 뒤, 대상 프로그램에 **호환 DLL과 실행 경로를 설정**하는 과정을 자동화함  
- Radeon RX 9060 XT에서 CUDA용 LibTorch를 사용하는 **강화학습 모델의 추론과 학습을 실제로 완료**했으며, 비공개 DLL 없이 공식 배포물만으로 재현할 수 있음  
- 현재 검증된 GPU는 **RX 9060 XT뿐**이며, 다른 AMD GPU는 자동으로 …
