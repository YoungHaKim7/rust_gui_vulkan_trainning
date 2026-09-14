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
