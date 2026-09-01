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

- I have the full picture now. Notably, U+2699 (⚙) is not in the font's cmap (fc-query shows it's missing — the middle dot, arrows, and minus sign exist, but not the gear). So I'll draw the gear icon rocedurally with the existing rotated-quad primitives (same style as the delete "X"), rather than getting a tofu box. The −(U+2212) glyph for the "smaller" button does exist and I'll add it to the tlas charset.
  - For font scaling: the atlas is a static, once-built texture with exactly two rasterization sizes (Text=20px, Title=32px). The cleanest runtime-scaling approach is to pre-rasterize 5 discrete font levels at startup (so no GPU texture rebuild is needed) and let the settings window step between them.
- ● Agent "Explore todo app UI architecture" finished · 4m 14s

- ● The exploration confirms my plan and adds one improvement: the font does contain the Nerd Font gear glyph at U+F013 (Font Awesome "cog" — visually the ⚙ icon), even though U+2699 itself is unmapped. I'll use that glyph for a crisp gear instead of hand-drawn geometry, alongside the − (U+2212) glyph for the "smaller" button.
- 이제 전체 그림을 가지고 있어요. 특히 U+2699(⚙)는 폰트의 cmap에 포함되어 있지 않은데, fc-query에서는 점, 화살표, 마이너스 기호는 있지만 톱니바퀴 기호는 없다고 나옵니다. 그래서 기존에 회전된 쿼드 프리미티브들과 함께 기어 아이콘을 순서대로 그릴 거예요(‘X’ 삭제 스타일과 같아요), 두부 박스를 만드는 대신에요. "작은" 버튼을 나타내는 -(U+2212) 글리프가 실제로 존재하며, 이를 tlas 문자 집합에 추가할 예정입니다.
  - 글꼴 크기 조절용: 아틀라스는 정적으로 한 번 제작된 텍스처로, 정확히 두 가지 래스터화 크기(텍스트=20px, 제목=32px)를 가집니다. 가장 깔끔한 런타임 스케일링 방법은 시작 시 5개의 이산 폰트 레벨을 미리 래스터화하는 것(그래서 GPU 텍스처 재구성이 필요 없음)이며, 설정 창에서 이들 사이를 단계별로 조정하는 것입니다.
- ● 에이전트 "Explore todo app UI 아키텍처" 완료 · 4분 14초

- ● 이번 탐구를 통해 계획이 맞았다는 걸 확인했고, 한 가지 개선점도 발견했습니다. U+F013(Font Awesome의 'cog' — 시각적으로는 ⚙ 아이콘)에 Nerd 폰트의 기어 글리프가 포함되어 있다는 점입니다. 비록 U+2699 자체는 매핑되어 있지 않지만요. 손으로 그린 기하학 대신 그 글리프를 크리스프 기어에 사용할 거고, '작은' 버튼에는 - (U+2212) 글리프를 함께 쓸 겁니다.이제 전체 그림을 가지고 있어요. 특히 U+2699(⚙)는 폰트의 cmap에 포함되어 있지 않은데, fc-query에서는 점, 화살표, 마이너스 기호는 있지만 톱니바퀴 기호는 없다고 나옵니다. 그래서 기존에 회전된 쿼드 프리미티브들과 함께 기어 아이콘을 순서대로 그릴 거예요(‘X’ 삭제 스타일과 같아요), 두부 박스를 만드는 대신에요. "작은" 버튼을 나타내는 -(U+2212) 글리프가 실제로 존재하며, 이를 tlas 문자 집합에 추가할 예정입니다.
  - 글꼴 크기 조절용: 아틀라스는 정적으로 한 번 제작된 텍스처로, 정확히 두 가지 래스터화 크기(텍스트=20px, 제목=32px)를 가집니다. 가장 깔끔한 런타임 스케일링 방법은 시작 시 5개의 이산 폰트 레벨을 미리 래스터화하는 것(그래서 GPU 텍스처 재구성이 필요 없음)이며, 설정 창에서 이들 사이를 단계별로 조정하는 것입니다.
- ● 에이전트 "Explore todo app UI 아키텍처" 완료 

- ● 이번 탐구를 통해 계획이 맞았다는 걸 확인했고, 한 가지 개선점도 발견했습니다. U+F013(Font Awesome의 'cog' — 시각적으로는 ⚙ 아이콘)에 Nerd 폰트의 기어 글리프가 포함되어 있다는 점입니다. 비록 U+2699 자체는 매핑되어 있지 않지만요. 손으로 그린 기하학 대신 그 글리프를 크리스프 기어에 사용할 거고, '작은' 버튼에는 - (U+2212) 글리프를 함께 쓸 겁니다.

Done — the settings menu is in, with font resizing across the whole app. `cargo build` is clean, all 10 tests pass, and I verified six rendered states visually via frame dumps.

## What was added

**Gear button (top-left)** — Sits left of the "ToDo" title. One note: I checked the font with `fc-query`, and **U+2699 is not covered** by Hack Nerd Font — it would render as nothing. The font does contain the Nerd Font gear at **U+F013** (the same ⚙ cog visually), so I added that codepoint to the atlas and use it. The button highlights on hover and opens the settings window.

**Settings window** — A modal centered over a dimmed scrim (`src/ui/screen.rs`):
- **Font size** row: `−` / `20 px` / `+` stepper, five steps (body text 16 → 28 px, title scales 26 → 44 px). Buttons disable at the limits.
- `Close` button; clicking the scrim outside the panel also closes, as does `Esc` (first Esc closes the window, only a second one quits).
- While open, the modal owns all input: clicks, typing, and scroll-wheel are withheld from the todo list behind it, and the input field is blurred.

**Runtime font scaling** — `font::Size` changed from a 2-variant enum to a `(kind, level)` pair with 5 levels, all pre-rasterized into the atlas at startup (so no GPU texture rebuild is needed when you change the size — the level just selects a different atlas band). All layout constants (rows, checkboxes, paddings, footer) scale with the level via `LAYOUT_SCALE`; the default level keeps the exact 20/32 px sizes the app had before.

**Persistence** — New `src/settings.rs`: the chosen size is saved to `settings.txt` (`font=N`) on every change and loaded at startup, clamped to the valid range.

## Verification

- `cargo test` — 10/10 (new tests: charset/index/level invariants, gear glyph has a bitmap at all sizes, settings roundtrip + clamping + garbage-file fallback).
- Headless renders (`TODO_DUMP_FRAME`, plus new dev hooks `TODO_SETTINGS_OPEN` / `TODO_FONT_LEVEL`) dumped and checked: default view, modal open, smallest/largest font, and modal at both extremes — gear renders as a gear (no tofu), all controls fit, nothing clipped or overlapping. The "28 px" shown at max size is the body-text size, which is what the stepper reports.

Run it as usual with `cargo run` — the gear is top-left next to the title. Files touched: `font.rs`, `atlas.rs`, `settings.rs` (new), `ui/{mod,theme,widgets,screen}.rs`, `app.rs`, `renderer.rs`, `todos.rs`, `main.rs`. Nothing is committed yet; `git status` shows the changes plus the new `src/settings.rs` if you want to review before committing.
