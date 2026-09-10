# grep으로 작업하

```bash
$ grep -rn "SampleFormat\|sample_fmt\|ChannelLayout\|swresample\|AudioFrame" src/ --include="*.rs" | head -5; echo "===util structure==="; ls src/util/; grep -n "pub mod\|pub use" src/util/mod.rs | head -12; grep -n "FFMPEG_VERSION" FFmpeg/version.h FFmpeg/libavutil/ffversion.h 2>/dev/null | head -2
```



```bash
wc -l src/swresample/resample.rs src/swresample/mod.rs; cargo test --lib 2>&1 | grep "test result"; git show --stat f60e4b6 | head -10; grep -c "#\[test\]" src/…
```
