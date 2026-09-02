# Color test(CMYK(0.13, 0.13,0.00,0.31))
- https://convertingcolors.com/cmyk-color-0.13_0.13_0.00_0.31.html

```rust
● Update(src/ui/theme.rs)
  ⎿  Added 1 line, removed 1 line
      1  //! Color palette and button styles.
      2
      3 -pub(crate) const COL_BG: [f32; 4] = [0.055, 0.06, 0.078, 1.0];
      3 +pub(crate) const COL_BG: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
      4  pub(crate) const COL_ROW_ALT: [f32; 4] = [0.085, 0.09, 0.115, 1.0];
      5  pub(crate) const COL_PANEL: [f32; 4] = [0.13, 0.135, 0.17, 1.0];
      6  pub(crate) const COL_PANEL_HOVER: [f32; 4] = [0.18, 0.19, 0.235, 1.0];
```

The background color is defined in `src/ui/theme.rs:3` — the `COL_BG` constant. The renderer uses it as the Vulkan clear color (`src/renderer.rs:676`).

I changed it to black for you:

```rust
// before: dark navy
pub(crate) const COL_BG: [f32; 4] = [0.055, 0.06, 0.078, 1.0];
// now: pure black
pub(crate) const COL_BG: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
```

The format is `[red, green, blue, alpha]` as floats 0.0–1.0. Build passes.

All the other colors live in the same file too — worth knowing since pure black makes the layered surfaces stand out more:
- `COL_ROW_ALT` (line 4) — alternating row stripes
- `COL_PANEL` / `COL_FIELD` — buttons and input field backgrounds
- `COL_BORDER` — outlines

Run it with `cargo run` to see how it looks.
