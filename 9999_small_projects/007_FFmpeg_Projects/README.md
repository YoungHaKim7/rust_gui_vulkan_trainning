# 여기에 한달이상 투자해서 변환해보자(260905)
- https://github.com/YoungHaKim7/ffmpeg_rs

<hr />

```bash
✅ Phase 1 — CPU pipeline (this)
Phase 2 — Vulkan swscale: headless compute (vulkano), port of vf_scale_vulkan.c + libswscale/vulkan/, bilinear/bicubic, -s
Phase 3 — filtergraph (libavfilter: buffersrc/sink, scale/format)
Phase 4 — swresample + audio paths
Phase 5 — NUT container, more filters
Stretch — winit player window on the Vulkan pipeline
```

<hr />

# day3

Phase 3b — this session's progress

The filtergraph is LIVE end-to-end:
- parser.rs (1640 lines, graphparser.c port) cherry-picked from its surviving worktree and wired: parse_ptr + the real ff_filter_opt_parse (replacing the naive splitter)
- -vf null is byte-identical to system ffmpeg through the full pipeline — demux → decode → buffersrc → parser-built graph → negotiation → buffersink → encode → mux
- Golden tests added and passing: -vf null and null,null,format=pix_fmts=yuv420p chains, byte-exact (10 golden total now)
- CLI edge cases verified: [in]null[out] labeled graphs work; nosuchfilter / null=x=5 / multi-chain shaped errors
- 207 lib + 3 integration + 10 golden tests green                           
In flight: the vf_scale implementer (workflow wf_6b12ae90-bfc, spec staged, worktree at current HEAD)delegates all pixel work to the existing ScaleContext and gates unported color-matrix conversions with honest Unsupported errors

Queued after it lands: reflip (yuv420p→gray8 becomes a success path through the auto-inserted converter), -vf scale= golden tests with the established tole adversarial verifyworkflow (buffersrc, buffersink, driver, parser, vf_scale — vf_format already reviewed clean) and any f

The pattern from the usagdled: specs are recoveredto disk, implementers commit in their worktrees, and each landed module gets integrated + verified befup the moment the workflownotifies.

✻ Sautéed for 3m 36s · done 4:33 AM(260907)


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
