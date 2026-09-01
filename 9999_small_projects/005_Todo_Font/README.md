- ● The baseline builds. The current font is an 8×8 bitmap font scaled 2–3× with nearest-neighbor filtering — that's the blockiness. I'll replace it
  with the Hack Nerd Font TTF, rasterized anti-aliased at the exact pixel sizes the UI uses.
  - 기초가 다져집니다. 현재 사용 중인 폰트는 8×8 비트맵 폰트로, 2~3배 크기로 확대되고 최근접 이웃 필터링이 적용되어 있습니다 — 그래서 블록이 뭉개져 보이는 거죠. 내가 대신할게
  Hack Nerd 폰트 TTF를 사용하면, UI에서 사용하는 정확한 픽셀 크기로 래스터화된 안티앨리어싱이 적용됩니다. 

- Rasterizing TTF font… (8m 28s · ↓ 30.8k tokens)
```bash
  ⎿  ◼ Add ab_glyph and rasterize TTF font
     ◻ Rebuild glyph atlas with per-glyph quads
     ◻ Update UI text rendering and layout
     ◻ Verify: tests + visual frame dump
```

- "This is a 4x nearest-neighbor zoomed crop of
  text rendered in a GPU app. Look closely at the glyph edges. 
  - Question 1: Do the letter edges show smooth multi-level anti-aliasing (grayscale intermediate pixels forming curved strokes), or hard binary edges with stair-stepping (pure on/off pixels, jagged diagonals like classic bitmap
  fonts)?
  - Question 2: What font does this appear to be (it should look like 'Hack' with slightly boxy curves)?
  - Question 3: Does the letter 'o' or 'D' show smooth rounded curves? Be specific about whether intermediate gray pixels exist at edges.

# Fix

Done. The app now renders real anti-aliased Hack Nerd Font glyphs instead of a stretched 8×8 bitmap font. Verified end-to-end: compiles with no warnings, all 5 tests pass, and an off-screen frame dump (`/tmp/todo_dump/frame.png`) confirms smooth grayscale anti-aliasing at 4× zoom, correct descenders (`g`/`y`/`p` sit below the baseline), full glyph coverage including `~ ^ {} ·`, and working row clipping.

**What changed**

| File                                       | Change                                                                                                                                                                                    |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Cargo.toml`                               | Added `ab_glyph` for TTF rasterization                                                                                                                                                    |
| `src/font.rs`                              | Replaced the 95-glyph 8×8 bitmap array with the embedded TTF (`include_bytes!`), rasterized at 20px (text) and 32px (title) with per-pixel coverage, bearings, and advances               |
| `src/atlas.rs`                             | Packs both sizes + the white cell into one texture with 1px padding against filter bleed; exposes per-glyph UV quads and metrics via a `OnceLock` global                                  |
| `src/ui/mod.rs`, `widgets.rs`, `screen.rs` | Text now draws per-glyph quads positioned by baseline instead of uniform 8px cells; `Size::{Text,Title}` replaces the float scale constants, `line_height()` replaces `GLYPH_ADV * SCALE` |
| `src/renderer.rs`                          | Sampler switched Nearest→Linear, atlas image sized dynamically from the packed texture                                                                                                    |
| `src/todos.rs`                             | `sanitize` now accepts `·` (the font covers it, so the `Enter: add · Esc: quit` hint renders properly — before it was an invisible gap)                                                   |

**Quality details**: glyphs are rasterized at their exact on-screen pixel size (not scaled after the fact), the white rect cell samples a solid 4×4 region so linear filtering doesn't dim UI rectangles, and baselines are rounded to whole pixels for crisp vertical alignment.

One catch fixed along the way: ab_glyph's glyph space is y-down, so the ink-top metric is `-bounds.min.y` — the test suite now asserts this (`'M' ink must sit above the baseline`).

Note: `assets/font/` is still untracked in git — commit it with the code, since the build embeds it at compile time.

# 메뉴 추가하기 유니코드 이용함.
```
fontTools isn't available — let me check the font's coverage with fc-query instead.

● Bash(fc-query -f '%{charset}\n' /home/gy/my_projects/Rust_Lang/9999/9191919/todo_app
      _vulkan/assets/font/HackNerdFont-Regular.ttf | tr ' ' '\n' | awk -F- '{ if ($1
      ==…
```
