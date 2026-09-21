```
MP3 phase bug fix — gapless in the real pipeline

The last session's commit left the gapless trim as a test-side hack (lame_gapless_trim lived inside the smoke test) plus a per-packet Info/Xing sniff stand-in. So -i x.mp3 output
still carried the encoder-delay garbage head/tail. Now it's ffmpeg-faithful, layer by  layer:                                                                                     
                                                                                       FFmpeg C: mp3_parse_vbr_tags/info_tag/vbri_tag (mp3dec.c:157-441)                          
Port: Mp3Demuxer::parse_vbr_tags — runs at read_header off the post-ID3 frame (C's ordetag parse before junk-skip), skips the tag frame, exports gapless/duration/VBR bitrate;    
replaces the sniff hack                                                                ────────────────────────────────────────                                                   
FFmpeg C: FFStream.start_skip_samples / first/last_discard_sample                      Port: New Stream fields                                                                    
────────────────────────────────────────                                               
FFmpeg C: demux.c:1536-1557 side-data injection
Port: InputFormatContext::read_frame → Packet::skip_samples (pts==0 packet) /
discard_padding (packet crossing first_discard_sample)
────────────────────────────────────────
FFmpeg C: decode.c trim
Port: AudioFrame::crop + skip carry (skip_left, whole-frame drop when skip spans) in
Mp3Decoder::send_packet
────────────────────────────────────────
FFmpeg C: st->start_time timeline shift
Port: set from start_skip_samples

Empirical fact pinned (fixture: 117 frames in file, tag field = 116): LAME's frames counts audio frames excluding the tag frame, so C's formulas transplant unchanged into the after-tag pts timeline. One off-by-4 found en route: C's xing_offtbl is measured after the 4 consumed header bytes.

Acceptance — the smoke now runs the real pipeline (open → read_frame → decode, zero manual trimming):
- Length: 132300 = 132300 samples exactly (== ffmpeg), start_skip=1105, max_diff=0.0000
- CLI -i /tmp/t_mono.mp3 out.wav vs system ffmpeg: payload identical except 43/132300 samples at ±1 LSB (float→s16 boundary rounding); deltas all {|1: 26, −1: 17}
- CLI dump fixed on the way: Duration: 00:00:03.00, start: 0.025057, bitrate: 130 kb/s / Stream #0:0: Audio: mp3, 44100 Hz, mono, fltp, 128 kb/s, start 0.025057 — the Audio branch also un-mislabels WAV inputs (was Video: pcm_s16le)

New tests: 2 demuxer (synthetic LAME tag → gapless export + tag skip; read_frame injection on first/last packets), 3 decoder (trim, cross-frame skip carry, flush reset). Memory roadmap updated with the port map, the frames-field fact, and the fixture-regen commands.

Next per the ladder: AAC.
```
