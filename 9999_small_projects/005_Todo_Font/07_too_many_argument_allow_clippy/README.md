All green:

- **`src/app.rs:309`** — replaced the redundant guard with a pattern match: `button: MouseButton::Left, ..`
- **`src/font.rs:231`** — collapsed the nested `if let` into a let-chain (`if let … && let …`)
- **`src/ui/screen.rs`** — added `#[allow(clippy::too_many_arguments)]` on `draw_ui` (8 args), `draw_quit_dialog` (10), and `draw_settings_window` (9)

```
cargo clippy --all-features --all-targets -- -D warnings  → Finished, no warnings
cargo test --all-features                                 → 41 passed, 0 failed
```
