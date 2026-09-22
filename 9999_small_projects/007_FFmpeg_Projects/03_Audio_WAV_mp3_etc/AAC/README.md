# 🎯 THE AAC DECODER IS CORRECT

- Mono: maxdiff = 1 LSB, zero samples over tolerance — matching system ffmpeg across the whole file. The bug was vector_fmul_window's C pointer arithmetic mistranslated (both the dst[j] second-half destination and the win[j] second-half window index). Run everything:

```bash
⏺ Bash(./target/debug/ffmpeg_rs -y -i /tmp/aac_stereo.aac /tmp/aac_stereo_our.wav 2>&1 | tail -1; python3 - <<'EOF'
      import struct…)
  ⎿  frame=  131 fps=82.7 q=-0.0 Lsize=     524KiB time=00:00:03.04 bitrate=1411.2kbits/s speed=2x
     stereo: maxdiff=1 samples>0.02: 0 of 254874
     test codec::audio::aac::tests::imdct_matches_avtx_vectors ... ok
     … +5 lines (ctrl+o to expand)
  ⎿  (timeout 5m)
  ⎿  Allowed by auto mode classifier

  Thought for 11s (ctrl+o to expand)

⏺ Stereo: maxdiff = 1 LSB, zero over tolerance. All 5 tests green. Clean up the debug scaffolding, format, full suite:

⏺ Bash(python3 - <<'EOF'
      p = 'src/codec/audio/aac.rs'…)
  ⎿  cleaned
     0

```
