# H.264
- H.264 — the big one. Same discipline: survey → bound the scope → layers with tests → real-file acceptance. The C decoder is ~15k lines across a dozen files, so this phase gets bounded like AAC-LC was: Baseline profile, CAVLC, I/P slices, loop filter gated (acceptance vs ffmpeg -skip_loop_filter), CABAC/B-frames/weighted-pred gated Unsupported. Survey first:

- Video Decoder trait plumbing exists (Frame output, rawvideo precedent). Fixtures first — baseline all-intra for the first milestone:

- Fixtures ready (Constrained Baseline, 8×6 MBs, 20 frames, with/without-loop-filter references). Now the C survey — SPS/PPS parse first:

- Now the slice header parse (the baseline/IDR path):

- Now the CAVLC core:

- CAVLC MB layer mapped. Now the remaining core: dequant init, intra prediction, MB reconstruction:

```bash
⏺ Running 3 shell commands…
  ⎿  $ sed -n '600,698p' FFmpeg/libavcodec/h264_ps.c
```

- Prediction templates are the spec formulas. Now the reconstruction ordering (h264_mb.c) and IDCT:

```bash
⏺ Running 3 shell commands…
  ⎿  $ sed -n '/hl_decode_mb_internal/,/^}/p' FFmpeg/libavcodec/h264_mb_template.c | head -110; echo ====; sed -n '/static void FUNCC(ff_h264_luma_dc_dequant_idct)/,/^}/p' FFmpeg/libavcodec/h264idct.c | head -55
```
