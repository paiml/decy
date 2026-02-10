//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1351-C1375: Codec and Media Processing patterns -- the kind of C code found
//! in FFmpeg, libavcodec, GStreamer, and multimedia processing libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world codec and media processing patterns commonly
//! found in audio/video encoders, image format libraries, and container parsers
//! -- all expressed as valid C99 with inline type definitions (no #include).
//!
//! Organization:
//! - C1351-C1355: Basic codecs (RLE encoder, RLE decoder, delta encoding, varint, base64)
//! - C1356-C1360: Audio codecs (PCM converter, mu-law, ADPCM, mixer, resampler)
//! - C1361-C1365: Image codecs (BMP header, PPM encoder, pixel format, RGB-YUV, dithering)
//! - C1366-C1370: Video primitives (motion vector, SAD, frame diff, deinterlace, chroma)
//! - C1371-C1375: Container formats (RIFF chunk, WAVE header, AVI index, MP4 box, FLV tag)
//!
//! ## Results
//! - 25 passing, 0 falsified (100.0% pass rate)

use decy_core::transpile;

// ============================================================================
// C1351-C1355: Basic Codecs (RLE, delta, varint, base64)
// ============================================================================

/// C1351: RLE encoder -- run-length encodes byte stream into (count, value) pairs
#[test]
fn c1351_rle_encoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

typedef struct { uint8_t *out; size_t pos; size_t cap; } codec_rle_enc_t;

void codec_rle_enc_init(codec_rle_enc_t *c, uint8_t *buf, size_t cap) {
    c->out = buf; c->pos = 0; c->cap = cap;
}

void codec_rle_enc_put(codec_rle_enc_t *c, uint8_t val, int count) {
    if (c->pos + 2 <= c->cap) {
        c->out[c->pos++] = (uint8_t)count;
        c->out[c->pos++] = val;
    }
}

size_t codec_rle_enc_flush(codec_rle_enc_t *c) {
    return c->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1351 failed: {:?}", result.err());
}

/// C1352: RLE decoder -- decodes run-length encoded stream back to raw bytes
#[test]
fn c1352_rle_decoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

typedef struct { const uint8_t *src; size_t pos; size_t len; } codec_rle_dec_t;

void codec_rle_dec_init(codec_rle_dec_t *d, const uint8_t *data, size_t len) {
    d->src = data; d->pos = 0; d->len = len;
}

int codec_rle_dec_read(codec_rle_dec_t *d, uint8_t *out, size_t out_cap) {
    size_t wp = 0;
    while (d->pos + 1 < d->len && wp < out_cap) {
        int count = d->src[d->pos];
        uint8_t val = d->src[d->pos + 1];
        int i;
        for (i = 0; i < count && wp < out_cap; i++) {
            out[wp++] = val;
        }
        d->pos += 2;
    }
    return (int)wp;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1352 failed: {:?}", result.err());
}

/// C1353: Delta encoding -- stores differences between consecutive samples
#[test]
fn c1353_delta_encoding() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct { int prev; } codec_delta_t;

void codec_delta_init(codec_delta_t *d) { d->prev = 0; }

void codec_delta_encode(codec_delta_t *d, const int *in, int *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        out[i] = in[i] - d->prev;
        d->prev = in[i];
    }
}

void codec_delta_decode(codec_delta_t *d, const int *in, int *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        d->prev += in[i];
        out[i] = d->prev;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1353 failed: {:?}", result.err());
}

/// C1354: Variable-length integer (varint) -- encodes integers with 7-bit groups
#[test]
fn c1354_varint_codec() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

int codec_varint_encode(uint32_t val, uint8_t *buf, size_t cap) {
    int pos = 0;
    while (val > 0x7F && pos < (int)cap) {
        buf[pos++] = (uint8_t)(val & 0x7F) | 0x80;
        val >>= 7;
    }
    if (pos < (int)cap) {
        buf[pos++] = (uint8_t)(val & 0x7F);
    }
    return pos;
}

uint32_t codec_varint_decode(const uint8_t *buf, size_t len, int *bytes_read) {
    uint32_t result = 0;
    int shift = 0;
    int i;
    for (i = 0; i < (int)len && i < 5; i++) {
        result |= (uint32_t)(buf[i] & 0x7F) << shift;
        shift += 7;
        if (!(buf[i] & 0x80)) { i++; break; }
    }
    *bytes_read = i;
    return result;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1354 failed: {:?}", result.err());
}

/// C1355: Base64 encode/decode -- converts binary to/from ASCII-safe representation
#[test]
fn c1355_base64_codec() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

static const char codec_b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

int codec_b64_encode(const uint8_t *src, size_t slen, char *dst, size_t dlen) {
    size_t i, j = 0;
    for (i = 0; i + 2 < slen && j + 3 < dlen; i += 3) {
        uint8_t a = src[i], b = src[i+1], c = src[i+2];
        dst[j++] = codec_b64_table[a >> 2];
        dst[j++] = codec_b64_table[((a & 3) << 4) | (b >> 4)];
        dst[j++] = codec_b64_table[((b & 0x0F) << 2) | (c >> 6)];
        dst[j++] = codec_b64_table[c & 0x3F];
    }
    if (j < dlen) dst[j] = '\0';
    return (int)j;
}

int codec_b64_char_val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1355 failed: {:?}", result.err());
}

// ============================================================================
// C1356-C1360: Audio Codecs (PCM, mu-law, ADPCM, mixer, resampler)
// ============================================================================

/// C1356: PCM converter -- converts between 8-bit and 16-bit PCM audio samples
#[test]
fn c1356_pcm_converter() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef short int16_t;

void codec_pcm_8to16(const uint8_t *in, int16_t *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        out[i] = (int16_t)((in[i] - 128) << 8);
    }
}

void codec_pcm_16to8(const int16_t *in, uint8_t *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        out[i] = (uint8_t)((in[i] >> 8) + 128);
    }
}

void codec_pcm_normalize(int16_t *buf, size_t n) {
    int16_t peak = 0;
    size_t i;
    for (i = 0; i < n; i++) {
        int16_t v = buf[i] < 0 ? -buf[i] : buf[i];
        if (v > peak) peak = v;
    }
    if (peak > 0) {
        for (i = 0; i < n; i++) {
            buf[i] = (int16_t)((int)buf[i] * 32767 / peak);
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1356 failed: {:?}", result.err());
}

/// C1357: Mu-law compander -- ITU-T G.711 mu-law compression for voice telephony
#[test]
fn c1357_mulaw_compander() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef short int16_t;
typedef unsigned long size_t;

static const int CODEC_MULAW_BIAS = 132;
static const int CODEC_MULAW_MAX = 32635;

uint8_t codec_mulaw_encode_sample(int16_t sample) {
    int sign = 0;
    int exponent, mantissa;
    uint8_t encoded;
    int s = (int)sample;
    if (s < 0) { sign = 0x80; s = -s; }
    if (s > CODEC_MULAW_MAX) s = CODEC_MULAW_MAX;
    s += CODEC_MULAW_BIAS;
    exponent = 7;
    while (exponent > 0 && !(s & (1 << (exponent + 3)))) exponent--;
    mantissa = (s >> (exponent + 3)) & 0x0F;
    encoded = ~(sign | (exponent << 4) | mantissa);
    return encoded;
}

void codec_mulaw_encode_buf(const int16_t *in, uint8_t *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        out[i] = codec_mulaw_encode_sample(in[i]);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1357 failed: {:?}", result.err());
}

/// C1358: ADPCM encoder -- adaptive differential PCM for audio compression
#[test]
fn c1358_adpcm_encoder() {
    let c_code = r#"
typedef short int16_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

typedef struct {
    int16_t predicted;
    int step_index;
} codec_adpcm_state_t;

static const int codec_adpcm_steps[] = {7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31};

void codec_adpcm_init(codec_adpcm_state_t *st) {
    st->predicted = 0; st->step_index = 0;
}

uint8_t codec_adpcm_encode_sample(codec_adpcm_state_t *st, int16_t sample) {
    int diff = sample - st->predicted;
    uint8_t nibble = 0;
    int step = codec_adpcm_steps[st->step_index & 0x0F];
    if (diff < 0) { nibble = 8; diff = -diff; }
    if (diff >= step) { nibble |= 4; diff -= step; }
    if (diff >= step / 2) { nibble |= 2; diff -= step / 2; }
    if (diff >= step / 4) { nibble |= 1; }
    st->step_index += (nibble & 7) - 3;
    if (st->step_index < 0) st->step_index = 0;
    if (st->step_index > 15) st->step_index = 15;
    return nibble;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1358 failed: {:?}", result.err());
}

/// C1359: Simple mixer -- mixes two audio streams with volume control
#[test]
fn c1359_audio_mixer() {
    let c_code = r#"
typedef short int16_t;
typedef unsigned long size_t;

typedef struct {
    int vol_a;
    int vol_b;
} codec_mixer_t;

void codec_mixer_init(codec_mixer_t *m, int vol_a, int vol_b) {
    m->vol_a = vol_a; m->vol_b = vol_b;
}

void codec_mixer_mix(const codec_mixer_t *m, const int16_t *a, const int16_t *b, int16_t *out, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        int mixed = ((int)a[i] * m->vol_a + (int)b[i] * m->vol_b) / 256;
        if (mixed > 32767) mixed = 32767;
        if (mixed < -32768) mixed = -32768;
        out[i] = (int16_t)mixed;
    }
}

void codec_mixer_fade(int16_t *buf, size_t n, int start_vol, int end_vol) {
    size_t i;
    for (i = 0; i < n; i++) {
        int vol = start_vol + (int)((long)(end_vol - start_vol) * (long)i / (long)n);
        buf[i] = (int16_t)((int)buf[i] * vol / 256);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1359 failed: {:?}", result.err());
}

/// C1360: Audio resampler -- nearest-neighbor sample rate conversion
#[test]
fn c1360_audio_resampler() {
    let c_code = r#"
typedef short int16_t;
typedef unsigned long size_t;

typedef struct {
    int src_rate;
    int dst_rate;
} codec_resamp_t;

void codec_resamp_init(codec_resamp_t *r, int src_rate, int dst_rate) {
    r->src_rate = src_rate; r->dst_rate = dst_rate;
}

int codec_resamp_nearest(const codec_resamp_t *r, const int16_t *in, size_t in_n, int16_t *out, size_t out_cap) {
    size_t i;
    size_t out_n = in_n * (size_t)r->dst_rate / (size_t)r->src_rate;
    if (out_n > out_cap) out_n = out_cap;
    for (i = 0; i < out_n; i++) {
        size_t src_idx = i * (size_t)r->src_rate / (size_t)r->dst_rate;
        if (src_idx >= in_n) src_idx = in_n - 1;
        out[i] = in[src_idx];
    }
    return (int)out_n;
}

int codec_resamp_linear(const codec_resamp_t *r, const int16_t *in, size_t in_n, int16_t *out, size_t out_cap) {
    size_t i;
    size_t out_n = in_n * (size_t)r->dst_rate / (size_t)r->src_rate;
    if (out_n > out_cap) out_n = out_cap;
    for (i = 0; i < out_n; i++) {
        long pos = (long)i * r->src_rate;
        size_t idx = (size_t)(pos / r->dst_rate);
        if (idx + 1 < in_n) {
            int frac = (int)(pos % r->dst_rate);
            out[i] = (int16_t)((int)in[idx] * (r->dst_rate - frac) / r->dst_rate + (int)in[idx+1] * frac / r->dst_rate);
        } else {
            out[i] = in[idx < in_n ? idx : in_n - 1];
        }
    }
    return (int)out_n;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1360 failed: {:?}", result.err());
}

// ============================================================================
// C1361-C1365: Image Codecs (BMP, PPM, pixel format, color space, dithering)
// ============================================================================

/// C1361: BMP header writer -- writes a minimal Windows BMP file header
#[test]
fn c1361_bmp_header_writer() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t header[54];
    int width;
    int height;
} codec_bmp_t;

void codec_bmp_write16(uint8_t *p, uint16_t v) {
    p[0] = (uint8_t)(v & 0xFF);
    p[1] = (uint8_t)(v >> 8);
}

void codec_bmp_write32(uint8_t *p, uint32_t v) {
    p[0] = (uint8_t)(v & 0xFF);
    p[1] = (uint8_t)((v >> 8) & 0xFF);
    p[2] = (uint8_t)((v >> 16) & 0xFF);
    p[3] = (uint8_t)((v >> 24) & 0xFF);
}

void codec_bmp_init(codec_bmp_t *b, int w, int h) {
    int i;
    uint32_t row_size = (uint32_t)((w * 3 + 3) & ~3);
    uint32_t img_size = row_size * (uint32_t)h;
    for (i = 0; i < 54; i++) b->header[i] = 0;
    b->header[0] = 'B'; b->header[1] = 'M';
    codec_bmp_write32(b->header + 2, 54 + img_size);
    codec_bmp_write32(b->header + 10, 54);
    codec_bmp_write32(b->header + 14, 40);
    codec_bmp_write32(b->header + 18, (uint32_t)w);
    codec_bmp_write32(b->header + 22, (uint32_t)h);
    codec_bmp_write16(b->header + 26, 1);
    codec_bmp_write16(b->header + 28, 24);
    codec_bmp_write32(b->header + 34, img_size);
    b->width = w; b->height = h;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1361 failed: {:?}", result.err());
}

/// C1362: PPM encoder -- writes Netpbm PPM (P6) image format
#[test]
fn c1362_ppm_encoder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

typedef struct {
    int width;
    int height;
    uint8_t *pixels;
} codec_ppm_t;

void codec_ppm_init(codec_ppm_t *p, int w, int h, uint8_t *buf) {
    p->width = w; p->height = h; p->pixels = buf;
}

void codec_ppm_set_pixel(codec_ppm_t *p, int x, int y, uint8_t r, uint8_t g, uint8_t b) {
    if (x >= 0 && x < p->width && y >= 0 && y < p->height) {
        int off = (y * p->width + x) * 3;
        p->pixels[off] = r;
        p->pixels[off + 1] = g;
        p->pixels[off + 2] = b;
    }
}

int codec_ppm_header_len(const codec_ppm_t *p, char *hdr, size_t cap) {
    int len = 0;
    int w = p->width, h = p->height;
    hdr[len++] = 'P'; hdr[len++] = '6'; hdr[len++] = '\n';
    if (w >= 100 && len < (int)cap) hdr[len++] = '0' + (w / 100) % 10;
    if (w >= 10 && len < (int)cap) hdr[len++] = '0' + (w / 10) % 10;
    if (len < (int)cap) hdr[len++] = '0' + w % 10;
    if (len < (int)cap) hdr[len++] = ' ';
    if (h >= 100 && len < (int)cap) hdr[len++] = '0' + (h / 100) % 10;
    if (h >= 10 && len < (int)cap) hdr[len++] = '0' + (h / 10) % 10;
    if (len < (int)cap) hdr[len++] = '0' + h % 10;
    if (len < (int)cap) hdr[len++] = '\n';
    return len;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1362 failed: {:?}", result.err());
}

/// C1363: Pixel format converter -- converts between RGB, BGR, and RGBA pixel formats
#[test]
fn c1363_pixel_format_converter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

void codec_pix_rgb_to_bgr(const uint8_t *src, uint8_t *dst, size_t pixel_count) {
    size_t i;
    for (i = 0; i < pixel_count; i++) {
        dst[i*3 + 0] = src[i*3 + 2];
        dst[i*3 + 1] = src[i*3 + 1];
        dst[i*3 + 2] = src[i*3 + 0];
    }
}

void codec_pix_rgb_to_rgba(const uint8_t *src, uint8_t *dst, size_t pixel_count, uint8_t alpha) {
    size_t i;
    for (i = 0; i < pixel_count; i++) {
        dst[i*4 + 0] = src[i*3 + 0];
        dst[i*4 + 1] = src[i*3 + 1];
        dst[i*4 + 2] = src[i*3 + 2];
        dst[i*4 + 3] = alpha;
    }
}

void codec_pix_rgba_to_rgb(const uint8_t *src, uint8_t *dst, size_t pixel_count) {
    size_t i;
    for (i = 0; i < pixel_count; i++) {
        dst[i*3 + 0] = src[i*4 + 0];
        dst[i*3 + 1] = src[i*4 + 1];
        dst[i*3 + 2] = src[i*4 + 2];
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1363 failed: {:?}", result.err());
}

/// C1364: Color space converter -- converts between RGB and YUV (BT.601)
#[test]
fn c1364_rgb_yuv_converter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

static int codec_cs_clamp(int v) { return v < 0 ? 0 : (v > 255 ? 255 : v); }

void codec_cs_rgb_to_yuv(const uint8_t *rgb, uint8_t *yuv, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        int r = rgb[i*3], g = rgb[i*3+1], b = rgb[i*3+2];
        yuv[i*3]   = (uint8_t)codec_cs_clamp((66*r + 129*g + 25*b + 128) / 256 + 16);
        yuv[i*3+1] = (uint8_t)codec_cs_clamp((-38*r - 74*g + 112*b + 128) / 256 + 128);
        yuv[i*3+2] = (uint8_t)codec_cs_clamp((112*r - 94*g - 18*b + 128) / 256 + 128);
    }
}

void codec_cs_yuv_to_rgb(const uint8_t *yuv, uint8_t *rgb, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        int y = yuv[i*3] - 16, u = yuv[i*3+1] - 128, v = yuv[i*3+2] - 128;
        rgb[i*3]   = (uint8_t)codec_cs_clamp((298*y + 409*v + 128) / 256);
        rgb[i*3+1] = (uint8_t)codec_cs_clamp((298*y - 100*u - 208*v + 128) / 256);
        rgb[i*3+2] = (uint8_t)codec_cs_clamp((298*y + 516*u + 128) / 256);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1364 failed: {:?}", result.err());
}

/// C1365: Simple dithering -- Floyd-Steinberg error diffusion on grayscale image
#[test]
fn c1365_simple_dithering() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int width;
    int height;
} codec_dither_t;

void codec_dither_init(codec_dither_t *d, int w, int h) {
    d->width = w; d->height = h;
}

void codec_dither_fs(codec_dither_t *d, int *pixels) {
    int x, y;
    int w = d->width, h = d->height;
    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            int old = pixels[y * w + x];
            int nv = old < 128 ? 0 : 255;
            int err = old - nv;
            pixels[y * w + x] = nv;
            if (x + 1 < w)          pixels[y * w + x + 1] += err * 7 / 16;
            if (y + 1 < h && x > 0) pixels[(y+1) * w + x - 1] += err * 3 / 16;
            if (y + 1 < h)          pixels[(y+1) * w + x] += err * 5 / 16;
            if (y + 1 < h && x + 1 < w) pixels[(y+1) * w + x + 1] += err * 1 / 16;
        }
    }
}

void codec_dither_threshold(uint8_t *pixels, int n, uint8_t thresh) {
    int i;
    for (i = 0; i < n; i++) {
        pixels[i] = pixels[i] >= thresh ? 255 : 0;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1365 failed: {:?}", result.err());
}

// ============================================================================
// C1366-C1370: Video Primitives (motion vector, SAD, frame diff, deinterlace, chroma)
// ============================================================================

/// C1366: Motion vector search -- full search block matching for video compression
#[test]
fn c1366_motion_vector_search() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct { int x; int y; int cost; } codec_mv_t;

int codec_mv_sad_block(const uint8_t *cur, const uint8_t *ref, int stride, int bw, int bh, int rx, int ry) {
    int sad = 0, x, y;
    for (y = 0; y < bh; y++) {
        for (x = 0; x < bw; x++) {
            int a = cur[y * stride + x];
            int b = ref[(y + ry) * stride + (x + rx)];
            int d = a - b;
            sad += d < 0 ? -d : d;
        }
    }
    return sad;
}

codec_mv_t codec_mv_search(const uint8_t *cur, const uint8_t *ref, int stride, int bw, int bh, int range) {
    codec_mv_t best;
    int dx, dy;
    best.x = 0; best.y = 0; best.cost = 0x7FFFFFFF;
    for (dy = -range; dy <= range; dy++) {
        for (dx = -range; dx <= range; dx++) {
            int cost = codec_mv_sad_block(cur, ref, stride, bw, bh, dx, dy);
            if (cost < best.cost) {
                best.x = dx; best.y = dy; best.cost = cost;
            }
        }
    }
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1366 failed: {:?}", result.err());
}

/// C1367: Block difference (SAD) -- sum of absolute differences for 8x8 and 16x16 blocks
#[test]
fn c1367_block_sad() {
    let c_code = r#"
typedef unsigned char uint8_t;

int codec_sad_8x8(const uint8_t *a, const uint8_t *b, int stride) {
    int sum = 0, x, y;
    for (y = 0; y < 8; y++) {
        for (x = 0; x < 8; x++) {
            int d = (int)a[y * stride + x] - (int)b[y * stride + x];
            sum += d < 0 ? -d : d;
        }
    }
    return sum;
}

int codec_sad_16x16(const uint8_t *a, const uint8_t *b, int stride) {
    int sum = 0, x, y;
    for (y = 0; y < 16; y++) {
        for (x = 0; x < 16; x++) {
            int d = (int)a[y * stride + x] - (int)b[y * stride + x];
            sum += d < 0 ? -d : d;
        }
    }
    return sum;
}

int codec_sad_mean(const uint8_t *block, int stride, int bw, int bh) {
    int sum = 0, x, y;
    for (y = 0; y < bh; y++)
        for (x = 0; x < bw; x++)
            sum += block[y * stride + x];
    return sum / (bw * bh);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1367 failed: {:?}", result.err());
}

/// C1368: Frame differencing -- computes difference between consecutive video frames
#[test]
fn c1368_frame_differencing() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

void codec_frame_diff(const uint8_t *prev, const uint8_t *cur, uint8_t *diff, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        int d = (int)cur[i] - (int)prev[i];
        diff[i] = (uint8_t)(d < 0 ? -d : d);
    }
}

int codec_frame_changed(const uint8_t *diff, size_t n, int threshold) {
    size_t i;
    int count = 0;
    for (i = 0; i < n; i++) {
        if (diff[i] > (uint8_t)threshold) count++;
    }
    return count;
}

void codec_frame_blend(const uint8_t *a, const uint8_t *b, uint8_t *out, size_t n, int alpha) {
    size_t i;
    for (i = 0; i < n; i++) {
        out[i] = (uint8_t)(((int)a[i] * (256 - alpha) + (int)b[i] * alpha) / 256);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1368 failed: {:?}", result.err());
}

/// C1369: Simple deinterlace -- bob and weave deinterlacing for interlaced video
#[test]
fn c1369_simple_deinterlace() {
    let c_code = r#"
typedef unsigned char uint8_t;

void codec_deint_bob(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x, y;
    for (y = 0; y < h; y++) {
        int src_y = y / 2 * 2 + (y % 2 == 0 ? 0 : 0);
        for (x = 0; x < w; x++) {
            dst[y * w + x] = src[src_y * w + x];
        }
    }
}

void codec_deint_weave(const uint8_t *field_top, const uint8_t *field_bot, uint8_t *dst, int w, int h) {
    int x, y;
    for (y = 0; y < h; y++) {
        const uint8_t *src_row;
        if (y % 2 == 0) {
            src_row = field_top + (y / 2) * w;
        } else {
            src_row = field_bot + (y / 2) * w;
        }
        for (x = 0; x < w; x++) {
            dst[y * w + x] = src_row[x];
        }
    }
}

void codec_deint_avg(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x, y;
    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            if (y > 0 && y < h - 1) {
                dst[y*w+x] = (uint8_t)(((int)src[(y-1)*w+x] + (int)src[y*w+x] + (int)src[(y+1)*w+x]) / 3);
            } else {
                dst[y*w+x] = src[y*w+x];
            }
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1369 failed: {:?}", result.err());
}

/// C1370: Chroma subsampling -- converts between YUV 4:4:4 and 4:2:0 formats
#[test]
fn c1370_chroma_subsampling() {
    let c_code = r#"
typedef unsigned char uint8_t;

void codec_chroma_444to420(const uint8_t *u_in, const uint8_t *v_in,
                           uint8_t *u_out, uint8_t *v_out, int w, int h) {
    int x, y;
    int hw = w / 2, hh = h / 2;
    for (y = 0; y < hh; y++) {
        for (x = 0; x < hw; x++) {
            int y2 = y * 2, x2 = x * 2;
            u_out[y * hw + x] = (uint8_t)(((int)u_in[y2*w+x2] + (int)u_in[y2*w+x2+1] +
                                            (int)u_in[(y2+1)*w+x2] + (int)u_in[(y2+1)*w+x2+1]) / 4);
            v_out[y * hw + x] = (uint8_t)(((int)v_in[y2*w+x2] + (int)v_in[y2*w+x2+1] +
                                            (int)v_in[(y2+1)*w+x2] + (int)v_in[(y2+1)*w+x2+1]) / 4);
        }
    }
}

void codec_chroma_420to444(const uint8_t *u_in, const uint8_t *v_in,
                           uint8_t *u_out, uint8_t *v_out, int w, int h) {
    int x, y;
    int hw = w / 2;
    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            u_out[y * w + x] = u_in[(y/2) * hw + x/2];
            v_out[y * w + x] = v_in[(y/2) * hw + x/2];
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1370 failed: {:?}", result.err());
}

// ============================================================================
// C1371-C1375: Container Formats (RIFF, WAVE, AVI, MP4, FLV)
// ============================================================================

/// C1371: RIFF chunk writer -- writes RIFF/IFF-style chunks with size and FourCC
#[test]
fn c1371_riff_chunk_writer() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

typedef struct { uint8_t *buf; size_t pos; size_t cap; } codec_riff_t;

void codec_riff_init(codec_riff_t *r, uint8_t *buf, size_t cap) {
    r->buf = buf; r->pos = 0; r->cap = cap;
}

static void codec_riff_put32le(codec_riff_t *r, uint32_t v) {
    if (r->pos + 4 <= r->cap) {
        r->buf[r->pos++] = (uint8_t)(v & 0xFF);
        r->buf[r->pos++] = (uint8_t)((v >> 8) & 0xFF);
        r->buf[r->pos++] = (uint8_t)((v >> 16) & 0xFF);
        r->buf[r->pos++] = (uint8_t)((v >> 24) & 0xFF);
    }
}

void codec_riff_begin_chunk(codec_riff_t *r, const char *fourcc, uint32_t size) {
    int i;
    if (r->pos + 8 <= r->cap) {
        for (i = 0; i < 4; i++) r->buf[r->pos++] = (uint8_t)fourcc[i];
        codec_riff_put32le(r, size);
    }
}

size_t codec_riff_size(const codec_riff_t *r) { return r->pos; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1371 failed: {:?}", result.err());
}

/// C1372: WAVE header -- writes a PCM WAV file header (RIFF/WAVE format)
#[test]
fn c1372_wave_header() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct { uint8_t hdr[44]; } codec_wav_hdr_t;

static void codec_wav_w16(uint8_t *p, uint16_t v) {
    p[0] = (uint8_t)(v & 0xFF); p[1] = (uint8_t)(v >> 8);
}
static void codec_wav_w32(uint8_t *p, uint32_t v) {
    p[0] = (uint8_t)(v); p[1] = (uint8_t)(v >> 8);
    p[2] = (uint8_t)(v >> 16); p[3] = (uint8_t)(v >> 24);
}

void codec_wav_init(codec_wav_hdr_t *w, int channels, int sample_rate, int bits_per_sample, uint32_t data_size) {
    int i;
    uint16_t block_align = (uint16_t)(channels * bits_per_sample / 8);
    uint32_t byte_rate = (uint32_t)(sample_rate * (int)block_align);
    for (i = 0; i < 44; i++) w->hdr[i] = 0;
    w->hdr[0]='R'; w->hdr[1]='I'; w->hdr[2]='F'; w->hdr[3]='F';
    codec_wav_w32(w->hdr + 4, 36 + data_size);
    w->hdr[8]='W'; w->hdr[9]='A'; w->hdr[10]='V'; w->hdr[11]='E';
    w->hdr[12]='f'; w->hdr[13]='m'; w->hdr[14]='t'; w->hdr[15]=' ';
    codec_wav_w32(w->hdr + 16, 16);
    codec_wav_w16(w->hdr + 20, 1);
    codec_wav_w16(w->hdr + 22, (uint16_t)channels);
    codec_wav_w32(w->hdr + 24, (uint32_t)sample_rate);
    codec_wav_w32(w->hdr + 28, byte_rate);
    codec_wav_w16(w->hdr + 32, block_align);
    codec_wav_w16(w->hdr + 34, (uint16_t)bits_per_sample);
    w->hdr[36]='d'; w->hdr[37]='a'; w->hdr[38]='t'; w->hdr[39]='a';
    codec_wav_w32(w->hdr + 40, data_size);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1372 failed: {:?}", result.err());
}

/// C1373: AVI index -- builds AVI 1.0 index (idx1) entries for video frames
#[test]
fn c1373_avi_index() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

typedef struct { uint32_t flags; uint32_t offset; uint32_t size; } codec_avi_idx_entry_t;

typedef struct {
    codec_avi_idx_entry_t entries[256];
    int count;
} codec_avi_idx_t;

void codec_avi_idx_init(codec_avi_idx_t *idx) { idx->count = 0; }

int codec_avi_idx_add(codec_avi_idx_t *idx, uint32_t flags, uint32_t offset, uint32_t size) {
    if (idx->count >= 256) return -1;
    idx->entries[idx->count].flags = flags;
    idx->entries[idx->count].offset = offset;
    idx->entries[idx->count].size = size;
    idx->count++;
    return 0;
}

int codec_avi_idx_write(const codec_avi_idx_t *idx, uint8_t *buf, size_t cap) {
    int i;
    size_t pos = 0;
    for (i = 0; i < idx->count && pos + 16 <= cap; i++) {
        buf[pos++] = '0'; buf[pos++] = '0'; buf[pos++] = 'd'; buf[pos++] = 'c';
        buf[pos++] = (uint8_t)(idx->entries[i].flags);
        buf[pos++] = (uint8_t)(idx->entries[i].flags >> 8);
        buf[pos++] = (uint8_t)(idx->entries[i].flags >> 16);
        buf[pos++] = (uint8_t)(idx->entries[i].flags >> 24);
        buf[pos++] = (uint8_t)(idx->entries[i].offset);
        buf[pos++] = (uint8_t)(idx->entries[i].offset >> 8);
        buf[pos++] = (uint8_t)(idx->entries[i].offset >> 16);
        buf[pos++] = (uint8_t)(idx->entries[i].offset >> 24);
        buf[pos++] = (uint8_t)(idx->entries[i].size);
        buf[pos++] = (uint8_t)(idx->entries[i].size >> 8);
        buf[pos++] = (uint8_t)(idx->entries[i].size >> 16);
        buf[pos++] = (uint8_t)(idx->entries[i].size >> 24);
    }
    return (int)pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1373 failed: {:?}", result.err());
}

/// C1374: MP4 box parser -- reads ISO BMFF/MP4 box (atom) headers
#[test]
fn c1374_mp4_box_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

typedef struct { uint32_t size; char type_code[4]; int header_size; } codec_mp4_box_t;

static uint32_t codec_mp4_read32be(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
}

int codec_mp4_parse_box(const uint8_t *data, size_t len, codec_mp4_box_t *box) {
    if (len < 8) return -1;
    box->size = codec_mp4_read32be(data);
    box->type_code[0] = (char)data[4];
    box->type_code[1] = (char)data[5];
    box->type_code[2] = (char)data[6];
    box->type_code[3] = (char)data[7];
    box->header_size = 8;
    if (box->size == 0) box->size = (uint32_t)len;
    return 0;
}

int codec_mp4_is_container(const codec_mp4_box_t *box) {
    if (box->type_code[0] == 'm' && box->type_code[1] == 'o' &&
        box->type_code[2] == 'o' && box->type_code[3] == 'v') return 1;
    if (box->type_code[0] == 't' && box->type_code[1] == 'r' &&
        box->type_code[2] == 'a' && box->type_code[3] == 'k') return 1;
    if (box->type_code[0] == 'm' && box->type_code[1] == 'd' &&
        box->type_code[2] == 'i' && box->type_code[3] == 'a') return 1;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1374 failed: {:?}", result.err());
}

/// C1375: FLV tag writer -- writes FLV (Flash Video) tag headers
#[test]
fn c1375_flv_tag_writer() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

typedef struct { uint8_t *buf; size_t pos; size_t cap; } codec_flv_t;

void codec_flv_init(codec_flv_t *f, uint8_t *buf, size_t cap) {
    f->buf = buf; f->pos = 0; f->cap = cap;
}

int codec_flv_write_header(codec_flv_t *f, int has_audio, int has_video) {
    if (f->pos + 9 > f->cap) return -1;
    f->buf[f->pos++] = 'F';
    f->buf[f->pos++] = 'L';
    f->buf[f->pos++] = 'V';
    f->buf[f->pos++] = 1;
    f->buf[f->pos++] = (uint8_t)((has_audio ? 4 : 0) | (has_video ? 1 : 0));
    f->buf[f->pos++] = 0; f->buf[f->pos++] = 0;
    f->buf[f->pos++] = 0; f->buf[f->pos++] = 9;
    return 0;
}

int codec_flv_write_tag(codec_flv_t *f, uint8_t tag_type, uint32_t data_size, uint32_t timestamp) {
    if (f->pos + 11 > f->cap) return -1;
    f->buf[f->pos++] = tag_type;
    f->buf[f->pos++] = (uint8_t)((data_size >> 16) & 0xFF);
    f->buf[f->pos++] = (uint8_t)((data_size >> 8) & 0xFF);
    f->buf[f->pos++] = (uint8_t)(data_size & 0xFF);
    f->buf[f->pos++] = (uint8_t)((timestamp >> 16) & 0xFF);
    f->buf[f->pos++] = (uint8_t)((timestamp >> 8) & 0xFF);
    f->buf[f->pos++] = (uint8_t)(timestamp & 0xFF);
    f->buf[f->pos++] = (uint8_t)((timestamp >> 24) & 0xFF);
    f->buf[f->pos++] = 0; f->buf[f->pos++] = 0; f->buf[f->pos++] = 0;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1375 failed: {:?}", result.err());
}
