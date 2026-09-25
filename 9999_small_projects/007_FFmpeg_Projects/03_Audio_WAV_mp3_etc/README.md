# 260925

- The harness's per-MB memset hack corrupted the left border — remove it (real C's borders stay 64):

- Found it — the first-level suffix update level_code + 3U > 6U is an unsigned compare; negative levels wrap and bump sl to 2. My Rust used signed:

```bash
$ J=/Users/gy/.claude/jobs/effb161f/tmp; $J/h264c /tmp/h264_alli.h264 2>/dev/null | grep -aE "n=1[1-5] |^MB1 |cp=" | head -8; echo ---; H264_DUMP=1 cargo test --lib video::h264 -- --nocapture 2>&1 | grep -aE "RES n=1[1-9] |TOK n=1[2-9] " | head -8
```

<hr />


- All of libswresample + the libavutil audio foundations verified present (~4K lines of C in scope for this wave).

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

# mp3

- 9.200282 matched the port, not my formula — reverted.)

- Two synthetic tests had encoded the buggy behavior ('10' as a table-1 codeword — actually a complete 1-bit code for symbol 0, y=0, silent pair; the correct code for 0x10 is '01'), and the impulse probe mis-expected odd-band frequency inversion. All three now assert C-true behavior.

- Codec ladder standing

    |            Step             |                    Status                    |
    |-|-|
    | PCM, WAV                    | ✅                                           |
    | MP3 decode                  | ✅ today                                     |
    | MP3 demux + -i x.mp3 CLI    | next (small: frame slicing + codec dispatch) |
    | AAC → H.264 → MP4 → … → MKV | queued, one per session as you asked         |

- The env-gated diagnostics stay in the tree (MP3_DUMP for sb_hybrid band energies, MP3_FBD for full sideinfo dumps) — they're what pinned the diagnosis, and the same pattern will audit AAC when its turn comes.
  - ※ recap: FFmpeg-to-Rust port: MP3 d
