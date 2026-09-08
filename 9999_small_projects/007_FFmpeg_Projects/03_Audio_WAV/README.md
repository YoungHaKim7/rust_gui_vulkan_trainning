Recovered prerequisite: the FFmpeg C tree is back — now inside the repo (./FFmpeg/ + ./rust_projects/, your "two folders"). All of libswresample + the libavutil audio foundations verified present (~4K lines of C in scope for this wave).

- In flight — 5 spec agents deep-reading the C, then 5 worktree implementers:

|     Zone     |        C source         |            Delivers              |
|-|-|-|                            
| fundamentals | `samplefmt.c`, + <br />  `channel_layout.c` + <br />| AVFrame audio fields     | `util/channel_layout.rs`,<br />`util/channel_layout.rs`,<br /> `util/audio_frame.rs`          |
| swr-driver   | `swresample.c` (983<br />lines) + options |`swresamp/mod.rs` — SwrContext,<br />swr_convert loop, delay/flush  |
| audioconvert | `audioconvert.c`          | the format-pair conversion matrix<br /> ,(int↔float, packed↔planar) |
| rematrix   | `rematrix.c (879)`        | mix matrix build + apply <br /> (stereo↔mono, 5.1 downmix, exact<br /> coefficients)|            
| resample   | `resample.c (513)` |   the Kaiser-windowed polyphase<br />     resampler — the bit-exactness <br />determinant  |



- The zones meet at C's own swri_* contract signatures from swresample_internal.h, so parallel implementers stay coherent by construction. Each commits in its worktree; I integrate + wire lib.rs/util/mod.rs + fix cross-zone seams when the workflow lands.

Queued next (task #2): Phase 4b — WAV demux/mux (riffdec/riffenc core), PCM codec, CLI (-i in.wav -ar/-ac/-sample_fmt), and golden tests vs system ffmpeg (resample 48k→44.1k, s16→f32, stereo→mono).

I'll pick up the moment the workflow notifies.
