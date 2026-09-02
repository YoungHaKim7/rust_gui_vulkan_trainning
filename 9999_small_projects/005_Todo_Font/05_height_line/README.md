Done. Wrapped-line spacing now hugs the text — at your font level (30px text), the gap between lines went from **9.0px to 0.0px (100% reduction)**; pitch 30px → 21px.

**Root cause:** the line pitch was Hack Nerd Font's nominal line box (`ascent − descent + line_gap`), which is far taller than the Hangul ink your wrapped lines actually draw (~70% of it at your size) — every wrapped line carried ~9px of dead whitespace.

**Fix** (`src/font.rs`, `src/atlas.rs`, `src/ui/mod.rs`, `src/ui/screen.rs`): line metrics are now measured from the rasterized ink itself:
- `line_height` = the body script's ink — the `가` raster from the Noto Serif KR fallback when it exists (you type Korean), Latin letters/digits otherwise — so consecutive lines touch exactly; capped at the old box only at degenerate tiny sizes.
- `ascent` (baseline anchor) = ink top; `descent` = shallowest ink bottom, and the IME composition underline now anchors just under the ink since the box no longer has slack below the glyphs.

**Verification:**
- New end-to-end test `wrapped_field_lines_nearly_touch` draws a real frame via `draw_ui` and measures the drawn glyph quads in the vertex stream: `gap=0.00 old_gap=9.00 reduction=100.0%`. All 40 tests pass, no warnings, `cargo fmt` clean.
- Screenshot of the running app confirms the layout is intact (title/field/rows fine, just tighter).
- Synthetic clicks turned out to be blocked entirely by GNOME (XTEST and `XSendEvent` both silently dropped), which is why verification went through vertex-level tests instead — noted in memory for next time.

One trade-off to know about: Latin ink is a few px taller than Hangul, so a long *pure-Latin* wrapped input could have descenders just kiss the line above. Single-line text (buttons, rows, settings) is unaffected. If you'd like a hair of breathing room back, it's a one-line pad constant away.
