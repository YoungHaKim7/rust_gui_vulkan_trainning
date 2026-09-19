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
```
