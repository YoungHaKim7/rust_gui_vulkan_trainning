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

- Prediction templates are the spec formulas. Now the reconstruction ordering (h264_mb.c) and IDCT:

- All DSP surveyed. Grab the last three C pieces (nnz prediction, intra mode checks, pred_intra_mode):

```bash
  Running 2 shell commands…
  ⎿  $ sed -n '42,60p' FFmpeg/libavcodec/h264_mvpred.h; sed -n '134,235p' FFmpeg/libavcodec/h264_parse.c
```


# 260923(07:01pm)

- The extractor's row-regex failed on the nested layout. Rewrite it to slice flat arrays by dimension:

- Found it — chroma_dc_coeff_token_len contains the search string as a suffix. Anchor with a word boundary:

- The C tables are triangular with implicit zero padding. Parse per-row and pad:

- 15 rows present, 16 declared — pad missing trailing rows too:

- The intra table uses enum names + arithmetic — generate it programmatically to match C's values:

```bash
  1 tasks (0 done, 1 in progress, 0 open)
  ◼ H.264 survey + module design
```
