# 해외에서 시도한 ffmpeg프로젝트들
- 레딧글
  - https://www.reddit.com/r/rust/comments/1n0fbhv/has_anyone_worked_with_ffmpeg_and_rust/
- Safe, idiomatic, and performant Rust wrappers for FFmpeg, FFprobe, and FFplay
  - https://github.com/RustNSparks/ffmpeg-suite-rs

- Ffmpeg binding프로젝트
  - https://github.com/zmwangx/rust-ffmpeg-sys

### 완성도 높은 프로젝트
- rav1d is an AV1 cross-platform decoder, open-source, and focused on speed and correctness. It is a Rust port of dav1d.
  - https://github.com/memorysafety/rav1d
- Video stabilization using gyroscope data (https://gyroflow.xyz/)
  - https://github.com/gyroflow/gyroflow

# 나무위키 한글 정의

- https://namu.wiki/w/FFmpeg

## 영문 위키 정의(Eng.)

- https://en.wikipedia.org/wiki/FFmpeg

<hr />

# 내가 혼자 하는중.(GY) | 여기에 한달이상 투자해서 변환해보자(260905)
- https://github.com/YoungHaKim7/ffmpeg_rs

<hr />

```bash
✅ Phase 1 — CPU pipeline (this)
✅ Phase 2 — Vulkan swscale: headless compute (vulkano), port of vf_scale_vulkan.c + libswscale/vulkan/, bilinear/bicubic, -s

✅Phase 3 — filtergraph (libavfilter: buffersrc/sink, scale/format)
  ✅ ◼ Phase 3a — libswscale variable-width filters on the CPU engine: initFilter port (utils.c:197-612), area/gauss/sinc/lanczos/spline via -scale_algo with filter-width widening on downscale, CPU fallback for algorithms the Vulkan engine cannot run
  ✅ ◼ Phase 3b — filtergraph (libavfilter: buffersrc/sink, scale/format)

✅ Phase 4 — swresample + audio paths
  ✅ ◼ Phase 4a: audio foundations + swresample core (spec→implement→integrate)
  ✅ ◼ Phase 4b: WAV container + PCM codec + CLI + goldens

Phase 5 — NUT container, more filters

Pahse 6 - SIMD 추가

x winit은 완전 난중에 할꺼임 일단 보류
Stretch — winit player window on the Vulkan pipeline
```

<hr />

# `mux` fix

```bash
All four round trips pass:
  1. ours→NUT→ffmpeg: byte-identical (video)
  2. ffmpeg→NUT→ours: byte-identical (video, after the r_frame_rate fix)
  3. ffmpeg→NUT→ours (audio): payload-identical
  4. ours→NUT (audio)→ffmpeg: payload-identical

  NUT is fully cross-verified. Now add the golden tests, commit, then Zone D (filters). Let me write the goldens.

                                                                                                                                           10:14 AM glm-5.3
● All four cross-tool NUT round trips are byte/payload-identical. Adding the golden tests and committing:
```
- The y4m muxer derives F from the stream's time_base, not avg_frame_rate. The transcode sets out_st.set_pts_info(den, num) from avg — set_pts_info sets time_base
  = reduce(den/num)... For avg 10/1 → set_pts_info(1, 10) → time_base 1/10 → F = 10/1 ✓ in theory.

  But we got F25:1 — meaning in_st.avg_frame_rate was still 0/25 when transcode ran → default branch set_pts_info(1,25)? Line 345: if in_st.avg_frame_rate.num > 0
  { ... } else set_pts_info(1,25). So avg was still UNKNOWN/0 at transcode time.

  Why? My demux fix sets avg only when the tag parsed. Did the tag parse? Check: the demuxer's decode_info runs during read_header — yes info packets are read at
  header time (C reads them in the header scan). But WAIT — the info packet with r_frame_rate comes AFTER the main+stream headers; our read_header reads until
  first syncpoint. It should have parsed. Unless the stream ordering: stream_id_plus1 handling — for STREAM info, the tag uses stream_id_plus1 > 0. My check if
  stream_id_plus1 != 0 && name == "r_frame_rate" — right.

  Let me test the demux directly: read the header and print avg. Quick unit check via a debug run.

# Phase 5 is rolling. Standing state:

- Phase 5 is rolling. Standing state:

- Phase 5 — NUT container + more filters

- Zone A 🔄 (running, wf_583602b5-42b): the NUT demuxer — format/nut.rs carrying nut.c's shared framing core (vint/svint, CRC, packet-header helpers — made pub for the muxer zone) + nutdec.c's read path (main/stream headers, info, frame codes, syncpoints, decode_frame with exact pts/duration/flags, C error texts). Hand-built byte fixtures as tests.

- Queued (sequential, the proven pattern):
  - Zone B — NUT muxer (nutenc.c, 1258 lines) reusing Zone A's framing core
  - Zone C — -f nut CLI plumbing + goldens. Honest fidelity framing: byte-comparing two NUT muxes isn't a stable target (frame-code choice, index presence, header ordering are all free), so the goldens cross-check through the OTHER tool — ffmpeg muxes NUT → we demux → payload equal, and vice versa
  - Zone D — the pixel filters: crop (398), transpose (419), hflip (157) — real negotiation + geometry work on the Phase-3b filtergraph, one zone

- Suite verified green (all 10 targets) before launch; the filter trio was scoped at 974 lines of C — right for one implementer. I'll integrate Zone A when its notification lands.

- ※ recap: Porting FFmpeg to Rust phase by phase; Phases 1-4 (video pipeline, GPU scaling, filtergraph, audio) are done and green. Phase 5's NUT demuxer agent is building now; next I integrate it, then the NUT muxer. (disable recaps in /config)

# day8(`av_channel_layout_compare`)

- Confirmed: C's swr_init at line 223-225 — if (s->in_ch_layout.order == UNSPEC) { s->in_ch_layout = copy of used_ch_layout (defaulted native) }. So after init, s->in_ch_layout IS the native default (mono → FrontCenter), and config_changed compares it against the FRAME's layout... the frame from ffmpeg's decoder carries UNSPEC too?! Then C would also flag INPUT_CHANGED...

  NO — wait: C's av_channel_layout_compare treats UNSPEC-vs-native-with-same-count... av_channel_layout_compare: "if either is unspecified, compare only channel counts"? YES — av_channel_layout_compare (channel_layout.c) returns 0 (equal) when both have the same nb_channels and at least one is UNSPEC! That's the missing semantic: our ChannelLayout needs C's compare semantics (unspec-compatible compare) in config_changed — OR our PartialEq on ChannelLayout (derived) is stricter than av_channel_layout_compare.

  Check our channel_layout.rs for a compare (the spec ported subset/union — maybe not compare). Fix: implement av_channel_layout_compare semantics in config_changed (or add ChannelLayout::compare). C's rule (channel_layout.c:~700): orders equal & masks equal → 0; both custom...; if either unspecified → compare nb_channels only.


# day7(audio) 작업 오류

The WIP is the final 4b piece — the CLI audio path (235 lines in transcode.rs), missing one import. Fixing and testing:

  Thought for 10s, ran 1 shell command

354/354 with the audio CLI path compiled. Smoke-testing it end-to-end against system ffmpeg:

  Thought for 9s, ran 1 shell command

The audio path panics with a subtract overflow. Getting the backtrace:

  Thought for 25s, ran 2 shell commands

start − 4 underflows inside write_header — the fmt chunk's start is wrong. Reading the muxer's header sequence:

  Thought for 16s, searched ommand

start_tag looks right — neo see which call
underflows:

  Thought for 14s, ran 2 shell commands

tell() returns ~0 after 20 written bytes — pos isn't tracking writes on the file
path (tests used the mem h

  Thought for 20s, ran 1 she
                                                                                    write_all tracks pos correlues on the failing path:
                                                                                    Thought for 13s, ran 1 she
                                                                                    The probe run shows both enic vanish? Checking the
run and output:                                                                  
  Thought for 21s, ran 1 shell command                                             
Two WIP bugs surfaced: our output is stereo when the input was mono (defaults not inherited from input), andard. Removing the probeand reading the audio transcode path:

  Thought for 21s, ran 1 shell command                                          
The inheritance looks right in transcode — so the demuxer probably mis-parses channels. Checking both su


# day6

- ◯ impl-wav-demux-pcm  WAV demuxer + PCM decoder


- Phase 4 status

- 4a — libswresample: COMPLETE ✅ (307/307)
- fundamentals (samplefmt / channel_layout / audio_frame)
- audioconvert (36 kernels + DSD copy, mono planar normalization)
- rematrix (3318 lines — coefficient synthesis, 17.15 quantization, Dolby/DPLII, the upstream-bug replication)
- resample (Kaiser polyphase — bessel_i0, FilterBank, four element kernels, full context with hand-verified init anchors)
- SwrContext driver — salvaged complete-but-uncommitted from the usage-wall casualty: swr_init validation, the full swr_convert loop, convert_frame, 
  next_pts compensation modes, delay/flush, 32 tests including a 48k→44.1k→4roundtrip

- 4b — underway 🔄
- ✅ Shared base: CodecParameters audio fields + the PCM CodecId family (committed, 307/307)
- 🔄 Zone 1 agent: format/wav.rs (RIFF walk, ff_get_wav_header fmt-tag map incl. WAVEFORMATEXTENSIBLE masks, pcm-style packet slicing) + codec/pcm.rs (decoder + the AudioDecoder trait)
- Queued: zone 2 (WAV muxer + PCM encoder), zone 3 (CLI -i x.wav -ar/-ac/-sample_fmt + transcode loop + goldens vs system ffmpeg — mine, the shared-file work)

- The sequential single-agent pattern keeps holding — each zone lands green in its worktree, I cherry-pick, and casualties get salvaged from their uncommitted drafts (three for three recoveries now).


# day5 (Phase 4a)

- WAV entry points scoped for 4b (riffdec's ff_get_wav_header for fmt parsing, wavenc's header write, wavdec's chunk walk). Standing summary:

- Phase 4a — one zone remaining

- Zone: fundamentals (samplefmt / channel_layout / audio_frame)
- Status: ✅

────────────────────────────────────────

- Zone: audioconvert (36 kernels)
- Status: ✅
  
────────────────────────────────────────

- Zone: rematrix (3318 lines, Dolby/DPLII, upstream-bug replication)
- Status: ✅
────────────────────────────────────────

- Zone: resample (Kaiser polyphase, 1543 lines + 6 hand-verified pinning tests)
- Status: ✅ — 281 lib green at landing, current tree 275 lib + all integration
- targets green after your test-file reorganization

────────────────────────────────────────

- Zone: SwrContext driver
- Status: 🔄 agent working now (wf_2541e33e-117) — swr_init validation order, the
- swr_convert buffer choreography (in_convert → rematrix → resample → out_convert), delay/flush/next_pts, options

- The driver's spec was re-staged from the journal (third /tmp clear — the journals remain the durable spec store). Its prompt carries the cross-zone wiring notes the rematrix implementer left (the exact call sites at swresample.c:676/679), and its tests include the full-driver constant-upsample test the raw resample zone couldn't run.

- When it lands: integrate → full suite → Phase 4a complete → 4b (WAV demux/mux + PCM codec + -i x.wav CLI + goldens vs system ffmpeg).

# day4

Phase 4a progress


|                    Zone                    |            Status             |
|-|-|
| fundamentals (samplefmt + channel_layout +<br />audio_frame) | ✅ landed, C-verified         |
| audioconvert (36 kernels + DSD copy)       | ✅ landed, 8 test corrections<br />  verified against C |
| rematrix (3318 lines — matrix synthesis, <br />17.15 quantization, Dolby/DPLII, all 5<br /> template kernels)   | ✅ landed via single agent — <br />  |
| resample (Kaiser polyphase)                | 🔄 agent working (math  <br />pre-pinned from my own C  <br />read)     |
| SwrContext driver                          | queued (cached spec staged;<br /> re-extractable from the <br />journal)        |

- The sequential single-agent tactic is holding — rematrix came back build_ok with 32 tests including two spec-vs-C corrections the agent discovered and pinned. Once resample lands, the driver stitches the three engines together behind swr_convert, and Phase 4a closes for the WAV/PCM/CLI wave.


# day3

### Phase 3b — this session's progress

- The filtergraph is LIVE end-to-end:
- parser.rs (1640 lines, graphparser.c port) cherry-picked from its surviving worktree and wired: parse_ptr + the real ff_filter_opt_parse (replacing the naive splitter)
- -vf null is byte-identical to system ffmpeg through the full pipeline — demux → decode → buffersrc → parser-built graph → negotiation → buffersink → encode → mux
- Golden tests added and passing: -vf null and null,null,format=pix_fmts=yuv420p chains, byte-exact (10 golden total now)
- CLI edge cases verified: [in]null[out] labeled graphs work; nosuchfilter / null=x=5 / multi-chain shaped errors
- 207 lib + 3 integration + 10 golden tests green                           
In flight: the vf_scale implementer (workflow wf_6b12ae90-bfc, spec staged, worktree at current HEAD)delegates all pixel work to the existing ScaleContext and gates unported color-matrix conversions with honest Unsupported errors

Queued after it lands: registry + the auto-scale test flip (yuv420p→gray8 becomes a success path through the auto-inserted converter), -vf scale= golden tests with the established tolerances, then the final 5-area adversarial verify workflow (buffersrc, buffersink, driver, parser, vf_scale — vf_format already reviewed clean) and any fixes it confirms.

The pattern from the usage-limit casualties is now handled: specs are recovered to disk, implementers commit in their worktrees, and each landed module gets integrated + verified before the next step. I'll pick up the moment the workflow notifies.

✻ Sautéed for 3m 36s · done 4:33 AM(260907)

  2 tasks (0 done, 2 open)
  ◻ Land vf_scale: cherry-pick, registry, test flips, scale goldens
  ◻ Final verify: 5-area adversarial review + fixes

<hr />

# day2

The identity-size conversion path changed semantics (bicubic + CENTER-siting chroma interpolation vs Phase 1's nearest/LEFT). A diff of 238 is too big for siting though — let me look at actual pixels:


<hr />

Phase 1 is complete. Here's the summary:

## What was built — ffmpeg_rs Phase 1 (CPU foundation)

Per your direction, the conversion is scoped as a pipeline-faithful port delivered in bounded phases over the month. This session landed the complete working pipeline:

**`src/util`** (libavutil) — `Rational` with the exact `av_reduce` continued-fraction walk, `av_rescale_q` family with C's rounding/PASS_MINMAX semantics, error enum (`Err(Eof)`/`Err(Again)` as values, like C's sentinels), av_log, 23-format `PixelFormat` + descriptor table transcribed from `pixdesc.c`, imgutils plane geometry, and `Frame` with Arc-backed planes (writable-iff-one-ref, zero-copy packet wrap).

**`src/codec`** (libavcodec) — `Packet`, `CodecParameters`, Decoder/Encoder send/receive traits, rawvideo decoder (zero-copy) + encoder, with every skipped C path documented against its unreachable guard.

**`src/format`** (libavformat) — buffered `IoContext` with aviobuf's EOF-ordering subtleties, Y4M demuxer/muxer (including the synthetic-space `444`/`444alpha` trick and guard rows against prefix mis-parses), rawvideo demuxer + raw muxer.

**`src/swscale`** — `ScaleContext` with identity + BT.601 yuv420p/gray8→packed-RGB kernels (Vulkan replaces the kernels in Phase 2 behind the same API).

**`src/fftools`** — the CLI: `-i/-f/-pix_fmt/-pixel_format/-video_size/-framerate/-y/-v`, ffmpeg-shaped banner, `Input #0`/`Output #0` dumps, `frame= … Lsize= …` summary.

## Verification — `cargo test`, 81/81 green, zero warnings

The golden tests run against your system ffmpeg 8.1.2:
- **y4m → rawvideo (yuv420p): byte-exact**
- **rawvideo → y4m: byte-exact** (header writer pinned, including `C420jpeg XYSCSS=420JPEG`)
- y4m → rgb24: max byte diff 3, mean 0.66 (within the planned ±3 vs swscale's integer tables)
- y4m → y4m through our own muxer+demuxer: byte-exact vs ffmpeg's original
- Stream-dump lines match ffmpeg's byte-shape (the tests caught and I fixed a real comma-vs-space fidelity bug)

Demo: `cargo run -- -i in.y4m -f rawvideo -pix_fmt rgb24 out.raw -y`

## Next

**Phase 2 (next session)**: the Vulkan heart — a headless compute context (the `HeadlessGpu` pattern from your Navier-Stokes project, already pinned in the roadmap notes), a port of `vf_scale_vulkan.c` + `scale.comp.glsl`, bilinear/bicubic resampling, and the `-s` flag. The roadmap, conventions, and verification setup are saved in the plan file and memory, so any future session can pick up exactly here. Nothing is committed yet — say the word if you want a Phase 1 commit.
