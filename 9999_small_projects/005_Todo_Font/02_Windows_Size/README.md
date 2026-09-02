# Windows Size 마지막으로 설정한 창 크기 기억하기260902

Done. Summary:

## Window-size persistence — implemented and verified live

**How it works** (committed in your `33e7d52`):
- `settings.txt` gains a `window=WxH` line (logical units, so display-scale changes don't distort it); garbage/zero values are ignored at load
- `RenderContext::new` takes the saved size, falls back to `940×640`, clamps to the `560×420` minimum
- Size is tracked on `Resized` (maximized/zero sizes skipped) and written once in `ApplicationHandler::exiting` — headless `TODO_DUMP_FRAME` runs never touch the file

**Verified on the real app** (driven under XWayland with a libX11 script): seeded `window=1100x700` → opened at exactly 1100×700 → resized 932×711 → closed → file updated → relaunch opened at 932×711. Also survived a 12-resize storm, saving and restoring `1140×790`.

## ⚠️ Your commit contains an intermediate state — please commit the working tree

`33e7d52` was taken mid-session and includes:
- my temporary `TEMP-DEBUG` instrumentation (`eprintln!`s and a `std::process::exit(2)` failure path), and
- **not** the final fix below — so the committed binary still crashes on rapid resizes

The working tree (`src/renderer.rs`, uncommitted) is the verified final version.

## Pre-existing resize crash — fixed in the working tree

Resizing crashed the app with `AccessConflict(DeviceRead)` — reproduced on unmodified HEAD (first resize). Root cause: vulkano releases a vertex buffer's CPU-write lock only when the reading frame's fence is *observed*, and `acquire_next_image` can hand back the same image back-to-back — so a frame could overwrite a buffer whose fence was never reaped (a failed flush during resize also leaves a buffer permanently locked). Fix: `wait_for_frame()` after acquire, before the write (plus in the recreate/OutOfDate paths). Verified: 12 rapid + 5 alternating resizes with zero panics, 36/36 tests, and `cargo check --release --target x86_64-pc-windows-gnu` passes.

I kept your committed `settings.txt` (`window=1674x895` — your real last size). Want me to commit the renderer fix (or fold it into `33e7d52` with `--amend`)?
