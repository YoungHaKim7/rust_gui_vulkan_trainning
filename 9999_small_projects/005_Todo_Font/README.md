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

Here is Claude's plan:
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 Full text editing in the input box

 Context

 The input field is currently append-only: Todos.input is a plain String, Backspace
 pops the last char (app.rs:80-84), typing always appends at the end, and the caret
 is always drawn at the text's end (screen.rs:116-127). There is no way to position
 the caret with the mouse, select text, delete words, or copy/paste.

 The user wants a real single-line editor in the input box:

 - Click to place the caret anywhere; drag with the mouse to select text
 - Ctrl+Backspace deletes the word before the caret (user confirmed standard behavior
   over "erase all"; erase-all = Ctrl+A + Backspace or triple-click + Backspace)
 - Arrow keys / Home / End to move the caret (Ctrl+arrows = word-wise, Shift = extend
   selection)
 - Copy / cut / paste via Ctrl+C / Ctrl+X / Ctrl+V (clipboard)

 Design

 Introduce a TextField model — text: String, caret: usize (byte index, always on a
 char boundary), anchor: usize (selection anchor; selection = min..max when caret !=
 anchor), scroll_x: f32 (horizontal view offset so the caret stays visible in
 overflowing text) — plus all editing operations as methods. Keyboard commands live
 in app.rs (winit events), mouse interaction policy lives in screen.rs (immediate
 mode, where layout is known); app.rs only relays raw mouse state.

 Note: sanitize (todos.rs:5-13) allows the multi-byte chars · (U+00B7) and −
 (U+2212), so all caret math must use char_indices()/char boundaries, never ±1 byte.

 Mouse mechanics: clicks already fire on release via pending_clicks → Ui::take_click.
 For drag-select, App additionally relays mouse_down: bool and the press origin
 press: [f32;2] into Ui each frame. screen.rs: while ui.mouse_down &&
 field.contains(ui.press), place/extend the selection (anchor fixed at press
 position, caret follows ui.mouse.x clamped); press-in-field also focuses immediately
 so the caret shows during the drag. Double/triple-click (within ~500 ms, ~6 px)
 selects word / all, tracked via a last_field_click timestamp in Todos.

 Clipboard (copypasta 0.10, already in the local cargo cache): create once when the
 window exists. Prefer Wayland: window.wayland_display() → unsafe {
 wayland_clipboard::create_clipboards_from_external(ptr).0 }; else
 ClipboardContext::new(); on failure NopClipboard. Stored as Box<dyn
 ClipboardProvider> in App; all get/set errors ignored (.ok()). Paste is filtered
 through sanitize with \n/\t → space, truncated to the existing 80-char cap.

 Changes

 1. Cargo.toml — add copypasta = "0.10".
 2. src/input.rs (new) — TextField struct + methods (all boundary-safe, unit-tested):
    - new, clear (resets caret/anchor/scroll), selection() -> Option<Range<usize>>,
      selected_text()
    - insert_str(&str, max_chars) — replaces selection, inserts at caret, truncates
      to cap
    - backspace, backspace_word, delete, delete_word (word ops skip whitespace run
      then the word run)
    - move_left/right(word, select), move_to_start/end(select), select_all
    - set_caret_from_x(x, text_x, Size) ->  and x_from_byte(byte, text_x, Size) —
      walk chars with atlas::global().advance() (pattern: text_width in
      ui/mod.rs:195-198)
    - #[cfg(test)] tests for all of the above, incl. multi-byte boundaries and an
      x↔caret roundtrip via atlas::global() (works in tests, cf. atlas.rs:187).
 3. src/main.rs — mod input; + one line in the module-map doc comment.
 4. src/todos.rs — input: TextField (replaces String), field_rect: Rect (offscreen
    default; updated by screen.rs each frame), dragging: bool, last_field_click:
    Option<(Instant, f32, f32)>. add_task calls self.input.clear(); sanitize
    unchanged. Update the struct-literal in the roundtrip test.
 5. src/app.rs
    - Track modifiers: WindowEvent::ModifiersChanged(m) => self.mods = m (winit 0.30
      KeyEvent has no modifiers field — verified).
    - Relay press state: on MouseInput Pressed set mouse_down, press = mouse, and
      todos.focused |= todos.field_rect.contains(mouse); on Released keep the
      existing pending_clicks push.
    - handle_keyboard gains self.mods logic: Ctrl+A/X/C/V (intercepted before the
      generic Key::Character arm so they never type letters); Backspace/Delete (+Ctrl
      = word); Left/Right/Home/End (+Ctrl word, +Shift select); Space/Character
      route through insert_str with sanitize (skip typing while Ctrl/Alt held); Enter
      unchanged. Every edit/move resets todos.caret_since (blink, existing pattern).
    - Clipboard init in resumed (wayland → x11 → nop, see Design); update the
      Controls: println to mention the new keys.
 6. src/ui/mod.rs — Ui gains pub(crate) mouse_down: bool and pub(crate) press: [f32;
    2] (filled by App before each frame alongside mouse/clicks).
 7. src/ui/theme.rs — pub(crate) const COL_SELECTION: [f32; 4] = [0.23, 0.52, 0.93,
    0.35]; (accent at low alpha).
 8. src/ui/screen.rs
    - Store todos.field_rect = field; press-in-field + drag-select and
      double/triple-click logic per Design (all x→caret math via TextField methods
      with tx, max_tx, Size::text(level) already at hand).
          double/triple-click logic per Design (all x→caret math via TextField methods
          with tx, max_tx, Size::text(level) already at hand).
        - Rendering: update scroll_x so x(caret) stays within [tx, max_tx]; draw the
          visible substring via the existing ui.text_clipped (ui/mod.rs:168) starting
          at the first partially-visible char, pen shifted by -scroll_x remainder;
          selection rect (COL_SELECTION) between clamped x(min(anchor,caret))..x(max);
          caret rect at clamped x(caret) (replaces the current
          fit_width-of-whole-string caret at screen.rs:117).

     Verification

     1. cargo build — copypasta 0.10.2 + x11-clipboard 0.9.3 + smithay-clipboard 0.7.3
        are already in the local cargo cache; if the network blocks the index, retry
        with --offline.
     2. cargo test — new input.rs tests + existing suites (all pure-CPU; no GPU
        needed).
     3. cargo run manual smoke test:
        - type; click mid-text and type/Backspace there; arrows/Home/End, Ctrl+arrows,
          Shift+arrows
        - drag selects; double-click selects a word; triple-click selects all;
          selection highlighted, typing/Backspace replaces it
        - Ctrl+Backspace deletes a word; Delete deletes forward
        - Ctrl+C/X/V round-trip with another app; pasting >80 chars or newlines
          sanitizes/truncates; long text scrolls to keep the caret visible
        - Enter adds the task and resets the field; settings modal open blocks all
          editing keys; Esc still quits

# 복사하기 붙혀넣기 기능추가
Done — the input box is now a full single-line editor. The app is running on your desktop; click into the input and try it.

**What you can do now**

| Action                                             | Result                                  |
| -------------------------------------------------- | --------------------------------------- |
| Click in the text                                  | Places the caret at any position        |
| Press + drag                                       | Selects text (highlighted in blue)      |
| Double-click / triple-click                        | Selects a word / everything             |
| ←/→, Home/End (+Ctrl = by word, +Shift = extend)   | Moves the caret / selection             |
| Ctrl+Backspace / Backspace / Delete (+Ctrl = word) | Deletes word-left / char-left / forward |
| Ctrl+A / C / X / V                                 | Select all / copy / cut / paste         |
| Typing over a selection                            | Replaces it                             |

Long text now scrolls horizontally so the caret always stays visible, and clicks map correctly against the scrolled text. Paste is sanitized (newlines/tabs become spaces, still capped at 80 chars). Per your choice, Ctrl+Backspace deletes the previous word; Ctrl+A + Backspace or a triple-click clears everything.

**How it's built**

- `src/input.rs` (new) — `TextField`: text + caret + anchor + scroll offset, every editing op as boundary-safe methods (the charset's `·`/`−` are multi-byte). 11 unit tests.
- `src/app.rs` — modifier tracking (`ModifiersChanged`), full keyboard commands, clipboard via `copypasta` (native Wayland through your window's display, X11 fallback, no-op stub) and press relay for drag selection.
- `src/ui/screen.rs` — mouse interaction policy (press-to-focus, drag-select, double/triple-click) plus rendering: selection rect, caret at its byte position, horizontal scroll.
- `src/todos.rs`, `ui/mod.rs`, `theme.rs`, `renderer.rs` — state plumbing, `Ui::click_in`, `COL_SELECTION`.

**Verified**: 22/22 tests pass, and I rendered headless frames (`TODO_DUMP_FRAME` + new `TODO_INPUT`/`TODO_INPUT_SELECT` debug overrides) — visually confirmed the selection highlight and the scrolled caret-at-end state. I also fixed a bug I caught in review: click hit-testing originally ignored the scroll offset, so clicks in scrolled long text landed at the wrong caret position.

One thing to know: dragging out of the field and releasing there unfocuses the input (standard click-away behavior). If you'd rather the field keep focus during an out-of-bounds drag-select, that's a small tweak — just say so.

# 2줄 나오게 업데이트
Done. The input field no longer scrolls text out the side — it now wraps. What changed:

**Behavior**
- Long text wraps onto a second line instead of sliding horizontally; the field grows downward (the Add button and the task list move down with it). Past two lines the extra text just runs off screen, as you said is fine.
- Wrapping breaks at spaces; an unbroken run longer than the line (a URL, long Hangul phrase) breaks at the character that overflows.
- The IME composition (Hangul preedit) is spliced in at the caret when wrapping, so it wraps together with the text and stays underlined and visible.
- Selection highlight spans both lines; clicking/dragging maps by (x, y) to the correct line and caret.
- New keys: **↑/↓** move between wrapped lines keeping the column; **Home/End** now go to the start/end of the visual line (same as before when everything fits one line).

**Code**
- `src/input.rs` — new `Wrap` type (line starts, `line_of`/`x_at`/`caret_at` mapping); replaced `scroll_x`/`x_from_byte`/`caret_from_x`/`visible_start`; added `move_up`/`move_down`/`move_to_line_start`/`move_to_line_end`.
- `src/ui/screen.rs` — the field block now computes the wrap per frame, sizes the field to `min(lines, 2)`, and draws selection/text/preedit/caret per line (`FIELD_MAX_LINES = 2`).
- `src/app.rs` — ArrowUp/ArrowDown keys wired up.

**Verified** — 28 unit tests pass (wrap breaks, boundary tie-breaks, Up/Down/Home/End, multibyte wrap), `cargo fmt` clean, and I dumped off-screen frames to check visually: two-line wrap, select-all covering both lines (this caught a real bug — the highlight collapsed to zero width at the wrap boundary, fixed with a line-aware `x_in_line` measurement), underlined Hangul preedit at the caret, big-font layout, and the single-line case unchanged. The two remaining clippy warnings (`app.rs:272`, `draw_settings_window` args) predate this change.

# 우선 순위(빨, 노, 회) 상단으로 정렬
Done — priority stripes are implemented, tested, and visually verified. Here's what changed:

## How it works

**Each row now has a colored stripe before its checkbox:**
- 🔴 **Red** = emergency → sorted to the very top
- 🟡 **Yellow** = next priority → after red
- ⚪ **Gray** = general → below yellow (the default for new tasks)

**Click the stripe to cycle a task's priority** gray → yellow → red → gray. The row jumps to its sorted position immediately (the thin stripe has a full row-height hit area, tinted on hover, so it's easy to click). This matches the app's existing mouse-driven list interaction (checkbox toggle, delete).

## Changes

| File                         | What                                                                                                                                                 |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/todos.rs`               | `Priority` enum (`Low`/`Mid`/`High`, default gray), `Todo.priority` field, `cycle_priority()`, stable `sort_by_priority()` applied on load/add/cycle |
| `src/todos.rs` (persistence) | Save lines are now `done<TAB>priority<TAB>text` (0=gray, 1=yellow, 2=red). **Old 2-column files still load** — they read back as gray                |
| `src/ui/theme.rs`            | `COL_PRIO_HIGH` (red), `COL_PRIO_MID` (yellow), `COL_PRIO_LOW` (gray)                                                                                |
| `src/ui/widgets.rs`          | `priority_button` — stripe + wide invisible hit rect with hover tint                                                                                 |
| `src/ui/screen.rs`           | Stripe drawn left of the checkbox (checkbox/text shifted right ~13px), `priority_color()` mapping, updated bottom hint                               |
| `src/app.rs`                 | Startup controls banner mentions the stripe                                                                                                          |

## Verification

- **31/31 unit tests pass**, including new ones: old/new save-line parsing, red→yellow→gray stable sort, and click-cycling re-sorting + persisting
- **Headless frame dump** with a mixed-priority list rendered exactly as specified — red rows (`URGENT server down`, `fix prod bug`) on top, yellow (`deploy Friday`, plus a done+struck-through one) next, gray last, and a legacy old-format line loading as gray. Checkbox/strikethrough/scrollbar all intact.

One design note: sorting is stable, so within the same priority, tasks keep their added order — and checked items stay in their priority group rather than sinking (let me know if you'd want done items pushed to the bottom too).
