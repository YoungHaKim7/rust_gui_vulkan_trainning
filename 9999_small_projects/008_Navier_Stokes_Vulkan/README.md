수정되었습니다. 진단 과정과 수정 사항을 정리하면 다음과 같습니다:

## 근본 원인

이 크래시는 시뮬레이션 자체의 버그가 아니라, **하나의 `vkQueueSubmit`에 컴퓨트 디스패치(dispatches)와 렌더 패스(render pass)가 혼재**되어 발생하는 MoltenVK/Apple-Silicon 수준의 문제입니다. 이러한 혼합 제출(mixed submission)이 초당 ~60회 발생하면 GPU가 `MTLCommandBuffer`를 `kIOGPUCommandBufferCallbackErrorInvalidResource`와 함께 중단시키고, MoltenVK는 이를 `VK_ERROR_OUT_OF_DEVICE_MEMORY` / device-lost로 변환합니다. 이후 다음 `then_signal_fence_and_flush()`에서 페닉이 발생했던 것입니다 (`src/app.rs:391`).

체계적인 바이섹션(bisection)을 통해 범위를 좁혔습니다:

| 설정                                              | 결과            |
| ------------------------------------------------- | --------------- |
| 렌더 패스 + 그리기 + 표시, 컴퓨트 없음            | 안정적          |
| 컴퓨트 전용 제출 (스왑체인 없음, 창 있음)         | 안정적          |
| 컴퓨트 + 획득/표시, 렌더 패스 없음                | 안정적          |
| 컴퓤트 + *빈* 렌더 패스 (바인딩 또는 그리기 없음) | **크래시 발생** |
| 이미지 클리어 + 렌더 패스 (디스패치 없음)         | 안정적          |
| 컴퓤트 전후에 렌더 패스 배치                      | **크래시 발생** |

즉, 트리거되는 조건은 오직 "하나의 제출 안에 컴퓤트 디스패치 인코더와 렌더 인코더가 함께 존재"하는 것입니다. 단순히 두 개의 커맨드 버퍼로 분리하는 것은 해결책이 되지 못했습니다. vulkano는 연결된 `then_execute` 호출을 하나의 `vkQueueSubmit`로 병합하기 때문입니다.

## 수정 사항 (`src/app.rs:357-412`)

시뮬레이션 단계와 디스플레이 패스는 이제 두 개의 커맨드 버퍼에 기록되며, 컴퓤트 제출은 **먼저 개별적으로 플러시(flush)** 됩니다. 따라서 이는 두 개의 개별 `vkQueueSubmit`(두 개의 `MTLCommandBuffer`)이 되며, 여기서 컴퓤트 제출은 렌더+표시 제출이 대기하는 세마포어를 신호(signal)합니다:

```rust
let compute_future = sync::now(device)
    .then_execute(queue, compute_cb).unwrap().boxed();
compute_future.flush()...;          // submit #1: compute only

let future = compute_future
    .join(acquire_future)
    .then_execute(queue, render_cb).unwrap()
    .then_swapchain_present(...)
    .then_signal_fence_and_flush();  // submit #2: render + present
```

`renderer.rs`의 주석 한 줄만 문맥에 맞게 업데이트했습니다.

## 검증

- **3분 소크 테스트(soak)**: 크래시 없이 실행됨 (수정 전에는 약 30초 만에 재현 가능하게 크래시 발생)
- `NS_CHECK=1` (헤드리스 솔버 검증): 모든 검사 통과
- `NS_DUMP_FRAME`: 확인 가능한 염료 콘텐츠를 포함한 렌더링

한 가지 참고할 점은, 근본적인 원인이 드라이버 쪽에 있으므로, 이는 앱 측면에서의 해결책(wedging 작업을 피하는 것)이지 vulkano나 MoltenVK 자체의 버그가 수정된 것은 아니라는 점입니다. Cargo.toml에 지정된 vulkano git revision(`fb4cfdb`)이 아닌 출시된 vulkano 버전으로 전환하면 다른 동작이 보일 수 있습니다.
