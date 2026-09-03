완료되었습니다. Rust 포트가 완성되었으며 전체 테스트 스위트가 모두 통과되었습니다 (:green_확인 표시:).

## 요약

**`6502-rs/`** — C++ 에뮬레이터를 충실하게 1:1로 Rust로 포팅한 것으로, 두 코드베이스를 나란히 비교해 볼 수 있도록 원본은 그대로 유지했습니다.

```
6502-rs/
├── src/
│   ├── cpu.rs   Cpu, StatusFlags, 모든 opcode 상수, execute big-match,
│   │            주소 지정 모드 11개, load_prg, print_status (~1,300줄)
│   └── mem.rs   64KB 플랫 Mem, Index<u16> 사용 → mem[0xFFFC]가 그대로 작동함
├── tests/       14개 파일 — C++ GoogleTest 스위트의 전체 1:1 변환
└── examples/demo.rs   JSR/RTS 서브루틴이 포함된 간단한 .prg 프로그램
```

**검증 결과**
- `cargo test`: 14개 파일 모두에서 **362개 테스트 통과, 0개 실패** (ADC/SBC 87, CMP 56, shifts 55, AND/EOR/ORA 45, 나머지 119개) — C++ 스위트와 동일한 사이클 수 및 플래그 검증이 포함됩니다.
- `cargo clippy --all-targets`: 깨끗함 · `cargo build --release`: 깨끗함 · `cargo run --example demo`: 21 사이클 실행, 마지막에 `A=0x42 X=2` 출력됨.

**학습 과정에서 주목할 만한 Rust 번역 결정 사항**
- C++의 `union { Byte PS; StatusFlags Flag; }`는 `StatusFlags::to_byte()/from_byte()`를 통해 비트를 패킹하는 **bool 필드**가 있는 `flags: StatusFlags`가 되었습니다 (`unsafe` 없음). `cpu.PS` → `cpu.ps()`.
- C++ 멤버 포인터 테스트 헬퍼 (`Byte CPU::*RegisterToTest`)는 **클로저** (`|cpu| &mut cpu.a`)가 되었고, 테스트 픽스처는 `setup()` 함수가 되었습니다.
- Rust 디버그 빌드에서 오버플로우 시 패닉이 발생하므로, 6502의 오버플로우를 활용한 모든 연산 (`INX` 랩어라운드, 제로 페이지 `+X` 등)은 명시적인 `wrapping_*` 산술 연산을 사용합니다.
- 원본의 특이한 동작(quirks)은 그대로 보존되었습니다. 10진 모드는 여전히 패닉 상태이며, PLP는 B/Unused를 지우고, JMP (ind) 페이지 경계 버그는 수정되지 않은 채로 남아 있습니다 (TODO 주석 포함).

README의 "Rust port" 섹션에 `cargo test` / `cargo run --example demo` 명령어와 향후 참고할 수 있도록 제공해주신 C64 링크(메모리 맵, 리셋 프로세스 — flat RAM 대신 매핑된 버스와 $FFFC/$FFFD 하드웨어 리셋을 위한)를 추가했습니다. 아직 커밋되지 않았으므로, 먼저 검토해 보시고 원하시면 커밋하도록 하겠습니다.
