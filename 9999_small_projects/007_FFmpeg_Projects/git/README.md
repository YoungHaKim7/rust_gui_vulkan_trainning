```bash
git show --stat b5e61c4 | head -8; git status --short | grep -v "^??" | head -3; cargo test --lib 2>&1 | grep "test result"
```

- git log 후 fail테스트 찾

```bash
● Bash(git log --oneline -2; git status --short | grep -v "^??" | head -3; grep -c "MSB-first" src/codec/audio/mp3.rs 2>/dev/null; grep -c
      "0xFFFE2000" src/codec/audio…)
  ⎿  2031354 audio mp3 + test fail
     357d144 move util/samplefmt test file
     2
     … +2 lines (ctrl+o to expand)

  Thought for 9s (ctrl+o to expand)

● Bash(grep -n "0xFFF72000\|0xFFF59000\|mod dbg_probe" src/codec/audio/mp3.rs | head -6; cargo test --lib -- codec::audio::mp3 2>&1 | grep -E
      "FAILED|test result" | he…)

```


# git diff

```
git diff --no-index src/codec/audio/mp3.rs /tmp/mp3_fix_partial.rs 2>/dev/null | grep -E "^\+" | grep -vE "^\+\+\+" | grep -E "fn |// |DBG|dump|imdct|synth|window|enwindow" | head -20


git diff --no-index src/codec/audio/mp3.rs /tmp/mp3_fix_partial.rs 2>/dev/null | grep -E "^[+-]" | grep -vE "^[+-]{3}" | grep -vE "^[+-]\s*//" | grep -vE "^[+-]\s*///" | grep -vE "^[+-]\s*$" | head -20)
-static TABLES: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();
-const EXP2_LUT: [f64; 4] = [
-    1.00000000000000000000,   // 2 ^ (0 * 0.25)
-    1.18920711500272106672,   // 2 ^ (1 * 0.25)
-    std::f64::consts::SQRT_2, // 2 ^ (2 * 0.25)
-    1.68179283050742908606,   // 2 ^ (3 * 0.25)
-];
-const I12_C3: f32 = fx(0.86602540378443864676 / 2.0);
-const I12_C4: f32 = fx(0.70710678118654752439 / 2.0); // 0.5 / cos(pi*(9)/36)
-const I12_C5: f32 = fx(0.51763809020504152469 / 2.0); // 0.5 / cos(pi*(5)/36)
-const I12_C6: f32 = fx(1.93185165257813657349 / 4.0); // 0.5 / cos(pi*(15)/36)
-const C1: f32 = fx(0.98480775301220805936 / 2.0);
-const C2: f32 = fx(0.93969262078590838405 / 2.0);
-const C3: f32 = fx(0.86602540378443864676 / 2.0);
-const C4: f32 = fx(0.76604444311897803520 / 2.0);
-const C5: f32 = fx(0.64278760968653932632 / 2.0);
-#[allow(dead_code)] // defined by C's cos(pi*i/18) set; unused there too
-const C6: f32 = fx(0.5 / 2.0);
-const C7: f32 = fx(0.34202014332566873304 / 2.0);
-const C8: f32 = fx(0.17364817766693034885 / 2.0);
```
