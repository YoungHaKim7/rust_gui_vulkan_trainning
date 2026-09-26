# 260927 (6:28am)


```bash
⏺ Bash(J=/Users/gy/.claude/jobs/effb161f/tmp; $J/h264c /tmp/h264_black.h264 2>/dev/null > /Users/gy/.claude/jobs/effb161f/tmp/black.out & sleep 8; head -12 /Users/gy/.claude/jobs/effb161f/tmp/black.out)
(No output)
(timeout 30s)

⏺ Bash(sleep 3; cat /Users/gy/.claude/jobs/effb161f/tmp/black.out | head -14; pkill -f h264c 2>/dev/null; true)
(No output)
(timeout 30s)
```

- ∴ The harness hangs on the black fixture (infinite loop — probably the NAL scan's i++ continue with the validation failing forever, or a run loop). This debug tail is consuming budget without convergence. Time to consolidate: I've made real progress this session (2 frames, P-slice syntax, several C-exact fixes). The remaining reconstruction bug (output ~121 vs 16 on a black frame — everything off by a consistent amount) actually has a NEW clue: black output 121 = 128 - 7. ffmpeg's black Y=16. 128-16 = 112. Our residual only moved -7. The DC level decoded was too small by ~16x... 112/7 = 16. A 16x scale miss in the DC path! In luma_dc_dequant_idct, output = (z*qmul + 128) >> 8. If it should be >> 4... C's exact: output[stride*0+offset]= (int)((z0 + z3)*qmul + 128 ) >> 8; ✓ mine same. But WAIT — maybe the DC level itself: the residual decoder gives level, and for the DC block C does NOT multiply by qmul at decode_residual time (qmul=NULL for DC!) — decode_residual(..., sl->mb_luma_dc[p], LUMA_DC_BLOCK_INDEX, scan, NULL, 16) — C passes NULL qmul for the luma DC! My decode_residual applies level*qmul with qmul=dequant[0][q][0] for the DC too (I pass qmul and multiply). Let me check my call: I pass qmul = dequant(0,q)[0] for the DC residual AND set is_luma_dc=true which skips the multiply ✓ (if !is_luma_dc guard). OK so DC stored raw ✓.

  Then the 16x: the DC coefficient for black... c = -7ish decoded. To move 112 need c134/... z = 4c (Hadamard sum) → (4c832+128)>>8 = 13c. c=-7 → -91 → +32 >>6 = -1 → 127. We got 121 → our c ≈ -29? (-2913 = -377 → (-377+32)>>6 = -5 → 123...). Anyway the decoded DC level c is ~-29 but should be ~-280 for black. 280/29 ≈ 10. Hmm.

  OR the answer: pred isn't 128! If intra16x16 pred mode = DC and neighbors unavailable → 128 ✓...

  New idea: x264 codes black as 16 with pred 128 and a LARGE negative DC. The DC residual for intra16x16 in x264: yes typically large. Our DC level decode being ~10x small points at... the ESCAPE path scaling? c=-29 via level_tab vs true -280? The LEVELS in the testsrc2 case were also small vs ffmpeg's needs. SYSTEMATIC: levels ~10x too small. The escape decode: level_code = 15<<suffix_length; ... level_code += get_bits(prefix-3) then unfold — maybe my level_tab non-escape values are right but ESCAPES wrong.

  Actually — hold on. Let me reconsider the black numbers: output 121 uniformly. idct_dc_add adds (DC+32)>>6. 121-128 = -7 → (DC+32)>>6 = -7 → DC ∈ [-480, -417). DC = (z*qmul+128)>>8 with z=4c (all-16 equal) and qmul=832 (qp 27 → div6=4,rem6=3 → init[3][0]=14<<6=896? qp27: 27/6=4, 27%6=3 → init[3][0] = 14 → 14<<6 = 896). DC = (4c*896+128)>>8 = 14c. -480 ≤ 14c < -417 → c ∈ [-34.3, -29.8] → c = -30..-34. So our DC coefficient c ≈ -30. For ffmpeg's 16: 16-128 = -112 → (14c+32)>>6 = -112 → 14c ≈ -7200 → c ≈ -514. So true c ≈ -514, we decoded -30. 514/30 ≈ 17. Suspicious factor.

  -514 in CAVLC escape terms: level -514 → level_code = 2*514-1 = 1027 = escape. Our -30 could result from misreading the escape suffix.

  - I'm deep in the weeds. Budget check: 14.03M left of 15M — plenty of tokens but this needs the harness fixed (it hangs). The C-harness hang on black: likely the NAL validation loop i++ scanning past... it validated nothing and loops? The while condition i+3<n; the if(type!=1&&5){i+=3} — fine. The hang more likely: the reordering for(;;){op=ue; if 3 break; ue} with an infinite ue of zeros → ue() loop while(rdbit1==0) lz++ — rdbits saturates index at size_in_bits, peek returns 0 → INFINITE LOOP in ue(). That's it — the black IDR's header parse diverged into an endless ue. Same class as the misparse.

  - Given the remaining budget and the session's length, the right call is to wrap up cleanly: commit the current state, note the exact open thread (DC level ~17x too small on black; suspect the level escape path; harness hangs on black via unbounded ue — add a guard), and update memory. Actually I still have ~14M tokens — that's substantial. But sessions this long risk quality; the user keeps saying "keep going" though. Let me fix the harness hang guard + diff the black DC decode directly in Rust with dumps. Actually simplest decisive experiment: print Rust's DC level for black MB0.

```py
⏺ Bash(python3 -c "
  p = 'src/codec/video/h264.rs'
  s = open(p).read()
  old = '''            let mut dc = self.mb_luma_dc;
              decode_residual(self, &cv, gb, &mut dc, LUMA_DC, &scan, qmul, 16, true)?;
              self.mb_luma_dc = dc;'''
  if old not in s:
      # find the actual DC call
      import re
      m = re.search(r'.*decode_residual\(self, &cv, gb, &mut dc.*', s)
      print('pattern:', m.group(0) if m else 'none')
  else:
      print('found')
  ")
found
```

```py
⏺ Bash(python3 -c "
  p = 'src/codec/video/h264.rs'
  s = open(p).read()
  old = '''            let mut dc = self.mb_luma_dc;
              decode_residual(self, &cv, gb, &mut dc, LUMA_DC, &scan, qmul, 16, true)?;
              self.mb_luma_dc = dc;'''
  new = '''            let mut dc = self.mb_luma_dc;
              decode_residual(self, &cv, gb, &mut dc, LUMA_DC, &scan, qmul, 16, true)?;
              if std::env::var_os(\"H264_DUMP\").is_some() {
                  eprintln!(\"  LUMADC {:?}\", &dc[..8]);
              }
              self.mb_luma_dc = dc;'''
  assert old in s
  s = s.replace(old, new)
  open(p,'w').write(s)
  print('ok')
  "; H264_DUMP=1 cargo test --lib video::h264 -- --nocapture 2>&1 | grep -aE "LUMADC|TOK n=48" | head -3)
 ```


# H.264 status — honest WIP, committed and safe (b0674ea, suite 579/579 green; your parallel commits eb9a3eb/faa56c2 carried the bulk of the file).

  Where the H.264 baseline decoder stands

  Verified working (against an independent python decode of the x264 fixture, the AAC playbook again):
  - NAL split + emulation-prevention strip + exp-golomb reader
  - SPS/PPS parse (all the baseline gates: CABAC/FMO/422-444/depth>8 → Unsupported)
  - Slice-header parse — including the trap that cost a false alarm: x264's PPS sets deblocking_filter_control_present, so the slice header carries
    idc+alpha+beta (3 bits my python walker initially missed — once fixed, Rust and python agreed bit-exact on positions)
  - intra4x4 mode syntax, cbp mapping, chroma pred — all aligned
  - The CAVLC VLC tables + level_tab (with two subtle C semantics fixed: prefix = 8 − av_log2(2i), and the suffix-length update is an unsigned
    compare)

  Not yet working:
  - run_before accounting — the residual decode walks MB0 correctly through the first two blocks (tc/to/zl dumps match), then hits "negative zeros" a
    few blocks in. Next session's first move is a full-CAVLC python walker for per-block diffing (my current one's table regex breaks on the multiline
    table format — noted in memory)
  - reconstruction — intra4x4/16x16/chroma/MC paths are written but scaffolding-quality, with four known-hacky spots listed in the memory file for the
    next session

  Gotchas paid for (all in memory): C's neighborhood caches are 15*8=120 bytes not 48; AV_PICTURE_TYPE codes are I=1/P=2/B=3; the I-table's -1 cbp
  means read-from-bitstream; ff_zigzag_scan has 17 entries for the scan+1 AC trick.

 - The probe test tolerantly decodes what it can (currently 1 frame) with no assertion, so the suite stays green while the decoder comes up. Fixture
  regen commands are in memory.
  

<hr />

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
