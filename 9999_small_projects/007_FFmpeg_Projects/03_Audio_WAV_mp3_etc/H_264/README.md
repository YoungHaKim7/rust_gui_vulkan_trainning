# H.264
- H.264 — the big one. Same discipline: survey → bound the scope → layers with tests → real-file acceptance. The C decoder is ~15k lines across a dozen files, so this phase gets bounded like AAC-LC was: Baseline profile, CAVLC, I/P slices, loop filter gated (acceptance vs ffmpeg -skip_loop_filter), CABAC/B-frames/weighted-pred gated Unsupported. Survey first:

- Video Decoder trait plumbing exists (Frame output, rawvideo precedent). Fixtures first — baseline all-intra for the first milestone:

- Fixtures ready (Constrained Baseline, 8×6 MBs, 20 frames, with/without-loop-filter references). Now the C survey — SPS/PPS parse first:
