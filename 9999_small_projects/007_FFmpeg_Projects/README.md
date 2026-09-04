# 여기에 한달이상 투자해서 변환해보자(260905)
- https://github.com/YoungHaKim7/ffmpeg_rs

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
