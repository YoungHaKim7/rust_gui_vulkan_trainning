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

#   Let me debug it directly. The plan:
- 1. Add mid-chain dumps to the decoder (IMDCT output / synth output per frame)
- 2. Compare against what C would produce — I can't easily run C, but I CAN compare the final signal against the reference at a structural level.

- Actually, a smarter approach: the correlation is zero at ALL lags. If the bug were a constant delay, some lag would correlate. Zero correlation everywhere with correct energy means either:
  - Channel swap + phase inversion per granule? L↔R swap would still correlate 
  - Per-granule time REVERSAL: energy/spectrum preserved, correlation zero! A
    reversed IMDCT output o do exactly this.
  - Random-ish block permutation of the 18-sample or 32-sample sub-blocks.

- Test: reverse our whole output and check correlation with ref? No — if sub-blocks are reversed individually. But if each granule (576
  samples) is internally reversed, correlation of our-vs-reversed-our... complex.  
  Better empirical probe: autocorrelation of our output at 576 lag vs ref's at 576. For a pure sine, autocorr40Hz periodicity either
  way.                                                                             
  Direct approach instead: compare our decoded samples against the reference SAMPLE BY SAMPLE in the first 11formations:
  1. our[k] vs ref[k] — known zero
  2. our granule-reversed: , compare
  3. reverse each 32-block (synth output)
  4. reverse each 18-block                                                                                That will pinpoint the rel is the bug. Let me dothis in Python on /tmp/our.pcm (need to regenerate it — the dump hook was added before the fmt commits; c

```bash
Bash(grep -n "sb_samples\|synth_filter\|fn synth" src/codec/audio/mp3.rs | head -12; sed -n '1560,1620p' FFmpeg/libavcodec/mpegaudiodec_template.c)
41://! | `ff_mpa_synth_filter_float` (`:178-195`) | [`mpa_synth_filter`] |
56://! long bands, three 12-point for short) into `sb_samples[ch][36][32]`
828:    sb_samples: [[f32; 36 * SBLIMIT]; MPA_MAX_CHANNELS],
849:            sb_samples: [[0.0; 36 * SBLIMIT]; MPA_MAX_CHANNELS],
1023:                    &mut self.sb_samples,
1190:                    &mut self.sb_samples,
1376:                let row = &self.core.sb_samples[ch][i * SBLIMIT..(i + 1) * SBLIMIT];
1377:                mpa_synth_filter(
2196:/// `out` is the granule's `sb_samples` base, `buf` the channel's
2507:/// `ff_mpa_synth_filter_float` (`mpegaudiodsp_template.c:178-195`) —
2509:fn mpa_synth_filter(
2516:    sb_samples: &[f32],
{
    const uint8_t *buf
    int buf_size        = avpkt->size;
    MPADecodeContext *s
    uint32_t header;
    int ret;

    int skipped = 0;
    while(buf_size && !*buf){
        buf++;
        buf_size--;
        skipped++;
    }

    if (buf_size < HEADER_SIZE)
        return AVERROR_

    header = AV_RB32(bu
    if (header >> 8 == AV_RB32("TAG") >> 8) {
        av_log(avctx, A tag\n");
        return buf_size + skipped;
    }
    ret = avpriv_mpegaudio_decode_header((MPADecodeHeader *)s, header);
    if (ret < 0) {
        av_log(avctx, AV_LOG_ERROR, "Header missing\n");
        return AVERROR_
    } else if (ret == 1) {
        /* free format:ze */
        s->frame_size = -1;
        return AVERROR_
    }
    /* update codec inf
    av_channel_layout_uninit(&avctx->ch_layout);
    avctx->ch_layout =
(AVChannelLayout)AV_CHANNEL_LAYOUT_MONO :

(AVChannelLayout)AV_CHANNEL_LAYOUT_STEREO;
    if (!avctx->bit_rat
        avctx->bit_rate = s->bit_rate;

    if (s->frame_size <= 0) {
        av_log(avctx, Ame\n");
        return AVERROR_INVALIDDATA;
    } else if (s->frame
        av_log(avctx, AV_LOG_DEBUG, "incorrect frame size - multiple frames in
buffer?\n");
        buf_size= s->frame_size;
    }

    s->frame = frame;

    ret = mp_decode_fra
    if (ret >= 0) {
        s->frame->nb_sa
        *got_frame_ptr       = 1;
        if (avctx->code
            avctx->sample_rate = s->sample_rate;
        //FIXME maybe mff from above here too
    } else {
        av_log(avctx, Acoding MPEG audio
frame.\n");
        /* Only return kes up the whole packet or
         * the error is related to buffer management.
         * If there is t consume the bad frame
         * instead of returning an error, which would discard the whole

● Bash(sed -n '1360,1400p' src/codec/audio/mp3.rs; echo "=== C synth loop ==="; sed
  -n '1690,1735p' FFmpeg/libe.c)
        let nb_frames = self.core.mp_decode_frame(&buf[..frame_bytes])?;

        // get output buffer + apply the synthesis filter
        // (template.c:
        let frame_size = self.params.frame_size as usize;
        let mut frame =mat::Fltp,
self.params.ch_layout, frame_size)?;
        frame.pts = pkt
        frame.duration = pkt.duration;
        frame.time_base
        frame.sample_rate = self.core.header.sample_rate;

        let t = tables();
        let mut samples
        for ch in 0..self.core.header.nb_channels as usize {
            samples.fil
            for i in 0..nb_frames {
                let row[i * SBLIMIT..(i + 1) *
SBLIMIT];
                mpa_syn
                    &mut self.core.synth_buf[ch],
                    &mu[ch],
                    &t.synth_window,
                    &mu
                    &mut samples[i * 32..(i + 1) * 32],
                    1,
                    row,
                );
            }
            let plane =
            for (dst, src) in plane.chunks_exact_mut(4).zip(samples.iter()) {
                dst.cops());
            }
        }
        Ok(Some(frame))
    }
}

impl AudioDecoder for Mp3Decoder {
    /// `decode_ctx_ini:283-315`) — the mp3
    /// float codec gate: output `OUT_FMT_P` = FLTP.
    fn init(&mut self,  Result<()> {
        match params.codec_id {
=== C synth loop ===

    *got_frame_ptr = 1;

    return buf_size;
}
#endif /* CONFIG_MP3ADUAT_DECODER */

#if CONFIG_MP3ON4_DECODODER

/**
 * Context for MP3On4 decoder
 */
typedef struct MP3On4DecodeContext {
    int frames;        mp3 frames per block
(number of mp3 decoder instances)
    int syncword;      atch
    const uint8_t *coff;            ///< channel offsets in output buffer
    MPADecodeContext *montext for every decoder
instance                                                                    } MP3On4DecodeContext;

#include "mpeg4audio.h"

/* Next 3 arrays are iner (passed via codecdata)
*/

/* number of mp3 decoder instances */
static const uint8_t mp, 3, 4, 5 };

/* offsets into output is FL FR C LFE BL BR SL SR
*/
static const uint8_t ch
    { 0             },
    { 0             },
    { 0             },  // FLR
    { 2, 0          },
    { 2, 0, 3       },  // C FLR BS
    { 2, 0, 3       },
    { 2, 0, 4, 3    },  // C FLR BLRS LFE
    { 2, 0, 6, 4, 3 },
};

/* mp3on4 channel layouts */
static const int16_t ch
    0,
    AV_CH_LAYOUT_MONO,
    AV_CH_LAYOUT_STEREO,
    AV_CH_LAYOUT_SURROU
    AV_CH_LAYOUT_4POINT0,
    AV_CH_LAYOUT_5POINT

● Bash(grep -n "ff_mpa_synth
  FFmpeg/libavcodec/mpegaudiodec_template.c | grep -nE "for|out_samples|synth" |
  head -10; sed -n '1495,154diodec_template.c)
10:1535-    /* apply the synthesis filter */
11:1536-    for (ch = 0 {
20:1545-        for (i = 0; i < nb_frames; i++) {
21:1546:            REN>mpadsp, s->synth_buf[ch],

        s->last_buf_siz
        if (s->in_gb.buffer) {
            align_get_b
            i = (get_bits_left(&s->gb) >> 3) - s->extrasize;
            if (i >= 0
                memmove(s->last_buf, s->gb.buffer + (get_bits_count(&s->gb) >>
3), i);
                s->last_buf_size=i;
            } else
                av_log(s->avctx, AV_LOG_ERROR, "invalid old backstep %d\n",
i);
            s->gb           = s->in_gb;
            s->in_gb.bu
            s->extrasize    = 0;
        }

        align_get_bits(
        av_assert1((get_bits_count(&s->gb) & 7) == 0);                              i = (get_bits_left(&s->gb) >> 3) - s->extrasize;
        if (i < 0 || i > BACKSTEP_SIZE || nb_frames < 0) {
            if (i < 0)
                av_log(s->avctx, AV_LOG_ERROR, "invalid new backstep %d\n",
i);
            i = FFMIN(BACKSTEP_SIZE, buf_size - HEADER_SIZE);
        }
        av_assert1(i <= buf_size - HEADER_SIZE && i >= 0);
        memcpy(s->last_b.buffer + buf_size -
HEADER_SIZE - i, i);
        s->last_buf_siz
    }

    if(nb_frames < 0)
        return nb_frames;

    /* get output buffe
    if (!samples) {
        av_assert0(s->f
        s->frame->nb_samples = s->avctx->frame_size;
        if ((ret = ff_g, 0)) < 0)
            return ret;
        samples = (OUT_ta;
    }

    /* apply the synthesis filter */
    for (ch = 0; ch < s
        int sample_stride;
        if (s->avctx->sample_fmt == OUT_FMT_P) {
            samples_ptr   = samples[ch];
            sample_stri
        } else {
            samples_ptr
            sample_stride = s->nb_channels;
        }
        for (i = 0; i < nb_frames; i++) {

● Bash(sed -n '1545,1560p' FFmpeg/libavcodec/mpegaudiodec_template.c; echo "===
  port imdct write into sb_s30p'
  src/codec/audio/mp3.rs; sed -n '1180,1200p' src/codec/audio/mp3.rs)
        for (i = 0; i <
            RENAME(ff_mpa_synth_filter)(&s->mpadsp, s->synth_buf[ch],
                                        &(s->synth_buf_offset[ch]),
                                        RENAME(ff_mpa_synth_window),
                       state, samples_ptr,
                                        sample_stride, s->sb_samples[ch][i]);
            samples_ptr
        }
    }

    return nb_frames * _channels;
}

static int decode_frame(AVCodecContext *avctx, AVFrame *frame,
                       t *avpkt)
{
=== port imdct write in
        let mut last_buf_bits = (self.last_buf_size * 8) as i64;

        // now we get bits from the main_data_begin offset
        let mut gr = 0usize;
        while gr < nb_granules && (last_buf_bits >> 3) < main_data_begin as
i64 {
            for ch in 0..nch {
                let g =
                last_buf_bits += g.part2_3_length as i64;
                g.sb_hy
                compute_imdct(
                    ch,
                    gr,
                    &mu
                    &mut self.sb_samples,
                    &mu
                );
            }
            gr += 1;
        }
        let skip = last_buf_bits - 8 * main_data_begin as i64;
        if skip >= selfs.extrasize as i64 * 8 &&
self.bs.in_gb.is_some()
                compute_stereo(self.header.mode_ext, lsf, sri, &mut
self.granules, gr);
            }

            for ch in 0
                reorder_block(sri, &mut self.granules[ch][gr]);
                computes[ch][gr]);
                compute_imdct(
                    ch,
                    gr,
                    &mu
                    &mut self.sb_samples,
                    &mu
                );
            }
            gr += 1;
        } /* gr */
        if self.bs.gb.get_bits_count() < 0 {
            let n = -se
            self.bs.gb.skip_bits_long(n);
        }
        Ok(nb_granules * 18)

∴ The synth loop matches C. The bug must be inside compute_imdct (the sb_samples
  write pattern) or mpa_synt_imdct
  (template.c:1358-1418ish) writes:

  for (i = 0; i < 36; i += 18) // two blocks... actually:
  IMDCT window: for each blo]
  s->sb_samples[ch][18*granule + i][sblimit...
  Actually C writes ptr = &s->sb_samples[ch][18 * gr][0] roughly, i.e. sb_samples
  is [32][18][SBLIMIT] with [i][sb] indexing: sample index i (0..36 per granule
  pair), subband sb. The synamples[ch][i] = 18 values ×
  32 subbands for frame i? No wait — C's synth gets s->sb_samples[ch][i] where
  sb_samples is declared DEC
  sb_samples)[MPA_MAX_CHANNELS][36][SBLIMIT]. So [ch][36][32] — 36 rows of 32
  subband values. nb_frames rame returns nb_frames =
  18*nb_granules/32? For MPEG1 stereo: 2 granules × 18 = 36 rows, each synth call
  consumes ONE row producingamples.

  Port has sb_samples: [[f32NNELS] — flat 3632 with row
  = iSBLIMIT — matches [36][32].

  So the bug is likely in compute_imdct's write into sb_samples or in
  mpa_synth_filter. I need tnst C's (template.c
  ~1358-1418) and mpa_synth_filter against mpegaudiodsp_template.c:178-195. Let me
  read both.

● Bash(grep -n "static void
  FFmpeg/libavcodec/mpegaudiodec_template.c; sed -n "$(grep -n 'static void
  compute_imdct' FFmpeg/liba.c | cut -d: -f1),+62p"
  FFmpeg/libavcodec/mpegaudiodec_template.c)
1132:static void comput GranuleDef *g,
static void compute_imdct(MPADecodeContext *s, GranuleDef *g,
                          INTFLOAT *sb_samples, INTFLOAT *mdct_buf)
{
    INTFLOAT *win, *out
    INTFLOAT out2[12];
    int i, j, mdct_long

    /* find last non ze
    ptr  = g->sb_hybrid + 576;
    ptr1 = g->sb_hybrid
    while (ptr >= ptr1) {
        int32_t *p;
        ptr -= 6;
        p    = (int32_t
        if (p[0] | p[1] | p[2] | p[3] | p[4] | p[5])
            break;
    }
    sblimit = ((ptr - g
    if (g->block_type == 2) {
        /* XXX: check for 8000 Hz */
        if (g->switch_p
            mdct_long_end = 2;
        else
            mdct_long_end = 0;
    } else {
        mdct_long_end = sblimit;
    }

    s->mpadsp.RENAME(imct_buf, g->sb_hybrid,
                                     mdct_long_end, g->switch_point,
                       ;

    buf = mdct_buf + 4*dct_long_end & 3);
    ptr = g->sb_hybrid + 18 * mdct_long_end;

    for (j = mdct_long_end; j < sblimit; j++) {
        /* select frequ
        win     = RENAME(ff_mdct_win)[2 + (4  & -(j & 1))];
        out_ptr = sb_sa
        for (i = 0; i < 6; i++) {
            *out_ptr = buf[4*i];
            out_ptr +=
        }
        imdct12(out2, p
        for (i = 0; i < 6; i++) {
            *out_ptr       ], 1) + buf[4*(i +
6*1)];
            buf[4*(i + in[i + 6], 1);
            out_ptr += SBLIMIT;
        }
        imdct12(out2, ptr + 1);
        for (i = 0; i <
            *out_ptr     = MULH3(out2[i    ], win[i    ], 1) + buf[4*(i +
6*2)];
            buf[4*(i + 6*0)] = MULH3(out2[i + 6], win[i + 6], 1);
            out_ptr +=        }
        imdct12(out2, ptr + 2);
        for (i = 0; i < 6; i++) {
            buf[4*(i + in[i    ], 1) + buf[4*(i +
6*0)];
            buf[4*(i + in[i + 6], 1);
            buf[4*(i + 6*2)] = 0;
        }

● Bash(sed -n '76,178p' FFmemplate.c)
    is just to preserve alignment for SIMD implementations.
*/
DECLARE_ALIGNED(16, INTFLOAT, RENAME(ff_mdct_win))[8][MDCT_BUF_SIZE];

DECLARE_ALIGNED(16, MPA_INT, RENAME(ff_mpa_synth_window))[512+256];

#define SUM8(op, sum, w, p)               \
{
    op(sum, (w)[0 * 64], (p)[0 * 64]);    \
    op(sum, (w)[1 * 64
    op(sum, (w)[2 * 64], (p)[2 * 64]);    \
    op(sum, (w)[3 * 64
    op(sum, (w)[4 * 64], (p)[4 * 64]);    \
    op(sum, (w)[5 * 64
    op(sum, (w)[6 * 64], (p)[6 * 64]);    \
    op(sum, (w)[7 * 64
}

#define SUM8P2(sum1, op1, sum2, op2, w1, w2, p) \
{
    INTFLOAT tmp;\
    tmp = p[0 * 64];\
    op1(sum1, (w1)[0 * 64], tmp);\
    op2(sum2, (w2)[0 *
    tmp = p[1 * 64];\
    op1(sum1, (w1)[1 * 64], tmp);\
    op2(sum2, (w2)[1 * 64], tmp);\
    tmp = p[2 * 64];\
    op1(sum1, (w1)[2 * 64], tmp);\
    op2(sum2, (w2)[2 *
    tmp = p[3 * 64];\
    op1(sum1, (w1)[3 *
    op2(sum2, (w2)[3 * 64], tmp);\
    tmp = p[4 * 64];\
    op1(sum1, (w1)[4 * 64], tmp);\
    op2(sum2, (w2)[4 *
    tmp = p[5 * 64];\
    op1(sum1, (w1)[5 *
    op2(sum2, (w2)[5 * 64], tmp);\
    tmp = p[6 * 64];\
    op1(sum1, (w1)[6 * 64], tmp);\
    op2(sum2, (w2)[6 *
    tmp = p[7 * 64];\
    op1(sum1, (w1)[7 *
    op2(sum2, (w2)[7 * 64], tmp);\
}

void RENAME(ff_mpadsp_apply_window)(MPA_INT *synth_buf, MPA_INT *window,
                                  int *dither_state, OUT_INT *samples,

{
    register const MPA
    int j;
    OUT_INT *samples2;
#if USE_FLOATS
    float sum, sum2;
#else
    int64_t sum, sum2;
#endif

    /* copy to avoid wrap */
    memcpy(synth_buf +*synth_buf));

    samples2 = samples
    w = window;
    w2 = window + 31;

    sum = *dither_state;
    p = synth_buf + 16;
    SUM8(MACS, sum, w,
    p = synth_buf + 48;
    SUM8(MLSS, sum, w
    *samples = round_sample(&sum);
    samples += incr;
    w++;

    /* we calculate two samples at the same time to avoid one memory
       access per two
    for(j=1;j<16;j++) {
        sum2 = 0;
        p = synth_buf + 16 + j;
        SUM8P2(sum, MACS, sum2, MLSS, w, w2, p);
        p = synth_buf + 48 - j;
        SUM8P2(sum, ML32, p);

        *samples = rou
        samples += incr;
        sum += sum2;
        *samples2 = round_sample(&sum);
        samples2 -= incr;
        w++;
        w2--;
    }

    p = synth_buf + 32;
    SUM8(MLSS, sum, w
    *samples = round_sample(&sum);
    *dither_state= sum;
}

/* 32 sub band synthesis filter. Input: 32 sub band samples, Output:
   32 samples. */
void RENAME(ff_mpa_synth_filter)(MPADSPContext *s, MPA_INT *synth_buf_ptr,

● Bash(sed -n '178,200p' FFmpeg/libavcodec/mpegaudiodsp_template.c; echo "=== port
  mpa_synth_filter + apply_0p' src/codec/audio/mp3.rs)
void RENAME(ff_mpa_synth_filter)(MPADSPContext *s, MPA_INT *synth_buf_ptr,
                      set,
                                 MPA_INT *window, int *dither_state,
                      ptrdiff_t incr,
                                 MPA_INT *sb_samples)
{
    MPA_INT *synth_buf;
    int offset;

    offset = *synth_bu
    synth_buf = synth_buf_ptr + offset;

    s->RENAME(dct32)(synth_buf, sb_samples);
    s->RENAME(apply_wier_state, samples, incr);

    offset = (offset - 32) & 511;
    *synth_buf_offset = offset;
}

static av_cold void mp
{
    int i, j;

=== port mpa_synth_fil
/// `ff_mpa_synth_filter_float` (`mpegaudiodsp_template.c:178-195`) —        /// 32 sub band synthesis: input 32 subband samples, output 32 samples.
fn mpa_synth_filter(
    synth_buf: &mut [f
    synth_buf_offset: &mut usize,
    window: &[f32],
    dither_state: &mut i32,
    samples: &mut [f32
    incr: usize,
    sb_samples: &[f32],                                                      ) {
    let offset = *synth_buf_offset;
    dct32(&mut synth_bamples);
    apply_window(                                                                    &mut synth_buf[offset..],
        window,
        dither_state,
        samples,
        incr,
    );
    *synth_buf_offset
}                                                                            /// `l3_unscale` (`mpegaudiodec_template.c:222-239`) — compute
/// `value^(4/3) * 2^(exponent/4)` normalized to FRAC_BITS, via the
/// shared fixed-point decoder's linbits
/// escape path exactly as in C).
fn l3_unscale(value: i
    let t = tables();
    let idx = (4 * vals usize;
    let idx = idx.min(TABLE_4_3_SIZE - 1);                                       let mut e = t.table_4_3_exp[idx] as i32;
    let m = t.table_4_3_value[idx];
    e -= exponent >> 2
    if !(0..=31).contains(&e) {
        // C: `if (e >tive e reads as
        // unsigned, so both out-of-range directions return 0.
        return 0;
    }                                                                            let m = (m + ((1u32 << e) >> 1)) >> e;
    m as i32
}

// ----------------------------------------
// Layer 3 scale-factor helpers — mpegaudiodec_template.c:659-723
// ----------------------------------------

/// The `SPLIT` macro -677`).
fn split(dst: &mut i32, sf: &mut i32, n: i32) {
    match n {
        3 => {                                                                           let m = (*sf * 171) >> 9;
            *dst = *sf - 3 * m;
            *sf = m;
        }
        4 => {
            *dst = *sf & 3;
            *sf >>= 2;
        }                                                                            5 => {
            let m = (*sf * 205) >> 10;
            *dst = *sf
            *sf = m;
        }
        6 => {
            let m = (*
            *dst = *sf - 6 * m;
            *sf = m;
        }
        _ => *dst = 0,
    }                                                                        }

/// `lsf_sf_expand` (`86`).
pub fn lsf_sf_expand(slen: &mut [i32; 4], mut sf: i32, n1: i32, n2: i32, n3:
i32) {
    split(&mut slen[3], &mut sf, n3);
    split(&mut slen[2]
    split(&mut slen[1], &mut sf, n1);
    slen[0] = sf;
}

/// `exponents_from_scale_factors` (`mpegaudiodec_template.c:688-723`).
fn exponents_from_scalleDef, exponents: &mut
[i16; 576]) {
    let mut ptr = 0usi
    let gain = g.globa
    let shift = g.scalefac_scale as i32 + 1;                               
    let sri = sri as usize;
    let bstab = &FF_BA
    let pretab = &FF_MPA_PRETAB[(g.preflag != 0) as usize];                    for i in 0..g.long_end as usize {
        let v0 = gain - ((g.scale_factors[i] as i32 + pretab[i] as i32) <<
shift) + 400;
        for _ in 0..bstab[i] {
            exponents[
            ptr += 1;
```

# Codec ladder (your roadmap) — step 3 in flight 🔄

|     Ladder      |                                                           Status                                                            |
|-|-|
| 1. PCM          | ✅                                                                                                                          |
| 2. WAV          | ✅                                                                                                                          |
| 3. MP3 decode                | 🔄 agent working — the full float mpegaudio family (~3.3K lines of C: header unpack, L3 sideinfo/scalefactors, requantize + <br />MS/intensity stereo, alias reduce, dct32 IMDCT, the polyphase synth window via the tablegen formulas) → codec/mp3.rs as an <br />AudioDecoder producing FLTP frames|
| 4. AAC → <br /> 5. H.264 →<br /> 6. MP4    |  queued, one at a time                                                                        |

- The MP3 zone explicitly allows an honest partial (dependency-ordered with named Unsupported guards) rather than a fake-complete — after it lands I'll wire the registry, smoke it against system ffmpeg's decode output, and queue the MP3 frame demuxing so -i file.mp3 works end to end. I'll report when the notification arrives.
  - `mpegaudiotabs.h` LSF row, band index 283), one test over-specification (C's reader zero-pads — 240 is right), and the huff one needs empirical probing. Fixing the clear ones first:

- C verified. Three test bugs and two impl bugs. Reading the two impl sites:


- C verdicts: three test bugs (free-format header byte, 156 is C-correct per mpegaudiotabs.h LSF row, band index 283), one test over-specification (C's reader zero-pads — 240 is right), and the huff one needs empirical probing. Fixing the clear ones first:


- Two more to pin empirically — probing the header fields and huff table 0 directly:


- MPEG-1 — impl correct). Probing the huff decode directly:

- Both header mysteries are C-verified test bugs (0xFFF72000 is MPEG-2 LSF, not MPEG-1 — impl correct). Probing the huff decode directly:

- All decodes return 49 and always consume 3 bits — the search never matches. Tracing inside decode itself:

- Every bit reads 0 even for a set bit. Probing bit() directly:


- Found it — the test helper packs bits as a number (LSB-aligned) while the reader consumes MSB-first; 0b001 becomes 0x01, so every decoded bit is 0. The impl is correct. Fixing the helper + the two MPEG-1 headers (0xFFF72/0xFFF59 are MPEG-2 LSF — C-verified 104/522) and cleaning my probes:

<hr />

# Phase 5 — COMPLETE ✅ (memory updated)
- NUT container: demuxer + muxer, the write_index infinite-loop fix, r_frame_rate
  tag — ffmpeg↔ours round trips byte-identical both ways (video + audio), 3
  cross-tool goldens
- Geometry filters: crop / transpose / hflip / vflip — 9 chains byte-exact vs
  system ffmpeg, 42 unit tests
- Suite: 421 lib + 20 golden, all green in ~5s

## Codec ladder (your roadmap) — step 3 in flight 🔄
```
Ladder: 1. PCM
Status: ✅
────────────────────────────────────────
Ladder: 2. WAV
Status: ✅
────────────────────────────────────────
Ladder: 3. MP3 decode
Status: 🔄 agent working — the full float mpegaudio family (~3.3K lines of C:
header unpack, L3 sideinfo/scalefactors, requantize + MS/intensity stereo, alias
 reduce, dct32 IMDCT, the polyphase synth window via the tablegen formulas) →
codec/mp3.rs as an AudioDecoder producing FLTP frames
────────────────────────────────────────
Ladder: 4. AAC → 5. H.264 → 6. MP4 → …
Status: queued, one at a time
```
- The MP3 zone explicitly allows an honest partial (dependency-ordered with named Unsupported guards) rather than a fake-complete — after it lands I'll wire the registry, smoke it against system ffmpeg's decode output, and queue the MP3 frame demuxing so -i file.mp3 works end to end. I'll report when the notification arrives.

- 1 tasks (0 done, 1 open)
  - ◻ Codec ladder: MP3 decoder (agent in flight), then demux+golden
- ◯ impl-mp3-decode  Port the MP3 decoder (mpegaudio family, decode side) to codec/mp3.rs — single agent, sequential  


# Phase 5 progress

|         Item          |                       Status                       |
|-|-|
| NUT demuxer (Zone A)  | ✅                                                 |
| NUT muxer (Zone B)    | ✅                                                 |
|                       | ✅ fixed — write_index's for(j...) → while         |
| Infinite-loop bug     | ✅ fixed — write_index's for(j...) → while <br />translation lost the increment; one-line restore,<br />all five disabled tests re-enabled green  |
| r_frame_rate tag + <br />cross-tool goldens <br />(Zone C)   | ✅ — ffmpeg→NUT→ours is byte-identical both ways <br /> ns committed  |
| crop / transpose /<br />hflip / vflip (Zone D)    | 🔄 agent working — 974 lines of C → 3 files; crop <br />transpose's 4 modes, <br />the flip pair in one file |

- Suite standing at 379 libn, all green in ~5s.

- On perf.data: it's root-oopen it — if you want theprofile itself analyzed, run ! sudo chown gy:gy perf.data. That said, the root cause is already found anhe DBG instrumentationtrail in the conversation above): the "slow" tests were hung in the trailer's index writer, and their freal work — shrinkingsamples 90% would not have helped.
  - Your codec roadmap (PCM→WAV→MP3→AAC→H.264→MP4→…→MKV) is noted for the phases     after this one — FLAC or  increment after the filter trio, each as its own zone in the codec/audio/ tree you sketched. I'll integrate Zone D when its notificat

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
