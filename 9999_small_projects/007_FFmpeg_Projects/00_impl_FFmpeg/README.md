# 구현 로드맵
- FFmpeg 구현은 크게 명령줄 도구 사용(CLI) 방식과 C/C++ 등의 프로그래밍 언어에서 라이브러리(libavcodec, libavformat 등)를 직접 호출하는 API 구현 방식으로 나뉩니다

## 1. FFmpeg 핵심 아키텍처 및 파이프라인 (CLI 기준)
- FFmpeg은 입력 파일을 읽어 디코딩, 필터링, 인코딩, muxing(합치기) 과정을 거쳐 출력합니다.
  - • 파이프라인 흐름: Demuxers (역방향 분할) → Decoders (디코딩) → Filtergraphs (필터 적용) → Encoders (인코딩) → Muxers (파일 포맷으로 묶기)
  - • 기본 명령어 구조:

```bash
ffmpeg -i input.mp4 [옵션] output.mp4
```

  - • 스트림 복사 (-c copy): 디코딩과 인코딩 과정을 생략하여 화질 저하 없이 매우 빠르게 변환(Remuxing)합니다.

## 2. 프로그래밍 언어에서의 라이브러리(Libav) 구현
- C/C++ 또는 외부 래퍼(Android의 FFmpeg Kit 등)를 이용해 코드로 구현할 때 사용하는 핵심 라이브러리들입니다.
  - • libavformat: 미디어 컨테이너 포맷(MP4, MKV 등)을 읽고 쓰는 역할 (AVFormatContext)
  - • libavcodec: 오디오/비디오 코덱의 인코딩 및 디코딩 담당 (AVCodec, AVCodecContext)
  - • libavutil: 메모리 관리, 수학 계산 등 공통 유틸리티 제공
  - • libswscale / libswresample: 비디오 해상도/픽셀 포맷 변환 및 오디오 리샘플링

## 3. 구현 시 주요 고려사항
- • 타임스탬프 동기화: 비디오와 오디오 스트림 간의 PTS(Presentation Timestamp)와 Timebase를 정확히 계산해야 싱크가 맞습니다.
- • 하드웨어 가속: NVIDIA(NVDEC/NVENC), AMD 등의 하드웨어 가속 기능을 활성화하면 대규모 미디어 처리 시 CPU 부하를 크게 줄일 수 있습니다.


### 구체적으로 어떤 환경에서 구현을 원하시나요?
- • 명령줄(CLI) 스크립트 작성법
- • C/C++ libav API를 이용한 커스텀 디코더/인코더 개발
- • 안드로이드나 iOS 모바일 앱 내 적용

- 필요한 목표 작업을 말씀해주시면 알맞은 코드를 안내해 드릴게요.
