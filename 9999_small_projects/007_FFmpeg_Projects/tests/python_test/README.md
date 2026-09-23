```py

Bash(python3 << 'EOF'
  import re
  src = open('FFmpeg/libavcodec/mpegaudiodsp_template.c').read()
  cdefs = re.findall(r'#define C\d+ [^\n]*', src)
  i12 = re.findall(r'#define I12_C\d+ [^\n]*', src)
  icos36 = re.search(r'static const INTFLOAT\s+icos36\[9\]\s*=\s*\{[^}]*\};', src).group(0)
  icos36h = re.search(r'static const INTFLOAT\s+icos36h\[9\]\s*=\s*\{[^}]*\};', src).group(0)
  imdct = re.search(r'static void imdct36\(.*?\n\}', src, re.S).group(0)
  harness = '''#include <stdio.h>
  #include <math.h>
  #include <string.h>
  #define USE_FLOATS 1
  #define MDCT_BUF_SIZE 40
  typedef float INTFLOAT; typedef float SUINTFLOAT; typedef float MPA_INT;
  #define FIXHR(x) ((float)(x))
  #define FIXR(x) ((float)(x))
  #define MULH3(x,y,s) ((s)*(y)*(x))
  #define SHR(a,b) ((a)*(1.0f/(1<<(b))))
  ''' + '\n'.join(cdefs + i12) + '\n' + icos36 + '\n' + icos36h + '\n' + imdct + '''
  int main(void) {
      static float win0[40];
      for (int i = 0; i < 36; i++) {
          double d = sin(M_PI * (i + 0.5) / 36.0);
          d *= 0.5 * 1.759 / cos(M_PI * (2*i + 19) / 72);
          int idx = i < 18 ? i : i + (MDCT_BUF_SIZE/2 - 18);
          win0[idx] = (float)(d / 32.0);
      }
      for (int t = 0; t < 5; t++) {
          float in[18], out[36*32]; static float buf[4*18*8];
          memset(out, 0, sizeof out); memset(buf, 0, sizeof buf);
          unsigned long long x = t*2654435761ULL + 12345;
          for (int k = 0; k < 18; k++) {
              x = x*6364136223846793005ULL + 1442695040888963407ULL;
              in[k] = (float)((int)(x >> 33)) / 65536.0f;
          }
          imdct36(out, buf, in, win0);
          printf("T%d IN", t);
          for (int k = 0; k < 18; k++) printf(" %.6e", in[k]);
          printf("\\nT%d OUT", t);
          for (int n = 0; n < 18; n++) printf(" %.6e", out[n*32]);
          printf("\\nT%d BUF", t);
          for (int n = 0; n < 18; n++) printf(" %.6e", buf[4*n]);
          printf("\\n");
      }
      return 0;
  }
  '''
  open('/tmp/cprobe/harness.c','w').write(harness)
  print('harness v4 ok')
  EOF
  cc -O1 -o /tmp/cprobe/h /tmp/cprobe/harness.c -lm 2>&1 | head -6 && /tmp/cprobe/h > /tmp/cprobe/ref.txt && head -3 /tmp/cprobe/ref.txt)
Error: Exit code 127
harness v4 ok
/tmp/cprobe/harness.c:93:14: error: call to undeclared function 'MULLx'; ISO C99 and later do not support implicit function declarations [-Wimplicit-function-declaration]
   93 |         s3 = MULLx(t3 - t2, icos36 [8 - j], FRAC_BITS);
      |              ^
/tmp/cprobe/harness.c:93:45: error: use of undeclared identifier 'FRAC_BITS'
   93 |         s3 = MULLx(t3 - t2, icos36 [8 - j], FRAC_BITS);
      |                                             ^~~~~~~~~
(eval):51: no such file or directory: /tmp/cprobe/h


Found it — intra4x4 CBP is read from the bitstream in C, my code never reads it. Fix:

⏺ Bash(python3 -c "
  p = 'src/codec/video/h264.rs'
  s = open(p).read()
  s = s.replace('''        let cmode = gb.ue()? as i32;
          self.chroma_pred_mode = self.check_intra_pred_mode(cmode, true)?;

          // residual
          self.decode_mb_residual(gb, mb_xy)?;''',
  '''        let cmode = gb.ue()? as i32;
          self.chroma_pred_mode = self.check_intra_pred_mode(cmode, true)?;

          // cbp: the I-table's -1 (255) means \"read from bitstream\" for
          // non-16x16 intra (cavlc.c:1053-1064).
          if self.cbp == u32::MAX {
              let mut cbp = gb.ue()?;
              if cbp > 47 {
                  return Err(Error::InvalidData(\"cbp too large\".into()));
              }
              cbp = GOLOMB_TO_INTRA4X4_CBP[cbp as usize] as u32;
              self.cbp = cbp;
          }

          // residual
          self.decode_mb_residual(gb, mb_xy)?;''')
  open(p,'w').write(s)
  print('ok')
  "; cargo test --lib "video::h264" -- --nocapture 2>&1 | grep -E "H264 SMOKE|test result|Err value|panicked at" | head -4)
ok
thread 'codec::video::h264::tests::decodes_all_intra_fixture' (30969) panicked at src/codec/video/h264.rs:3463:52:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 475 filtered out; finished in 0.00s

⏺ Bash(cargo test --lib "video::h264" -- --nocapture 2>&1 | grep -A2 "panicked at" | head -5)
thread 'codec::video::h264::tests::decodes_all_intra_fixture' (31089) panicked at src/codec/video/h264.rs:3463:52:
called `Result::unwrap()` on an `Err` value: InvalidData("negative zeros")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

⏺ Bash(python3 - <<'EOF'
  # Extend the python decoder through the first few MBs' residuals to find
  # where the streams diverge — print cbp + residual summary per MB.
  data = open('/tmp/h264_alli.h264','rb').read()
  i = data.find(b'\x00\x00\x01\x65')
  j = i + 4
  rbsp = bytearray()
  while j < len(data):
      if data[j] == 0 and data[j+1] == 0 and data[j+2] == 1: break
      if data[j] == 0 and data[j+1] == 0 and data[j+2] == 3:
          rbsp += b'\x00\x00'; j += 3; continue
      rbsp.append(data[j]); j += 1
  while rbsp and rbsp[-1] == 0: rbsp.pop()

  import re
  src = open('src/codec/video/h264/tables.rs').read()
  def arr(name, n):
      m = re.search(r'static ' + name + r': \[u8; (\d+)\] = \[(.*?)\];', src, re.S)
      v = [int(x) for x in re.findall(r'\d+', m.group(2))]
      return v[:n]

  class BR:
      def __init__(s, b): s.b, s.p = b, 0
      def r(s, n):
          v = 0
          for _ in range(n):
              byte = s.b[s.p>>3] if (s.p>>3) < len(s.b) else 0
              v = (v<<1) | ((byte >> (7-(s.p&7))) & 1); s.p += 1
          return v
      def ue(s):
          lz = 0
          while s.r(1) == 0: lz += 1
          return (1<<lz) - 1 + s.r(lz) if lz else 0
      def se(s):
          k = s.ue(); return ((k+1)>>1) * (1 if k & 1 else -1)

  g = BR(bytes(rbsp))
  g.ue(); g.ue(); g.ue(); g.r(4); g.ue(); g.r(4); g.se()  # header
  ZIGZAG = [0,1,4,8,5,2,3,6,9,12,13,10,7,11,14,15]
  CT_LEN = [arr(f'COEFF_TOKEN_LEN_{k}', 68) for k in range(4)]
  CT_BITS = [arr(f'COEFF_TOKEN_BITS_{k}', 68) for k in range(4)]

  def vlc(lens, bits):
      d = {}
      for idx, (l, c) in enumerate(zip(lens, bits)):
          if l: d[(c, l)] = idx
      return d, max(lens)

  CT_VLC = [vlc(CT_LEN[k], CT_BITS[k]) for k in range(4)]

  def coeff_token(nC):
      idx = min(16, nC)
      bucket = [0,0,1,1,2,2,2,2,3,3,3,3,3,3,3,3,3][idx]
      d, ml = CT_VLC[bucket]
      code, l = 0, 0
      for _ in range(ml+1):
          l += 1; code = (code<<1) | g.r(1)
          if (code, l) in d: return d[(code,l)]
      raise Exception('bad coeff_token')

  for mb in range(8):
      mbt = g.ue()
      if mbt != 0:
          print(f"MB {mb}: mb_type={mbt} (not 4x4) — stopping"); break
      for k in range(16):
          if not g.r(1): g.r(3)
      cp = g.ue()
      cbp_raw = g.ue()
      I4 = arr('GOLOMB_TO_INTRA4X4_CBP', 48)
      cbp = I4[cbp_raw]
      line = f"MB {mb}: type=4x4 cp={cp} cbp={cbp:#04x}"
      if cbp & 0xf:
          dq = g.se()
          line += f" dq={dq}"
          for i8 in range(4):
              if cbp & (1<<i8):
                  for i4 in range(4):
                      tok = coeff_token(0)  # nC 0 approx for the dump
                      tc = tok >> 2
                      line += f" b{i8}{i4}:tc={tc}"
      if cbp & 0x30:
          for ch in range(2):
              tok = coeff_token(0); line += f" cdc{ch}:tc={tok>>2}"
      print(line)
  EOF)


✳ Surveying H.264 C sources and port plumbing… 
```
