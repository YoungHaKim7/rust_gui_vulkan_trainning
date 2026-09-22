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

# AAC-LC is done and landed. Final state: commit 871c420, 578/578 tests green, memory updated.

- What the session delivered

- The AAC decoder had been committed broken (26 compile errors, zero tests). This session took it from there to matching system ffmpeg to ±1 LSB on mono and stereo, end-to-end through the real CLI (-i x.aac → wav: max_diff 0.0000, over=/0 on the f32 smoke; ±1 LSB in s16).

The hunt — the MP3-era recipe worked again, with a new twist: every component verified clean while the composition failed, so the bug had to be in a mistranslated bridge:
1. Bitstream layer proven bit-exact — an independent python decoder (sections → scalefactors → huffman → VMUL/escapes) of frame 5 matched our Rust coeffs to the last bit, exonerating everything upstream
2. IMDCT proven exact against av_tx vectors compiled against brew's libavutil (with a detour: a "formula is wrong at L=128" scare turned out to be my python LCG seed missing a 32-bit mask)
3. ret_buf ≠ emitted frame ⇒ plumbing; stage dumps (COEFFS/BUF/RETBUF) isolated the divergence to the windowing step
4. Root cause: vector_fmul_window — C's dst += len; win += len; src0 += len with the (i<0, j≥0) loop writes both output halves [0..2n) and reads the window's second half (win[2n−1−k]). The port wrote n mirrored outputs with first-half windows — every frame's second half was stale garbage with the right energy. Pinned with a compiled-C harness (/tmp/fmw*.c) before fixing, not by re-reading.

Also fixed en route: TNS ramp (FFMIN(m, order) per filter run), GET_GAIN = powf(scale, −gain) with C's gain += t ordering, CPE output slots carrying explicit subchannel indices (L/R swap in mixed layouts), the ADTS demuxer's num_rdb bits and codecpar.sample_fmt.

Scope delivered with it: ADTS demuxer (format/aac.rs), shared format/id3.rs, CodecId::Aac, CLI dispatch, 5 decoder tests (VLC roundtrips, av_tx-verified IMDCT, synthetic silent frame, real-file smoke) + 5 demuxer tests.

- Next on the ladder: H.264 (decoder + the big one — then MP4 mux/demux makes it reachable from the CLI).
