# Linux에서 WindowsOS테스트 하기

```bash
# target install
rustup target add x86_64-pc-windows-gnu 2>&1 | tail -2

  - Linux: cargo check --release ✓
  - Windows: cargo check --release --target x86_64-pc-windows-gnu
```

## 첫 오류

```bash
 cargo r --release
   Compiling todo_app_vulkan v0.1.0 (C:\Users\ytok1\OneDrive\Desktop\Finacial_working\todo_app_vulkan)
error[E0433]: cannot find `wayland_clipboard` in `copypasta`
   --> src\app.rs:237:24
    |
237 |             copypasta::wayland_clipboard::create_clipboards_from_external(display.display.as_ptr())
    |                        ^^^^^^^^^^^^^^^^^ could not find `wayland_clipboard` in `copypasta`
    |
note: found an item that was configured out
   --> C:\Users\ytok1\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\copypasta-0.10.2\src\lib.rs:29:9
    |
 20 |     unix,
    |     ---- the item is gated here
...
 29 | pub mod wayland_clipboard;
    |         ^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0433`.
error: could not compile `todo_app_vulkan` (bin "todo_app_vulkan") due to 1 previous error
PS C:\Users\ytok1\OneDrive\Desktop\Finacial_working\todo_app_vulkan>
error[E0433]: cannot find `wayland_clipboard` in `copypasta`
   --> src\app.rs:237:24
    |
237 |             copypasta::wayland_clipboard::create_clipboards_from_external(display.display.as_ptr())
    |                        ^^^^^^^^^^^^^^^^^ could not find `wayland_clipboard` in `copypasta`
    |
note: found an item that was configured out
   --> C:\Users\ytok1\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\copypasta-0.10.2\src\lib.rs:29:9
    |
 20 |     unix,
    |     ---- the item is gated here
...
 29 | pub mod wayland_clipboard;
    |         ^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0433`.
error: could not compile `todo_app_vulkan` (bin "todo_app_vulkan") due to 1 previous error
```

### WindowsOS 오류 수정최종

수정 완료 및 양쪽 플랫폼에서 검증했습니다.

## 원인

`copypasta::wayland_clipboard` 모듈은 copypasta 크레이트 내부에서 `#[cfg(unix)]`로 게이트되어 있어 Windows에서는 컴파일 자체가 안 됩니다. `make_clipboard`가 이 모듈을 무조건 참조해서 Windows 빌드가 깨졌습니다.

## 수정 (`src/app.rs:229`)

Wayland 분기를 `#[cfg(all(unix, not(target_os = "macos")))]` 블록으로 감쌌습니다 (copypasta 자체의 게이트와 동일하게 맞춤):

```rust
fn make_clipboard(window: &Window) -> Box<dyn ClipboardProvider> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
        if let Ok(handle) = window.display_handle()
            && let RawDisplayHandle::Wayland(display) = handle.as_raw()
        {
            let clipboard = unsafe {
                copypasta::wayland_clipboard::create_clipboards_from_external(
                    display.display.as_ptr(),
                )
            }
            .1;
            return Box::new(clipboard);
        }
    }
    let _ = window; // 분기가 컴파일 아웃된 플랫폼에서 unused 경고 방지
    ClipboardContext::new()
        .map(|ctx| Box::new(ctx) as Box<dyn ClipboardProvider>)
        .unwrap_or_else(|_| Box::new(NopClipboardContext::new().unwrap()))
}
```

Windows에서는 이 분기가 사라지고 `ClipboardContext`로 빠지는데, Windows에서 `ClipboardContext`는 copypasta의 Win32 클립보드 백엔드(`clipboard-win`)라서 기능도 동일하게 작동합니다.

## 검증

- Linux: `cargo check --release` ✓
- Windows: `cargo check --release --target x86_64-pc-windows-gnu` ✓ (경고 없음, `clipboard-win` 백엔드 정상 컴파일)

Windows 머신에서 `cargo r --release`를 다시 실행하면 됩니다.

# 윈도우에서 Font를 못찾는 오류

```bash
Vulkan ToDo
Controls: type + Enter = add task · click/drag in the input = caret/selection · Ctrl+A/C/X/V · Ctrl+Backspace = delete word · click checkbox = toggle · click a row's color stripe = cycle priority (red on top, yellow next, gray last) · X = delete · pencil = edit its text (Enter/Add saves, Esc cancels) · scroll = move list · settings: gear (top left) · Esc: close window / quit
Using device: NVIDIA GeForce RTX 3060 Ti (type: DiscreteGpu)
0 task(s) loaded from C:\Users\ytok1\OneDrive\Desktop\Finacial_working\todo_app_vulkan\todos.txt
no Korean fallback font found; Hangul will render blank
```

### font다른거 깔아서 해결

Done and verified. Here's the summary:

## What changed

**Root cause:** `fallback()` in `src/font.rs` only probed hardcoded Linux font paths, so Windows found nothing.

**Fix — the font is now embedded in the binary:**
- Instanced the 23 MB variable font down to a static Regular first: `fonttools varLib.instancer ... wght=400` → `assets/font/NotoSerifKR-Regular.ttf` (14 MB). This step mattered — ab_glyph can't apply `gvar` deltas, so the raw VF renders its thin default weight (at 16 px, Hangul strokes peaked at coverage 160–213, never solid). The Regular instance peaks at 255 from 20 px up.
- `FALLBACK_TTF` is embedded via `include_bytes!` (same as the Hack font), replacing the system-path list. Works identically on Windows/Linux — no font install, no env var, no shipping `assets/` next to the exe.
- `TODO_KOREAN_FONT` env override still works.
- New test `bundled_fallback_font_covers_hangul` guards coverage + ink darkness (would catch re-embedding the thin VF).

## Verification
- `cargo test`: **37 passed**
- `cargo check --release --target x86_64-pc-windows-gnu`: clean
- Ran the app from a scratch dir seeded with 4 Hangul todos: startup prints `Korean fallback font: bundled Noto Serif KR`, and the screenshot shows all Hangul rendering legibly (the dim gray you may notice is the app's existing text color — it applies to Latin too).

## Notes
- Rebuild the Windows exe (`cargo build --release --target x86_64-pc-windows-gnu`) and Hangul will render there.
- I installed `fonttools` via **pipx** (PEP 668 blocks system pip) — remove with `pipx uninstall fonttools` if unwanted.
- The 23 MB VF is still in `assets/font/`; it's the re-instancing source (e.g. `wght=500` if you'd like bolder Hangul), but feel free to delete it to slim the repo — the exe doesn't need it.
- Exe size grows ~+14 MB from the embedded font.
