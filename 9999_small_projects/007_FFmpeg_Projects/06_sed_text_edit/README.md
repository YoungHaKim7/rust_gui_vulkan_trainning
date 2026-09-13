# `vi`
```bash
:1504,1513 m1496
```

# wc활용

- https://twpower.github.io/193-print-num-of-bytes-characters-words-lines-using-wc

# sed
```bash
sed -n '674,700p' src/swresample/resample.rs; sed -n '613,638p' src/swresample/resample.rs; sed -n '1033,1050p' src/swresample/resample.rs)
  ⎿      pub fn new


 Phase 4b begins. First the shared-base addition (audio fields on CodecParameters) so the zone agents build against a stable surface:

● Bash(sed -n '31,68p' src/codec/params.rs; grep -n "pub enum MediaType\|MediaType::" src/codec/params.rs | head -4)
  ⎿  pub enum CodecId {
         #[default]
         None,
     … +37 lines (ctrl+o to expand)

● Bash(sed -n '68,110p' src/codec/params.rs; grep -n "pub enum MediaType" -A 8 src/codec/params.rs)
  ⎿  #[derive(Debug, Clone)]
     pub struct CodecParameters {
         pub codec_type: MediaType,

grep -n "fps" src/format/y4m.rs | head -6)

  Bash(sed -n '498,510p' src/format/y4m.rs)
  ⎿  Running…
  ⎿          let st = &streams[0];
             let width = st.codecpar.width;
             let height = st.codecpar.height;

             // F from the stream timebase (reduced), like C.
             let (fps, _) = Rational::reduce(
                 st.time_base.den as i64,
                 st.time_base.num as i64,
                 (1u64 << 31) as i64 - 1,
             );

             let aspectn = st.sample_aspect_ratio.num;
             let mut aspectd = st.sample_aspect_ratio.den;
```
