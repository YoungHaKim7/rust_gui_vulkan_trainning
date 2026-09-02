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
