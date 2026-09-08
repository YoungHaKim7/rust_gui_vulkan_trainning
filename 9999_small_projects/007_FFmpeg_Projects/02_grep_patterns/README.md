# grep으로 작업하

```bash
$ grep -rn "SampleFormat\|sample_fmt\|ChannelLayout\|swresample\|AudioFrame" src/ --include="*.rs" | head -5; echo "===util structure==="; ls src/util/; grep -n "pub mod\|pub use" src/util/mod.rs | head -12; grep -n "FFMPEG_VERSION" FFmpeg/version.h FFmpeg/libavutil/ffversion.h 2>/dev/null | head -2
```
