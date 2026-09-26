#include <stdio.h>
#include <stdint.h>
#include <string.h>

typedef struct { const uint8_t *buf; int index, size_in_bits; } GB;
static unsigned peek(GB *g, int n){
    if(n==0 || g->index >= g->size_in_bits) return 0;
    unsigned long long w=0; for(int k=0;k<8;k++){ unsigned b=g->buf[(g->index>>3)+k]; w=(w<<8)|b; }
    int sh = 64-(g->index&7)-n; return (w>>sh)&((1ULL<<n)-1);
}
static unsigned rdbits(GB *g,int n){ unsigned v=peek(g,n); g->index+=n; if(g->index>g->size_in_bits)g->index=g->size_in_bits; return v; }
static unsigned rdbit1(GB *g){ return rdbits(g,1); }
static unsigned ue(GB *g){ int lz=0; while(rdbit1(g)==0) lz++; if(!lz) return 0; return (1u<<lz)-1+rdbits(g,lz); }
static int se(GB *g){ int k=ue(g); return ((k+1)>>1)*(k&1?1:-1); }
#define av_log2(x) ((x) ? 31 - __builtin_clz(x) : 0)

static const uint8_t CT_LEN[272] = {1, 0, 0, 0, 6, 2, 0, 0, 8, 6, 3, 0, 9, 8, 7, 5, 10, 9, 8, 6, 11, 10, 9, 7, 13, 11, 10, 8, 13, 13, 11, 9, 13, 13, 13, 10, 14, 14, 13, 11, 14, 14, 14, 13, 15, 15, 14, 14, 15, 15, 15, 14, 16, 15, 15, 15, 16, 16, 16, 15, 16, 16, 16, 16, 16, 16, 16, 16, 2, 0, 0, 0, 6, 2, 0, 0, 6, 5, 3, 0, 7, 6, 6, 4, 8, 6, 6, 4, 8, 7, 7, 5, 9, 8, 8, 6, 11, 9, 9, 6, 11, 11, 11, 7, 12, 11, 11, 9, 12, 12, 12, 11, 12, 12, 12, 11, 13, 13, 13, 12, 13, 13, 13, 13, 13, 14, 13, 13, 14, 14, 14, 13, 14, 14, 14, 14, 4, 0, 0, 0, 6, 4, 0, 0, 6, 5, 4, 0, 6, 5, 5, 4, 7, 5, 5, 4, 7, 5, 5, 4, 7, 6, 6, 4, 7, 6, 6, 4, 8, 7, 7, 5, 8, 8, 7, 6, 9, 8, 8, 7, 9, 9, 8, 8, 9, 9, 9, 8, 10, 9, 9, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 6, 0, 0, 0, 6, 6, 0, 0, 6, 6, 6, 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6};
static const uint8_t CT_BITS[272] = {1, 0, 0, 0, 5, 1, 0, 0, 7, 4, 1, 0, 7, 6, 5, 3, 7, 6, 5, 3, 7, 6, 5, 4, 15, 6, 5, 4, 11, 14, 5, 4, 8, 10, 13, 4, 15, 14, 9, 4, 11, 10, 13, 12, 15, 14, 9, 12, 11, 10, 13, 8, 15, 1, 9, 12, 11, 14, 13, 8, 7, 10, 9, 12, 4, 6, 5, 8, 3, 0, 0, 0, 11, 2, 0, 0, 7, 7, 3, 0, 7, 10, 9, 5, 7, 6, 5, 4, 4, 6, 5, 6, 7, 6, 5, 8, 15, 6, 5, 4, 11, 14, 13, 4, 15, 10, 9, 4, 11, 14, 13, 12, 8, 10, 9, 8, 15, 14, 13, 12, 11, 10, 9, 12, 7, 11, 6, 8, 9, 8, 10, 1, 7, 6, 5, 4, 15, 0, 0, 0, 15, 14, 0, 0, 11, 15, 13, 0, 8, 12, 14, 12, 15, 10, 11, 11, 11, 8, 9, 10, 9, 14, 13, 9, 8, 10, 9, 8, 15, 14, 13, 13, 11, 14, 10, 12, 15, 10, 13, 12, 11, 14, 9, 12, 8, 10, 13, 8, 13, 7, 9, 12, 9, 12, 11, 10, 5, 8, 7, 6, 1, 4, 3, 2, 3, 0, 0, 0, 0, 1, 0, 0, 4, 5, 6, 0, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63};
static const uint8_t CDT_LEN[20] = {2, 0, 0, 0, 6, 1, 0, 0, 6, 6, 3, 0, 6, 7, 7, 6, 6, 8, 8, 7};
static const uint8_t CDT_BITS[20] = {1, 0, 0, 0, 7, 1, 0, 0, 4, 6, 1, 0, 3, 3, 2, 5, 2, 3, 2, 0};
static const uint8_t TZ_LEN[256] = {1, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 9, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 6, 6, 6, 6, 0, 4, 3, 3, 3, 4, 4, 3, 3, 4, 5, 5, 6, 5, 6, 0, 0, 5, 3, 4, 4, 3, 3, 3, 4, 3, 4, 5, 5, 5, 0, 0, 0, 4, 4, 4, 3, 3, 3, 3, 3, 4, 5, 4, 5, 0, 0, 0, 0, 6, 5, 3, 3, 3, 3, 3, 3, 4, 3, 6, 0, 0, 0, 0, 0, 6, 5, 3, 3, 3, 2, 3, 4, 3, 6, 0, 0, 0, 0, 0, 0, 6, 4, 5, 3, 2, 2, 3, 3, 6, 0, 0, 0, 0, 0, 0, 0, 6, 6, 4, 2, 2, 3, 2, 5, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 3, 2, 2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 3, 3, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 2, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
static const uint8_t TZ_BITS[256] = {1, 3, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3, 2, 1, 7, 6, 5, 4, 3, 5, 4, 3, 2, 3, 2, 3, 2, 1, 0, 0, 5, 7, 6, 5, 4, 3, 4, 3, 2, 3, 2, 1, 1, 0, 0, 0, 3, 7, 5, 4, 6, 5, 4, 3, 3, 2, 2, 1, 0, 0, 0, 0, 5, 4, 3, 7, 6, 5, 4, 3, 2, 1, 1, 0, 0, 0, 0, 0, 1, 1, 7, 6, 5, 4, 3, 2, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 5, 4, 3, 3, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 3, 3, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 3, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 3, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
static const uint8_t CDTZ_LEN[12] = {1, 2, 3, 3, 1, 2, 2, 0, 1, 1, 0, 0};
static const uint8_t CDTZ_BITS[12] = {1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0};
static const uint8_t RUN_LEN[112] = {1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0};
static const uint8_t RUN_BITS[112] = {1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 3, 2, 5, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0};

#define LEVEL_TAB_BITS 8
static int16_t level_tab[7][1<<LEVEL_TAB_BITS][2];
static void init_level_tab(void){
    for(int sl=0; sl<7; sl++) for(unsigned i=0; i<(1u<<LEVEL_TAB_BITS); i++){
        int prefix = LEVEL_TAB_BITS - av_log2(2*i);
        if(prefix + 1 + sl <= LEVEL_TAB_BITS){
            int lc = (prefix<<sl) + (i >> (av_log2(i)-sl)) - (1<<sl);
            int mask = -(lc&1);
            lc = (((2+lc)>>1)^mask)-mask;
            level_tab[sl][i][0]=lc; level_tab[sl][i][1]=prefix+1+sl;
        } else if(prefix+1 <= LEVEL_TAB_BITS){
            level_tab[sl][i][0]=prefix+100; level_tab[sl][i][1]=prefix+1;
        } else { level_tab[sl][i][0]=LEVEL_TAB_BITS+100; level_tab[sl][i][1]=LEVEL_TAB_BITS; }
    }
}
static int get_level_prefix(GB *gb){
    int cnt = 0;
    while (rdbit1(gb) == 0) { cnt++; if (cnt > 30) break; }
    return cnt;
}
static int vlc_sym(GB *g, const uint8_t *lens, const uint8_t *codes, int n, int maxlen){
    int code=0, l=0;
    for(int i=0;i<maxlen+1;i++){
        l++; code=(code<<1)|rdbit1(g);
        for(int k=0;k<n;k++) if(lens[k]==l && codes[k]==code) return k;
    }
    return -1;
}
static const int scan8[51] = {
    4+1*8,5+1*8,4+2*8,5+2*8, 6+1*8,7+1*8,6+2*8,7+2*8,
    4+3*8,5+3*8,4+4*8,5+4*8, 6+3*8,7+3*8,6+4*8,7+4*8,
    4+6*8,5+6*8,4+7*8,5+7*8, 6+6*8,7+6*8,6+7*8,7+7*8,
    4+8*8,5+8*8,4+9*8,5+9*8, 6+8*8,7+8*8,6+9*8,7+9*8,
    4+11*8,5+11*8,4+12*8,5+12*8, 6+11*8,7+11*8,6+12*8,7+12*8,
    4+13*8,5+13*8,4+14*8,5+14*8, 6+13*8,7+13*8,6+14*8,7+14*8,
    0+0*8, 0+5*8, 0+10*8
};
static const uint8_t zigzag[16] = {0,1,4,8,5,2,3,6,9,12,13,10,7,11,14,15};
static uint8_t nnz_cache[15*8];
#define LUMA_DC 48


static int decode_residual(GB *gb, int16_t *block, int n, uint32_t qmul, int max_coeff)
{
    int level[16];
    int zeros_left, coeff_token, total_coeff, i;
    (void)block; (void)qmul;
    if(max_coeff <= 8){
        coeff_token = vlc_sym(gb, CDT_LEN, CDT_BITS, 20, 8);
    } else {
        int np = n >= LUMA_DC ? (n - LUMA_DC) * 16 : n;
        int index8 = scan8[np];
        int left = nnz_cache[index8-1], top = nnz_cache[index8-8];
        int idx2 = left+top; if(idx2<64) idx2=(idx2+1)>>1;
        int pred = idx2 & 31;
        int bucket = pred<2?0: pred<4?1: pred<8?2:3;
        if(n<3||n==12) printf("  CTOK n=%d pred=%d b=%d bits=%016b\\n", n, pred, bucket, peek(gb,16));
        coeff_token = vlc_sym(gb, CT_LEN + bucket*68, CT_BITS + bucket*68, 68, 16);
    }
    total_coeff = coeff_token >> 2;
    nnz_cache[scan8[n]] = total_coeff;
    if(total_coeff==0){ printf("  n=%d tc=0 pos=%d\n", n, gb->index); return 0; }
    if(total_coeff > max_coeff){ printf("  n=%d TC>max\n", n); return -1; }
    int trailing_ones = coeff_token & 3;
    i = peek(gb,3);
    gb->index += trailing_ones;
    level[0] = 1-((i&4)>>1);
    level[1] = 1-(i&2);
    level[2] = 1-((i&1)<<1);
    if(trailing_ones<total_coeff) {
        int mask, prefix;
        int suffix_length = total_coeff > 10 & trailing_ones < 3;
        int bitsi = peek(gb, LEVEL_TAB_BITS);
        int level_code = level_tab[suffix_length][bitsi][0];
        gb->index += level_tab[suffix_length][bitsi][1];
        if(level_code >= 100){
            prefix = level_code - 100;
            if(prefix == LEVEL_TAB_BITS) prefix += get_level_prefix(gb);
            if(prefix<14){
                if(suffix_length) level_code = (prefix<<1) + rdbit1(gb);
                else level_code = prefix;
            } else if(prefix==14){
                if(suffix_length) level_code = (prefix<<1) + rdbit1(gb);
                else level_code = prefix + rdbits(gb,4);
            } else {
                level_code = 30;
                if(prefix>=16){
                    if(prefix > 25+3){ printf("esc ovf\n"); return -1; }
                    level_code += (1<<(prefix-3))-4096;
                }
                level_code += rdbits(gb, prefix-3);
            }
            if(trailing_ones < 3) level_code += 2;
            suffix_length = 2;
            mask = -(level_code&1);
            level[trailing_ones] = (((2+level_code)>>1)^mask)-mask;
        } else {
            level_code += ((level_code>>31)|1) & -(trailing_ones < 3);
            suffix_length = 1 + (level_code + 3U > 6U);
            level[trailing_ones] = level_code;
        }
        for(i=trailing_ones+1;i<total_coeff;i++) {
            static const unsigned int suffix_limit[7] = {0,3,6,12,24,48,(unsigned)-1};
            int bitsi = peek(gb, LEVEL_TAB_BITS);
            int level_code = level_tab[suffix_length][bitsi][0];
            gb->index += level_tab[suffix_length][bitsi][1];
            if(n==1) printf("  RL i=%d sl=%d bitsi=%d code=%d pos=%d\n", i, suffix_length, bitsi, level_code, gb->index);
            if(level_code >= 100){
                int prefix = level_code - 100;
                if(prefix == LEVEL_TAB_BITS) prefix += get_level_prefix(gb);
                if(n==11) printf("  RESC i=%d prefix=%d\n", i, prefix);
                if(prefix<15){
                    level_code = (prefix<<suffix_length) + rdbits(gb, suffix_length);
                } else {
                    level_code = 15<<suffix_length;
                    if(prefix>=16){
                        if(prefix > 25+3){ printf("esc ovf2\n"); return -1; }
                        level_code += (1<<(prefix-3))-4096;
                    }
                    level_code += rdbits(gb, prefix-3);
                }
                mask = -(level_code&1);
                level_code = (((2+level_code)>>1)^mask)-mask;
            }
            level[i] = level_code;
            suffix_length += suffix_limit[suffix_length] + level_code > 2U*suffix_limit[suffix_length];
        }
    }
    if(total_coeff == max_coeff) zeros_left=0;
    else{
        if (max_coeff <= 8)
            zeros_left = vlc_sym(gb, CDTZ_LEN + (total_coeff-1)*4, CDTZ_BITS + (total_coeff-1)*4, 4, 3);
        else
            zeros_left = vlc_sym(gb, TZ_LEN + (total_coeff-1)*16, TZ_BITS + (total_coeff-1)*16, 16, 9);
    }
    printf("  n=%d tc=%d to=%d zl=%d pos=%d\n", n, total_coeff, trailing_ones, zeros_left, gb->index);
    {
        int pos = zeros_left + total_coeff - 1;
        if(n==0) printf("  STOR lvl0=%d at scan[%d]\n", level[0], zigzag[pos]);
        int zi = zeros_left;
        for(i=1;i<total_coeff && zi > 0;i++) {
            int run;
            if(zi < 7)
                run = vlc_sym(gb, RUN_LEN + (zi-1)*16, RUN_BITS + (zi-1)*16, 7, 3);
            else
                run = vlc_sym(gb, RUN_LEN + 6*16, RUN_BITS + 6*16, 7, 6);
            if(n==1) printf("  CRUN zi=%d run=%d pos=%d\n", zi, run, gb->index);
            pos -= 1 + run;
            if(n==0) printf("  COEF i=%d lvl=%d at scan[%d]\n", i, level[i], zigzag[pos]);
            zi -= run;
        }
        if(zi<0){ printf("NEGZ\n"); return -1; }
    }
    return 0;
}


int main(int argc, char **argv){
    FILE *f = fopen(argv[1], "rb");
    static uint8_t file[1<<20]; int n = fread(file,1,sizeof file,f); fclose(f);
    static uint8_t nnz_mb[16][48];

    // iterate NALs
    int i = 0;
    while(i + 3 < n){
        if(!(file[i]==0 && file[i+1]==0 && file[i+2]==1)){ i++; continue; }
        int type = file[i+3] & 0x1f;
        if(type != 1 && type != 5){ i += 3; continue; }
        // SEI bodies can contain 000001-looking junk; validate the slice
        // header before trusting a "slice" NAL found mid-SEI.
        {
            GB v = { file + i + 4, 0, (n - i - 4) * 8 };
            unsigned fmb = ue(&v), st = ue(&v), pps = ue(&v);
            (void)pps;
            if(st > 9 || fmb > 1000 || v.index > 40){ i++; continue; }
        }
        int j = i + 4;
        static uint8_t rb[1<<16]; int m = 0;
        while(j + 2 < n){
            if(file[j]==0 && file[j+1]==0 && file[j+2]==1) break;
            if(file[j]==0 && file[j+1]==0 && file[j+2]==3){ rb[m++]=0; rb[m++]=0; j+=3; continue; }
            rb[m++]=file[j++];
        }
        while(m && rb[m-1]==0) m--;
        GB g = { rb, 0, m*8 };
        init_level_tab();
        memset(nnz_cache, 64, sizeof nnz_cache);
        // slice header
        unsigned first_mb = ue(&g);
        unsigned slice_type = ue(&g); if(slice_type > 4) slice_type -= 5;
        ue(&g); // pps_id
        rdbits(&g, 4); // frame_num
        if(type == 5) ue(&g); // idr_pic_id
        rdbits(&g, 4); // poc_lsb (poc type 0, log2 4)
        if(slice_type != 2){
            if(rdbit1(&g)) ue(&g);                 // ref idx override (l0 only for P)
            if(rdbit1(&g))                          // reordering
                for(;;){ unsigned op = ue(&g); if(op == 3) break; ue(&g); }
        }
        if((file[i+3] >> 5) != 0){                  // nal_ref_idc
            if(type == 5){ rdbit1(&g); rdbit1(&g); }
            else rdbit1(&g);                        // adaptive marking flag (no MMCO parsed)
        }
        int idc = ue(&g); if(idc<2 && (idc^1)){ se(&g); se(&g); }
        int qp0 = se(&g);
        printf("SLICE type=%d first_mb=%u pos=%d rb0=%02x rb1=%02x\n", slice_type, first_mb, g.index, rb[0], rb[1]);
        (void)qp0;

        static const uint8_t I4CBP[48] = {47,31,15,0,23,27,29,30,7,11,13,14,39,43,45,46,
                                           16,3,5,10,12,19,21,26,28,35,37,42,44,20,34,18,
                                           33,17,36,9,24,32,25,22,38,41,8,40,2,6,1,4};
        int skip_run = -1;
        for(int mb=first_mb; mb<48; mb++){
            if(slice_type != 2){
                if(skip_run == -1) skip_run = ue(&g);
                int run = skip_run; skip_run--;
                if(run > 0){
                    printf("  P%d SKIP pos=%d\n", mb, g.index);
                    // write back zero nnz
                    for(int r=0;r<4;r++) memset(&nnz_mb[mb%16][4*r], 0, 4);
                    memset(&nnz_mb[mb%16][16],0,4); memset(&nnz_mb[mb%16][20],0,4);
                    memset(&nnz_mb[mb%16][32],0,4); memset(&nnz_mb[mb%16][36],0,4);
                    continue;
                }
            }
            // neighbor borders
            int my = mb/8, mx = mb%8;
            if(mx > 0){
                int lm = mb-1;
                nnz_cache[3+8*1]=nnz_mb[lm%16][3]; nnz_cache[3+8*2]=nnz_mb[lm%16][7];
                nnz_cache[3+8*3]=nnz_mb[lm%16][11]; nnz_cache[3+8*4]=nnz_mb[lm%16][15];
                nnz_cache[3+8*6]=nnz_mb[lm%16][17]; nnz_cache[3+8*7]=nnz_mb[lm%16][21];
                nnz_cache[3+8*11]=nnz_mb[lm%16][33]; nnz_cache[3+8*12]=nnz_mb[lm%16][37];
            } else {
                nnz_cache[3+8*1]=nnz_cache[3+8*2]=nnz_cache[3+8*3]=nnz_cache[3+8*4]=64;
                nnz_cache[3+8*6]=nnz_cache[3+8*7]=64;
                nnz_cache[3+8*11]=nnz_cache[3+8*12]=64;
            }
            if(my > 0){
                int tm = mb-8;
                for(int c=0;c<4;c++){
                    nnz_cache[4+8*0+c]=nnz_mb[tm%16][12+c];
                    nnz_cache[4+8*5+c]=nnz_mb[tm%16][20+c];
                    nnz_cache[4+8*10+c]=nnz_mb[tm%16][36+c];
                }
            } else {
                for(int c=0;c<4;c++){
                    nnz_cache[4+8*0+c]=64; nnz_cache[4+8*5+c]=64; nnz_cache[4+8*10+c]=64;
                }
            }
            int mbt = ue(&g);
            printf("  P%d mbt=%d pos=%d\n", mb, mbt, g.index);
            int cp, cbp;
            if(mbt == 25){ // PCM in I
                g.index = (g.index+7)&~7; g.index += 384*8;
                memset(&nnz_mb[mb%16][0],16,48);
                continue;
            }
            if(slice_type == 2 || mbt >= 5){
                int row = slice_type == 2 ? mbt : mbt-5;
                if(row > 25) { printf("  bad intra row\n"); return 1; }
                if(row == 0){
                    for(int k=0;k<16;k++) if(!rdbit1(&g)) rdbits(&g,3);
                    cp = ue(&g);
                    cbp = I4CBP[ue(&g)];
                } else {
                    int grp = (row-1)/4;
                    static const int CB[6] = {0,16,32,15,31,47};
                    cp = -1; cbp = CB[grp]; (void)cp;
                }
                if(mbt >= 1){
                    ue(&g); // intra chroma pred (every 16x16 reads it)
                    se(&g);
                    if(decode_residual(&g, NULL, 48, 1, 16) < 0) return 1;
                    if(cbp & 0xf)
                        for(int k=0;k<16;k++)
                            if(decode_residual(&g, NULL, k, 1, 15) < 0) return 1;
                } else if(cbp & 0xf){
                    se(&g);
                    for(int i8=0;i8<4;i8++){
                        if(cbp & (1<<i8)){
                            for(int i4=0;i4<4;i4++)
                                if(decode_residual(&g, NULL, i4+4*i8, 1, 16) < 0) return 1;
                        } else {
                            for(int i4=0;i4<4;i4++) nnz_cache[scan8[4*i8+i4]] = 0;
                        }
                    }
                } else {
                    for(int k=0;k<16;k++) nnz_cache[scan8[k]] = 0;
                }
            } else {
                // P inter
                int part = mbt;
                if(part == 0){ se(&g); se(&g); }
                else if(part == 1 || part == 2){ se(&g); se(&g); se(&g); se(&g); }
                else {
                    for(int s=0;s<4;s++){
                        unsigned st = ue(&g);
                        if(st > 3){ printf("  SUB bad st=%u pos=%d\n", st, g.index); return 1; }
                        int cnt = st==0?1: st==3?4:2;
                        for(int k=0;k<cnt;k++){ se(&g); se(&g); }
                    }
                }
                cbp = ue(&g);
                printf("  P%d cbp=%d pos=%d\n", mb, cbp, g.index);
                if(cbp > 47){ printf("  cbp big\n"); return 1; }
                if(cbp & 0xf){
                    se(&g);
                    for(int i8=0;i8<4;i8++){
                        if(cbp & (1<<i8)){
                            for(int i4=0;i4<4;i4++)
                                if(decode_residual(&g, NULL, i4+4*i8, 1, 16) < 0) return 1;
                        } else {
                            for(int i4=0;i4<4;i4++) nnz_cache[scan8[4*i8+i4]] = 0;
                        }
                    }
                } else {
                    for(int k=0;k<16;k++) nnz_cache[scan8[k]] = 0;
                }
            }
            if(cbp & 0x30){
                static int16_t s1[16], s2[16];
                if(decode_residual(&g, s1, 49, 1, 4) < 0) return 1;
                if(decode_residual(&g, s2, 50, 1, 4) < 0) return 1;
            }
            if(cbp & 0x20){
                for(int ch=0; ch<2; ch++)
                    for(int b=0;b<4;b++){
                        static int16_t sc[16];
                        if(decode_residual(&g, sc, 16+16*ch+b, 1, 15) < 0) return 1;
                    }
            }
            for(int r=0;r<4;r++) memcpy(&nnz_mb[mb%16][4*r], &nnz_cache[4+8*(r+1)], 4);
            memcpy(&nnz_mb[mb%16][16], &nnz_cache[4+8*6], 4);
            memcpy(&nnz_mb[mb%16][20], &nnz_cache[4+8*7], 4);
            memcpy(&nnz_mb[mb%16][32], &nnz_cache[4+8*11], 4);
            memcpy(&nnz_mb[mb%16][36], &nnz_cache[4+8*12], 4);
        }
        i = j;
    }
    return 0;
}
