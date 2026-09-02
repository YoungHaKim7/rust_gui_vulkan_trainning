All three warnings are pre-existing (unrelated lines — `app.rs:294`, `font.rs:196`, `screen.rs:538`), not from this change. Leaving them out of the diff.

Done. Six new font steps added — **11 levels total**, spanning ±10 px around the default:

|       | level 0 | 1   | 2   | 3   | 4   | **5 (default)** | 6   | 7   | 8   | 9   | 10    |
| ----- | ------- | --- | --- | --- | --- | --------------- | --- | --- | --- | --- | ----- |
| text  | 10      | 12  | 14  | 16  | 18  | **20**          | 22  | 24  | 26  | 28  | 30 px |
| title | 16      | 19  | 22  | 26  | 29  | **32**          | 35  | 38  | 41  | 44  | 47 px |

The old five sizes keep their exact pixel values (now at levels 3–5, 7, 9), and title stays ~1.6× text.

Changes:
- `src/font.rs` — `LEVELS = 11`, `DEFAULT_LEVEL = 5`, extended `PX` table.
- `src/ui/screen.rs:26` — `LAYOUT_SCALE` extended by the same progression (0.07/step below default, 0.08/step above), so paddings/buttons scale smoothly across the new range.
- `src/settings.rs` — settings files now carry a `v=2` marker, and old 5-step files are migrated on load by pixel size, so your saved `font=4` loads as level 9 (**28 px, exactly what you have now**) instead of dropping to 18 px. Your live `settings.txt` upgrades itself on the first run.
- `src/font.rs` tests — the solid-ink assertions now only require full coverage at ≥16 px, where strokes are at least a pixel wide (measured: 'M' peaks at 202–237 below that); smaller sizes still must reach ≥100 coverage.

Verified: 38 tests pass (incl. a new migration test), fmt clean, and headless frame dumps confirm the migrated 28 px setting plus both new extremes render with nothing clipped. The live `settings.txt`/`todos.txt` were never touched — all visual runs used copies in `/tmp`.

# 조금더 다양하게 폰트 크기 조

That `settings.txt` diff is from your own runs of the v2 binary (font=10, new window size) — my runs never wrote to it. With this change your 30 px setting loads as level 14, same size.

Done — **20 font steps**, spanning ±20 px around the default:

|       | level 0 | …   | 4   | …   | **9 (default)** | …   | 13  | …   | 19    |
| ----- | ------- | --- | --- | --- | --------------- | --- | --- | --- | ----- |
| text  | 2       | …   | 10  | …   | **20**          | …   | 28  | …   | 40 px |
| title | 3       | …   | 16  | …   | **32**          | …   | 44  | …   | 62 px |

(2 px steps throughout; the bottom end is 2 px since 0 px can't rasterize. All previous sizes — 10–30 px — keep their exact values at levels 4–14.)

Changes:
- `src/font.rs` — `LEVELS = 20`, `DEFAULT_LEVEL = 9`, extended `PX` tables (title stays ~1.6× text).
- `src/ui/screen.rs:26` — `LAYOUT_SCALE` now 0.37 → 1.80, extending the same +0.07/+0.08 per-step progression.
- `src/settings.rs` — format bumped to `v=3`; loads migrate **both** older formats by pixel size: v1 (five steps) via table, v2 (eleven steps) via +4 offset. Tests cover both, including out-of-range clamping.
- `src/font.rs` tests — measured ink coverage at the new tiny sizes ('가' peaks at 44/255 at 2 px) and lowered the sub-16 px faint-ink floor from 100 to 40; solid-ink checks unchanged at ≥16 px.
- `src/ui/screen.rs:474` — the footer hint now clips short of the "Clear completed" button (it already ran underneath at ≥24 px in narrow windows; at 40 px it collided in any window).

Verified: 39 tests pass, fmt clean, clippy shows only the 3 pre-existing warnings, and headless dumps confirm the v2→v3 migration (30 px preserved), level 0 and level 19 render with nothing clipped at the window edges, and the hint/button overlap is gone.
