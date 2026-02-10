//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C576-C600: File Format Parsing and Generation -- the kind of C code found
//! in image loaders, audio codecs, archive tools, and document parsers.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world file format programming patterns commonly
//! found in stb_image, libpng, zlib, libxml2, and similar format libraries --
//! all expressed as valid C99.
//!
//! Organization:
//! - C576-C580: Image formats (BMP header, BMP pixels, WAV header, WAV PCM, CSV simple)
//! - C581-C585: Text formats (CSV quoted, INI parser, JSON tokenizer, JSON numbers, JSON strings)
//! - C586-C590: Binary image/audio (PGM reader, PPM writer, TGA reader, MIDI events, ELF header)
//! - C591-C595: Executable/archive (PE header, TAR header, ZIP header, GIF LZW, PNG chunks)
//! - C596-C600: Document/encoding (TIFF tags, PDF objects, XML tokens, S-expressions, protobuf varint)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C576-C580: Image and Audio Formats (BMP, WAV, CSV)
// ============================================================================

#[test]
fn c576_bmp_header_parsing() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t signature[2];
    uint32_t file_size;
    uint16_t reserved1;
    uint16_t reserved2;
    uint32_t data_offset;
} bmp_file_header_t;

typedef struct {
    uint32_t header_size;
    int width;
    int height;
    uint16_t planes;
    uint16_t bits_per_pixel;
    uint32_t compression;
    uint32_t image_size;
    int x_ppm;
    int y_ppm;
    uint32_t colors_used;
    uint32_t colors_important;
} bmp_info_header_t;

uint32_t bmp_read_le32(const uint8_t *data) {
    return (uint32_t)data[0]
         | ((uint32_t)data[1] << 8)
         | ((uint32_t)data[2] << 16)
         | ((uint32_t)data[3] << 24);
}

uint16_t bmp_read_le16(const uint8_t *data) {
    return (uint16_t)data[0] | ((uint16_t)data[1] << 8);
}

int bmp_parse_file_header(const uint8_t *data, int len, bmp_file_header_t *hdr) {
    if (len < 14) {
        return -1;
    }
    hdr->signature[0] = data[0];
    hdr->signature[1] = data[1];
    if (hdr->signature[0] != 0x42 || hdr->signature[1] != 0x4D) {
        return -2;
    }
    hdr->file_size = bmp_read_le32(data + 2);
    hdr->reserved1 = bmp_read_le16(data + 6);
    hdr->reserved2 = bmp_read_le16(data + 8);
    hdr->data_offset = bmp_read_le32(data + 10);
    return 0;
}

int bmp_parse_info_header(const uint8_t *data, int len, bmp_info_header_t *info) {
    if (len < 40) {
        return -1;
    }
    info->header_size = bmp_read_le32(data);
    info->width = (int)bmp_read_le32(data + 4);
    info->height = (int)bmp_read_le32(data + 8);
    info->planes = bmp_read_le16(data + 12);
    info->bits_per_pixel = bmp_read_le16(data + 14);
    info->compression = bmp_read_le32(data + 16);
    info->image_size = bmp_read_le32(data + 20);
    return 0;
}

int bmp_is_valid(const bmp_file_header_t *fh, const bmp_info_header_t *ih) {
    if (ih->width <= 0 || ih->height == 0) {
        return 0;
    }
    if (ih->bits_per_pixel != 1 && ih->bits_per_pixel != 4
        && ih->bits_per_pixel != 8 && ih->bits_per_pixel != 24
        && ih->bits_per_pixel != 32) {
        return 0;
    }
    if (fh->data_offset < 54) {
        return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C576: BMP header parsing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C576: Output should not be empty");
    assert!(
        code.contains("fn bmp_parse_file_header"),
        "C576: Should contain bmp_parse_file_header function"
    );
}

#[test]
fn c577_bmp_pixel_data_reading() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} pixel_t;

void bmp_read_24bit_row(const uint8_t *row_data, pixel_t *pixels, int width) {
    int x;
    for (x = 0; x < width; x++) {
        int offset = x * 3;
        pixels[x].b = row_data[offset];
        pixels[x].g = row_data[offset + 1];
        pixels[x].r = row_data[offset + 2];
        pixels[x].a = 255;
    }
}

void bmp_read_32bit_row(const uint8_t *row_data, pixel_t *pixels, int width) {
    int x;
    for (x = 0; x < width; x++) {
        int offset = x * 4;
        pixels[x].b = row_data[offset];
        pixels[x].g = row_data[offset + 1];
        pixels[x].r = row_data[offset + 2];
        pixels[x].a = row_data[offset + 3];
    }
}

int bmp_row_stride(int width, int bits_per_pixel) {
    int row_bytes = (width * bits_per_pixel + 7) / 8;
    return (row_bytes + 3) & ~3;
}

void bmp_flip_vertical(pixel_t *pixels, int width, int height) {
    int y;
    for (y = 0; y < height / 2; y++) {
        int top_row = y * width;
        int bot_row = (height - 1 - y) * width;
        int x;
        for (x = 0; x < width; x++) {
            pixel_t tmp = pixels[top_row + x];
            pixels[top_row + x] = pixels[bot_row + x];
            pixels[bot_row + x] = tmp;
        }
    }
}

uint32_t bmp_pixel_to_packed(const pixel_t *p) {
    return ((uint32_t)p->a << 24) | ((uint32_t)p->r << 16)
         | ((uint32_t)p->g << 8) | (uint32_t)p->b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C577: BMP pixel data reading should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C577: Output should not be empty");
    assert!(
        code.contains("fn bmp_read_24bit_row"),
        "C577: Should contain bmp_read_24bit_row function"
    );
}

#[test]
fn c578_wav_header_creation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t riff_id[4];
    uint32_t file_size;
    uint8_t wave_id[4];
    uint8_t fmt_id[4];
    uint32_t fmt_size;
    uint16_t audio_format;
    uint16_t num_channels;
    uint32_t sample_rate;
    uint32_t byte_rate;
    uint16_t block_align;
    uint16_t bits_per_sample;
    uint8_t data_id[4];
    uint32_t data_size;
} wav_header_t;

void wav_write_le32(uint8_t *buf, uint32_t val) {
    buf[0] = (uint8_t)(val & 0xFF);
    buf[1] = (uint8_t)((val >> 8) & 0xFF);
    buf[2] = (uint8_t)((val >> 16) & 0xFF);
    buf[3] = (uint8_t)((val >> 24) & 0xFF);
}

void wav_write_le16(uint8_t *buf, uint16_t val) {
    buf[0] = (uint8_t)(val & 0xFF);
    buf[1] = (uint8_t)((val >> 8) & 0xFF);
}

void wav_init_header(wav_header_t *h, uint16_t channels,
                     uint32_t sample_rate, uint16_t bits) {
    h->riff_id[0] = 'R';
    h->riff_id[1] = 'I';
    h->riff_id[2] = 'F';
    h->riff_id[3] = 'F';
    h->wave_id[0] = 'W';
    h->wave_id[1] = 'A';
    h->wave_id[2] = 'V';
    h->wave_id[3] = 'E';
    h->fmt_id[0] = 'f';
    h->fmt_id[1] = 'm';
    h->fmt_id[2] = 't';
    h->fmt_id[3] = ' ';
    h->fmt_size = 16;
    h->audio_format = 1;
    h->num_channels = channels;
    h->sample_rate = sample_rate;
    h->bits_per_sample = bits;
    h->block_align = channels * (bits / 8);
    h->byte_rate = sample_rate * (uint32_t)h->block_align;
    h->data_id[0] = 'd';
    h->data_id[1] = 'a';
    h->data_id[2] = 't';
    h->data_id[3] = 'a';
    h->data_size = 0;
}

void wav_set_data_size(wav_header_t *h, uint32_t num_samples) {
    h->data_size = num_samples * (uint32_t)h->block_align;
    h->file_size = 36 + h->data_size;
}

int wav_is_valid_header(const wav_header_t *h) {
    if (h->audio_format != 1) {
        return 0;
    }
    if (h->num_channels == 0 || h->num_channels > 8) {
        return 0;
    }
    if (h->sample_rate < 8000 || h->sample_rate > 192000) {
        return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C578: WAV header creation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C578: Output should not be empty");
    assert!(
        code.contains("fn wav_init_header"),
        "C578: Should contain wav_init_header function"
    );
}

#[test]
fn c579_wav_pcm_data_writing() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

void wav_write_pcm16_mono(uint8_t *buf, const float *samples, int count) {
    int i;
    for (i = 0; i < count; i++) {
        float s = samples[i];
        if (s > 1.0f) {
            s = 1.0f;
        }
        if (s < -1.0f) {
            s = -1.0f;
        }
        int val = (int)(s * 32767.0f);
        if (val > 32767) {
            val = 32767;
        }
        if (val < -32768) {
            val = -32768;
        }
        buf[i * 2] = (uint8_t)(val & 0xFF);
        buf[i * 2 + 1] = (uint8_t)((val >> 8) & 0xFF);
    }
}

void wav_write_pcm16_stereo(uint8_t *buf, const float *left,
                            const float *right, int count) {
    int i;
    for (i = 0; i < count; i++) {
        float sl = left[i];
        float sr = right[i];
        if (sl > 1.0f) { sl = 1.0f; }
        if (sl < -1.0f) { sl = -1.0f; }
        if (sr > 1.0f) { sr = 1.0f; }
        if (sr < -1.0f) { sr = -1.0f; }
        int vl = (int)(sl * 32767.0f);
        int vr = (int)(sr * 32767.0f);
        buf[i * 4] = (uint8_t)(vl & 0xFF);
        buf[i * 4 + 1] = (uint8_t)((vl >> 8) & 0xFF);
        buf[i * 4 + 2] = (uint8_t)(vr & 0xFF);
        buf[i * 4 + 3] = (uint8_t)((vr >> 8) & 0xFF);
    }
}

float wav_read_pcm16_sample(const uint8_t *buf) {
    int val = (int)buf[0] | ((int)(buf[1]) << 8);
    if (val >= 32768) {
        val = val - 65536;
    }
    return (float)val / 32768.0f;
}

float wav_compute_rms(const float *samples, int count) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < count; i++) {
        sum += samples[i] * samples[i];
    }
    if (count > 0) {
        return sum / (float)count;
    }
    return 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C579: WAV PCM data writing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C579: Output should not be empty");
    assert!(
        code.contains("fn wav_write_pcm16_mono"),
        "C579: Should contain wav_write_pcm16_mono function"
    );
}

#[test]
fn c580_csv_parser_simple() {
    let c_code = r#"
#define CSV_MAX_FIELDS 32
#define CSV_MAX_FIELD_LEN 256

typedef struct {
    char fields[CSV_MAX_FIELDS][CSV_MAX_FIELD_LEN];
    int num_fields;
} csv_row_t;

int csv_parse_row(const char *line, csv_row_t *row) {
    int field_idx = 0;
    int char_idx = 0;
    int i = 0;
    row->num_fields = 0;

    while (line[i] != '\0' && line[i] != '\n' && field_idx < CSV_MAX_FIELDS) {
        char_idx = 0;
        while (line[i] != '\0' && line[i] != ',' && line[i] != '\n'
               && char_idx < CSV_MAX_FIELD_LEN - 1) {
            row->fields[field_idx][char_idx] = line[i];
            char_idx++;
            i++;
        }
        row->fields[field_idx][char_idx] = '\0';
        field_idx++;
        if (line[i] == ',') {
            i++;
        }
    }
    row->num_fields = field_idx;
    return field_idx;
}

int csv_field_is_empty(const csv_row_t *row, int idx) {
    if (idx < 0 || idx >= row->num_fields) {
        return 1;
    }
    return row->fields[idx][0] == '\0';
}

int csv_count_rows(const char *data) {
    int count = 0;
    int i = 0;
    int in_line = 0;
    while (data[i] != '\0') {
        if (data[i] == '\n') {
            if (in_line) {
                count++;
            }
            in_line = 0;
        } else {
            in_line = 1;
        }
        i++;
    }
    if (in_line) {
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C580: CSV parser (simple) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C580: Output should not be empty");
    assert!(
        code.contains("fn csv_parse_row"),
        "C580: Should contain csv_parse_row function"
    );
}

// ============================================================================
// C581-C585: Text Formats (CSV Quoted, INI, JSON)
// ============================================================================

#[test]
fn c581_csv_parser_quoted_fields() {
    let c_code = r#"
#define CSV_MAX_LEN 256

int csv_parse_quoted_field(const char *line, int start, char *out, int max_len) {
    int i = start;
    int j = 0;
    if (line[i] != '"') {
        return -1;
    }
    i++;
    while (line[i] != '\0' && j < max_len - 1) {
        if (line[i] == '"') {
            if (line[i + 1] == '"') {
                out[j] = '"';
                j++;
                i += 2;
            } else {
                break;
            }
        } else {
            out[j] = line[i];
            j++;
            i++;
        }
    }
    out[j] = '\0';
    if (line[i] == '"') {
        i++;
    }
    return i;
}

int csv_needs_quoting(const char *field) {
    int i = 0;
    while (field[i] != '\0') {
        if (field[i] == ',' || field[i] == '"' || field[i] == '\n') {
            return 1;
        }
        i++;
    }
    return 0;
}

int csv_write_quoted_field(const char *field, char *out, int max_len) {
    int i = 0;
    int j = 0;
    if (j < max_len - 1) {
        out[j] = '"';
        j++;
    }
    while (field[i] != '\0' && j < max_len - 2) {
        if (field[i] == '"') {
            if (j < max_len - 3) {
                out[j] = '"';
                out[j + 1] = '"';
                j += 2;
            }
        } else {
            out[j] = field[i];
            j++;
        }
        i++;
    }
    if (j < max_len - 1) {
        out[j] = '"';
        j++;
    }
    out[j] = '\0';
    return j;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C581: CSV parser (quoted fields) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C581: Output should not be empty");
    assert!(
        code.contains("fn csv_parse_quoted_field"),
        "C581: Should contain csv_parse_quoted_field function"
    );
}

#[test]
fn c582_ini_file_parser() {
    let c_code = r#"
#define INI_MAX_LINE 256
#define INI_MAX_SECTION 64
#define INI_MAX_KEY 64
#define INI_MAX_VALUE 192

typedef struct {
    int type;
    char section[INI_MAX_SECTION];
    char key[INI_MAX_KEY];
    char value[INI_MAX_VALUE];
} ini_entry_t;

void ini_trim_whitespace(char *str) {
    int i = 0;
    int start = 0;
    int end;
    while (str[start] == ' ' || str[start] == '\t') {
        start++;
    }
    end = start;
    while (str[end] != '\0') {
        end++;
    }
    end--;
    while (end > start && (str[end] == ' ' || str[end] == '\t'
           || str[end] == '\n' || str[end] == '\r')) {
        end--;
    }
    i = 0;
    while (start <= end) {
        str[i] = str[start];
        i++;
        start++;
    }
    str[i] = '\0';
}

int ini_parse_line(const char *line, ini_entry_t *entry) {
    int i = 0;
    entry->type = 0;
    entry->section[0] = '\0';
    entry->key[0] = '\0';
    entry->value[0] = '\0';

    while (line[i] == ' ' || line[i] == '\t') {
        i++;
    }
    if (line[i] == '\0' || line[i] == '\n' || line[i] == ';' || line[i] == '#') {
        entry->type = 0;
        return 0;
    }
    if (line[i] == '[') {
        i++;
        int j = 0;
        while (line[i] != ']' && line[i] != '\0' && j < INI_MAX_SECTION - 1) {
            entry->section[j] = line[i];
            j++;
            i++;
        }
        entry->section[j] = '\0';
        entry->type = 1;
        return 1;
    }
    {
        int j = 0;
        while (line[i] != '=' && line[i] != '\0' && j < INI_MAX_KEY - 1) {
            entry->key[j] = line[i];
            j++;
            i++;
        }
        entry->key[j] = '\0';
        ini_trim_whitespace(entry->key);
    }
    if (line[i] == '=') {
        i++;
        int j = 0;
        while (line[i] != '\0' && line[i] != '\n' && j < INI_MAX_VALUE - 1) {
            entry->value[j] = line[i];
            j++;
            i++;
        }
        entry->value[j] = '\0';
        ini_trim_whitespace(entry->value);
    }
    entry->type = 2;
    return 2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C582: INI file parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C582: Output should not be empty");
    assert!(
        code.contains("fn ini_parse_line"),
        "C582: Should contain ini_parse_line function"
    );
}

#[test]
fn c583_json_tokenizer() {
    let c_code = r#"
#define JSON_TOK_NONE    0
#define JSON_TOK_LBRACE  1
#define JSON_TOK_RBRACE  2
#define JSON_TOK_LBRACK  3
#define JSON_TOK_RBRACK  4
#define JSON_TOK_COLON   5
#define JSON_TOK_COMMA   6
#define JSON_TOK_STRING  7
#define JSON_TOK_NUMBER  8
#define JSON_TOK_TRUE    9
#define JSON_TOK_FALSE   10
#define JSON_TOK_NULL    11
#define JSON_TOK_EOF     12
#define JSON_TOK_ERROR   13

typedef struct {
    const char *input;
    int pos;
    int len;
    int token_type;
    int token_start;
    int token_len;
} json_lexer_t;

void json_lexer_init(json_lexer_t *lex, const char *input, int len) {
    lex->input = input;
    lex->pos = 0;
    lex->len = len;
    lex->token_type = JSON_TOK_NONE;
    lex->token_start = 0;
    lex->token_len = 0;
}

void json_skip_whitespace(json_lexer_t *lex) {
    while (lex->pos < lex->len) {
        char c = lex->input[lex->pos];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            lex->pos++;
        } else {
            break;
        }
    }
}

int json_next_token(json_lexer_t *lex) {
    json_skip_whitespace(lex);
    if (lex->pos >= lex->len) {
        lex->token_type = JSON_TOK_EOF;
        return JSON_TOK_EOF;
    }
    char c = lex->input[lex->pos];
    lex->token_start = lex->pos;
    if (c == '{') { lex->pos++; lex->token_type = JSON_TOK_LBRACE; lex->token_len = 1; return lex->token_type; }
    if (c == '}') { lex->pos++; lex->token_type = JSON_TOK_RBRACE; lex->token_len = 1; return lex->token_type; }
    if (c == '[') { lex->pos++; lex->token_type = JSON_TOK_LBRACK; lex->token_len = 1; return lex->token_type; }
    if (c == ']') { lex->pos++; lex->token_type = JSON_TOK_RBRACK; lex->token_len = 1; return lex->token_type; }
    if (c == ':') { lex->pos++; lex->token_type = JSON_TOK_COLON; lex->token_len = 1; return lex->token_type; }
    if (c == ',') { lex->pos++; lex->token_type = JSON_TOK_COMMA; lex->token_len = 1; return lex->token_type; }
    lex->token_type = JSON_TOK_ERROR;
    lex->token_len = 1;
    lex->pos++;
    return JSON_TOK_ERROR;
}

int json_is_structural(int tok) {
    return tok == JSON_TOK_LBRACE || tok == JSON_TOK_RBRACE
        || tok == JSON_TOK_LBRACK || tok == JSON_TOK_RBRACK
        || tok == JSON_TOK_COLON || tok == JSON_TOK_COMMA;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C583: JSON tokenizer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C583: Output should not be empty");
    assert!(
        code.contains("fn json_next_token"),
        "C583: Should contain json_next_token function"
    );
}

#[test]
fn c584_json_number_parsing() {
    let c_code = r#"
typedef struct {
    int is_float;
    int int_value;
    float float_value;
    int valid;
} json_number_t;

int json_is_digit(char c) {
    return c >= '0' && c <= '9';
}

json_number_t json_parse_number(const char *s, int len) {
    json_number_t result;
    result.is_float = 0;
    result.int_value = 0;
    result.float_value = 0.0f;
    result.valid = 0;

    int i = 0;
    int sign = 1;
    if (i < len && s[i] == '-') {
        sign = -1;
        i++;
    }
    if (i >= len || !json_is_digit(s[i])) {
        return result;
    }
    int int_part = 0;
    while (i < len && json_is_digit(s[i])) {
        int_part = int_part * 10 + (s[i] - '0');
        i++;
    }
    if (i < len && s[i] == '.') {
        result.is_float = 1;
        i++;
        float frac = 0.0f;
        float place = 0.1f;
        while (i < len && json_is_digit(s[i])) {
            frac += (float)(s[i] - '0') * place;
            place *= 0.1f;
            i++;
        }
        result.float_value = (float)sign * ((float)int_part + frac);
    }
    if (i < len && (s[i] == 'e' || s[i] == 'E')) {
        result.is_float = 1;
        i++;
        int exp_sign = 1;
        if (i < len && s[i] == '-') {
            exp_sign = -1;
            i++;
        } else if (i < len && s[i] == '+') {
            i++;
        }
        int exp_val = 0;
        while (i < len && json_is_digit(s[i])) {
            exp_val = exp_val * 10 + (s[i] - '0');
            i++;
        }
        float base = result.float_value;
        if (base == 0.0f) {
            base = (float)(sign * int_part);
        }
        float mul = 1.0f;
        int e;
        for (e = 0; e < exp_val; e++) {
            mul *= 10.0f;
        }
        if (exp_sign < 0) {
            result.float_value = base / mul;
        } else {
            result.float_value = base * mul;
        }
    }
    if (!result.is_float) {
        result.int_value = sign * int_part;
    }
    result.valid = 1;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C584: JSON number parsing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C584: Output should not be empty");
    assert!(
        code.contains("fn json_parse_number"),
        "C584: Should contain json_parse_number function"
    );
}

#[test]
fn c585_json_string_escape_parsing() {
    let c_code = r#"
#define JSON_STR_MAX 512

int json_parse_string(const char *input, int start, char *out, int max_len) {
    int i = start;
    int j = 0;
    if (input[i] != '"') {
        return -1;
    }
    i++;
    while (input[i] != '\0' && input[i] != '"' && j < max_len - 1) {
        if (input[i] == '\\') {
            i++;
            if (input[i] == '"') { out[j] = '"'; }
            else if (input[i] == '\\') { out[j] = '\\'; }
            else if (input[i] == '/') { out[j] = '/'; }
            else if (input[i] == 'b') { out[j] = '\b'; }
            else if (input[i] == 'f') { out[j] = '\f'; }
            else if (input[i] == 'n') { out[j] = '\n'; }
            else if (input[i] == 'r') { out[j] = '\r'; }
            else if (input[i] == 't') { out[j] = '\t'; }
            else { out[j] = input[i]; }
            j++;
            i++;
        } else {
            out[j] = input[i];
            j++;
            i++;
        }
    }
    out[j] = '\0';
    if (input[i] == '"') {
        i++;
    }
    return i;
}

int json_string_length(const char *input, int start) {
    int i = start;
    if (input[i] != '"') {
        return -1;
    }
    i++;
    int len = 0;
    while (input[i] != '\0' && input[i] != '"') {
        if (input[i] == '\\') {
            i++;
        }
        len++;
        i++;
    }
    return len;
}

int json_string_needs_escape(const char *str) {
    int i = 0;
    while (str[i] != '\0') {
        if (str[i] == '"' || str[i] == '\\' || str[i] == '\n'
            || str[i] == '\r' || str[i] == '\t') {
            return 1;
        }
        i++;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C585: JSON string escape parsing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C585: Output should not be empty");
    assert!(
        code.contains("fn json_parse_string"),
        "C585: Should contain json_parse_string function"
    );
}

// ============================================================================
// C586-C590: Binary Image/Audio Formats (PGM, PPM, TGA, MIDI, ELF)
// ============================================================================

#[test]
fn c586_pgm_image_reader() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int width;
    int height;
    int max_val;
    int data_offset;
} pgm_header_t;

int pgm_skip_whitespace_and_comments(const char *data, int pos, int len) {
    while (pos < len) {
        if (data[pos] == '#') {
            while (pos < len && data[pos] != '\n') {
                pos++;
            }
        } else if (data[pos] == ' ' || data[pos] == '\t'
                   || data[pos] == '\n' || data[pos] == '\r') {
            pos++;
        } else {
            break;
        }
    }
    return pos;
}

int pgm_read_int(const char *data, int pos, int len, int *value) {
    *value = 0;
    int started = 0;
    while (pos < len && data[pos] >= '0' && data[pos] <= '9') {
        *value = (*value) * 10 + (data[pos] - '0');
        pos++;
        started = 1;
    }
    if (!started) {
        return -1;
    }
    return pos;
}

int pgm_parse_header(const char *data, int len, pgm_header_t *hdr) {
    if (len < 3) {
        return -1;
    }
    if (data[0] != 'P' || data[1] != '5') {
        return -2;
    }
    int pos = 2;
    pos = pgm_skip_whitespace_and_comments(data, pos, len);
    pos = pgm_read_int(data, pos, len, &hdr->width);
    if (pos < 0) { return -3; }
    pos = pgm_skip_whitespace_and_comments(data, pos, len);
    pos = pgm_read_int(data, pos, len, &hdr->height);
    if (pos < 0) { return -4; }
    pos = pgm_skip_whitespace_and_comments(data, pos, len);
    pos = pgm_read_int(data, pos, len, &hdr->max_val);
    if (pos < 0) { return -5; }
    hdr->data_offset = pos + 1;
    return 0;
}

uint8_t pgm_get_pixel(const uint8_t *data, int data_offset, int width, int x, int y) {
    return data[data_offset + y * width + x];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C586: PGM image reader should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C586: Output should not be empty");
    assert!(
        code.contains("fn pgm_parse_header"),
        "C586: Should contain pgm_parse_header function"
    );
}

#[test]
fn c587_ppm_image_writer() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} rgb_pixel_t;

int ppm_write_header(char *buf, int buf_size, int width, int height) {
    int i = 0;
    char tmp[32];
    int j;

    buf[i] = 'P'; i++;
    buf[i] = '6'; i++;
    buf[i] = '\n'; i++;

    int w = width;
    j = 0;
    while (w > 0 && j < 31) {
        tmp[j] = '0' + (w % 10);
        w = w / 10;
        j++;
    }
    j--;
    while (j >= 0 && i < buf_size) {
        buf[i] = tmp[j];
        i++;
        j--;
    }
    buf[i] = ' '; i++;

    int h = height;
    j = 0;
    while (h > 0 && j < 31) {
        tmp[j] = '0' + (h % 10);
        h = h / 10;
        j++;
    }
    j--;
    while (j >= 0 && i < buf_size) {
        buf[i] = tmp[j];
        i++;
        j--;
    }
    buf[i] = '\n'; i++;
    buf[i] = '2'; i++;
    buf[i] = '5'; i++;
    buf[i] = '5'; i++;
    buf[i] = '\n'; i++;
    return i;
}

void ppm_write_pixel_data(uint8_t *buf, const rgb_pixel_t *pixels,
                          int width, int height) {
    int i;
    int total = width * height;
    for (i = 0; i < total; i++) {
        buf[i * 3] = pixels[i].r;
        buf[i * 3 + 1] = pixels[i].g;
        buf[i * 3 + 2] = pixels[i].b;
    }
}

rgb_pixel_t ppm_blend_pixels(rgb_pixel_t a, rgb_pixel_t b, int alpha) {
    rgb_pixel_t result;
    result.r = (uint8_t)((a.r * (255 - alpha) + b.r * alpha) / 255);
    result.g = (uint8_t)((a.g * (255 - alpha) + b.g * alpha) / 255);
    result.b = (uint8_t)((a.b * (255 - alpha) + b.b * alpha) / 255);
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C587: PPM image writer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C587: Output should not be empty");
    assert!(
        code.contains("fn ppm_write_header"),
        "C587: Should contain ppm_write_header function"
    );
}

#[test]
fn c588_tga_image_reader() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t id_length;
    uint8_t color_map_type;
    uint8_t image_type;
    uint16_t cm_first_entry;
    uint16_t cm_length;
    uint8_t cm_entry_size;
    uint16_t x_origin;
    uint16_t y_origin;
    uint16_t width;
    uint16_t height;
    uint8_t pixel_depth;
    uint8_t image_descriptor;
} tga_header_t;

int tga_parse_header(const uint8_t *data, int len, tga_header_t *hdr) {
    if (len < 18) {
        return -1;
    }
    hdr->id_length = data[0];
    hdr->color_map_type = data[1];
    hdr->image_type = data[2];
    hdr->cm_first_entry = (uint16_t)data[3] | ((uint16_t)data[4] << 8);
    hdr->cm_length = (uint16_t)data[5] | ((uint16_t)data[6] << 8);
    hdr->cm_entry_size = data[7];
    hdr->x_origin = (uint16_t)data[8] | ((uint16_t)data[9] << 8);
    hdr->y_origin = (uint16_t)data[10] | ((uint16_t)data[11] << 8);
    hdr->width = (uint16_t)data[12] | ((uint16_t)data[13] << 8);
    hdr->height = (uint16_t)data[14] | ((uint16_t)data[15] << 8);
    hdr->pixel_depth = data[16];
    hdr->image_descriptor = data[17];
    return 0;
}

int tga_is_uncompressed_rgb(const tga_header_t *hdr) {
    return hdr->image_type == 2;
}

int tga_is_rle_rgb(const tga_header_t *hdr) {
    return hdr->image_type == 10;
}

int tga_pixel_data_offset(const tga_header_t *hdr) {
    int cm_size = 0;
    if (hdr->color_map_type == 1) {
        cm_size = hdr->cm_length * ((hdr->cm_entry_size + 7) / 8);
    }
    return 18 + hdr->id_length + cm_size;
}

int tga_is_top_to_bottom(const tga_header_t *hdr) {
    return (hdr->image_descriptor & 0x20) != 0;
}

int tga_bytes_per_pixel(const tga_header_t *hdr) {
    return (hdr->pixel_depth + 7) / 8;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C588: TGA image reader should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C588: Output should not be empty");
    assert!(
        code.contains("fn tga_parse_header"),
        "C588: Should contain tga_parse_header function"
    );
}

#[test]
fn c589_midi_event_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define MIDI_NOTE_OFF    0x80
#define MIDI_NOTE_ON     0x90
#define MIDI_AFTERTOUCH  0xA0
#define MIDI_CC          0xB0
#define MIDI_PROGRAM     0xC0
#define MIDI_CHAN_PRESS   0xD0
#define MIDI_PITCH_BEND  0xE0
#define MIDI_SYSEX       0xF0

typedef struct {
    uint8_t status;
    uint8_t channel;
    uint8_t data1;
    uint8_t data2;
    uint32_t delta_time;
} midi_event_t;

int midi_read_variable_length(const uint8_t *data, int pos, int len, uint32_t *value) {
    *value = 0;
    int i = pos;
    while (i < len) {
        uint8_t byte = data[i];
        *value = (*value << 7) | (byte & 0x7F);
        i++;
        if ((byte & 0x80) == 0) {
            break;
        }
        if (i - pos >= 4) {
            break;
        }
    }
    return i;
}

int midi_parse_event(const uint8_t *data, int pos, int len, midi_event_t *evt) {
    if (pos >= len) {
        return -1;
    }
    pos = midi_read_variable_length(data, pos, len, &evt->delta_time);
    if (pos >= len) {
        return -1;
    }
    uint8_t status = data[pos];
    if (status < 0x80) {
        return -1;
    }
    evt->status = status & 0xF0;
    evt->channel = status & 0x0F;
    pos++;
    int msg_type = evt->status;
    if (msg_type == MIDI_NOTE_ON || msg_type == MIDI_NOTE_OFF
        || msg_type == MIDI_AFTERTOUCH || msg_type == MIDI_CC
        || msg_type == MIDI_PITCH_BEND) {
        if (pos + 1 >= len) { return -1; }
        evt->data1 = data[pos];
        evt->data2 = data[pos + 1];
        pos += 2;
    } else if (msg_type == MIDI_PROGRAM || msg_type == MIDI_CHAN_PRESS) {
        if (pos >= len) { return -1; }
        evt->data1 = data[pos];
        evt->data2 = 0;
        pos++;
    }
    return pos;
}

int midi_event_size(uint8_t status) {
    int type = status & 0xF0;
    if (type == MIDI_PROGRAM || type == MIDI_CHAN_PRESS) {
        return 2;
    }
    if (type >= 0x80 && type <= 0xE0) {
        return 3;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C589: MIDI event parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C589: Output should not be empty");
    assert!(
        code.contains("fn midi_parse_event"),
        "C589: Should contain midi_parse_event function"
    );
}

#[test]
fn c590_elf_header_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define ELF_MAGIC_0  0x7F
#define ELF_MAGIC_1  0x45
#define ELF_MAGIC_2  0x4C
#define ELF_MAGIC_3  0x46
#define ELF_CLASS_32  1
#define ELF_CLASS_64  2
#define ELF_DATA_LSB  1
#define ELF_DATA_MSB  2

typedef struct {
    uint8_t magic[4];
    uint8_t elf_class;
    uint8_t data_encoding;
    uint8_t version;
    uint8_t os_abi;
    uint16_t type;
    uint16_t machine;
    uint32_t entry_point;
    uint32_t ph_offset;
    uint32_t sh_offset;
    uint16_t ph_entry_size;
    uint16_t ph_num;
    uint16_t sh_entry_size;
    uint16_t sh_num;
} elf_header_t;

uint16_t elf_read16(const uint8_t *data, int is_le) {
    if (is_le) {
        return (uint16_t)data[0] | ((uint16_t)data[1] << 8);
    }
    return ((uint16_t)data[0] << 8) | (uint16_t)data[1];
}

uint32_t elf_read32(const uint8_t *data, int is_le) {
    if (is_le) {
        return (uint32_t)data[0] | ((uint32_t)data[1] << 8)
             | ((uint32_t)data[2] << 16) | ((uint32_t)data[3] << 24);
    }
    return ((uint32_t)data[0] << 24) | ((uint32_t)data[1] << 16)
         | ((uint32_t)data[2] << 8) | (uint32_t)data[3];
}

int elf_parse_header(const uint8_t *data, int len, elf_header_t *hdr) {
    if (len < 52) {
        return -1;
    }
    hdr->magic[0] = data[0];
    hdr->magic[1] = data[1];
    hdr->magic[2] = data[2];
    hdr->magic[3] = data[3];
    if (hdr->magic[0] != ELF_MAGIC_0 || hdr->magic[1] != ELF_MAGIC_1
        || hdr->magic[2] != ELF_MAGIC_2 || hdr->magic[3] != ELF_MAGIC_3) {
        return -2;
    }
    hdr->elf_class = data[4];
    hdr->data_encoding = data[5];
    hdr->version = data[6];
    hdr->os_abi = data[7];
    int is_le = (hdr->data_encoding == ELF_DATA_LSB);
    hdr->type = elf_read16(data + 16, is_le);
    hdr->machine = elf_read16(data + 18, is_le);
    hdr->entry_point = elf_read32(data + 24, is_le);
    hdr->ph_offset = elf_read32(data + 28, is_le);
    hdr->sh_offset = elf_read32(data + 32, is_le);
    hdr->ph_entry_size = elf_read16(data + 42, is_le);
    hdr->ph_num = elf_read16(data + 44, is_le);
    hdr->sh_entry_size = elf_read16(data + 46, is_le);
    hdr->sh_num = elf_read16(data + 48, is_le);
    return 0;
}

int elf_is_executable(const elf_header_t *hdr) {
    return hdr->type == 2;
}

int elf_is_shared_object(const elf_header_t *hdr) {
    return hdr->type == 3;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C590: ELF header parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C590: Output should not be empty");
    assert!(
        code.contains("fn elf_parse_header"),
        "C590: Should contain elf_parse_header function"
    );
}

// ============================================================================
// C591-C595: Executable and Archive Formats (PE, TAR, ZIP, GIF, PNG)
// ============================================================================

#[test]
fn c591_pe_header_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define PE_DOS_MAGIC  0x5A4D
#define PE_SIGNATURE  0x00004550

typedef struct {
    uint16_t dos_magic;
    uint32_t pe_offset;
    uint16_t machine;
    uint16_t num_sections;
    uint32_t timestamp;
    uint16_t optional_hdr_size;
    uint16_t characteristics;
} pe_header_t;

uint16_t pe_read16_le(const uint8_t *d) {
    return (uint16_t)d[0] | ((uint16_t)d[1] << 8);
}

uint32_t pe_read32_le(const uint8_t *d) {
    return (uint32_t)d[0] | ((uint32_t)d[1] << 8)
         | ((uint32_t)d[2] << 16) | ((uint32_t)d[3] << 24);
}

int pe_parse_header(const uint8_t *data, int len, pe_header_t *hdr) {
    if (len < 64) {
        return -1;
    }
    hdr->dos_magic = pe_read16_le(data);
    if (hdr->dos_magic != PE_DOS_MAGIC) {
        return -2;
    }
    hdr->pe_offset = pe_read32_le(data + 60);
    if ((int)hdr->pe_offset + 24 > len) {
        return -3;
    }
    uint32_t sig = pe_read32_le(data + hdr->pe_offset);
    if (sig != PE_SIGNATURE) {
        return -4;
    }
    int coff = (int)hdr->pe_offset + 4;
    hdr->machine = pe_read16_le(data + coff);
    hdr->num_sections = pe_read16_le(data + coff + 2);
    hdr->timestamp = pe_read32_le(data + coff + 4);
    hdr->optional_hdr_size = pe_read16_le(data + coff + 16);
    hdr->characteristics = pe_read16_le(data + coff + 18);
    return 0;
}

int pe_is_dll(const pe_header_t *hdr) {
    return (hdr->characteristics & 0x2000) != 0;
}

int pe_is_x86(const pe_header_t *hdr) {
    return hdr->machine == 0x014C;
}

int pe_is_x64(const pe_header_t *hdr) {
    return hdr->machine == 0x8664;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C591: PE header parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C591: Output should not be empty");
    assert!(
        code.contains("fn pe_parse_header"),
        "C591: Should contain pe_parse_header function"
    );
}

#[test]
fn c592_tar_header_creation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    char name[100];
    char mode[8];
    char uid[8];
    char gid[8];
    char size[12];
    char mtime[12];
    char checksum[8];
    char typeflag;
    char linkname[100];
    char magic[6];
    char version[2];
    char uname[32];
    char gname[32];
} tar_header_t;

void tar_write_octal(char *buf, int buf_size, uint32_t value) {
    int i = buf_size - 2;
    buf[buf_size - 1] = '\0';
    while (i >= 0) {
        buf[i] = '0' + (char)(value & 7);
        value = value >> 3;
        i--;
    }
}

void tar_copy_string(char *dst, const char *src, int max_len) {
    int i = 0;
    while (i < max_len - 1 && src[i] != '\0') {
        dst[i] = src[i];
        i++;
    }
    while (i < max_len) {
        dst[i] = '\0';
        i++;
    }
}

void tar_init_header(tar_header_t *hdr, const char *filename,
                     uint32_t file_size, uint32_t mode) {
    int i;
    char *p = (char *)hdr;
    for (i = 0; i < 500; i++) {
        p[i] = '\0';
    }
    tar_copy_string(hdr->name, filename, 100);
    tar_write_octal(hdr->mode, 8, mode);
    tar_write_octal(hdr->uid, 8, 0);
    tar_write_octal(hdr->gid, 8, 0);
    tar_write_octal(hdr->size, 12, file_size);
    tar_write_octal(hdr->mtime, 12, 0);
    hdr->typeflag = '0';
    hdr->magic[0] = 'u';
    hdr->magic[1] = 's';
    hdr->magic[2] = 't';
    hdr->magic[3] = 'a';
    hdr->magic[4] = 'r';
    hdr->magic[5] = '\0';
    hdr->version[0] = '0';
    hdr->version[1] = '0';
}

uint32_t tar_compute_checksum(const uint8_t *header) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i < 512; i++) {
        if (i >= 148 && i < 156) {
            sum += 32;
        } else {
            sum += header[i];
        }
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C592: TAR header creation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C592: Output should not be empty");
    assert!(
        code.contains("fn tar_init_header"),
        "C592: Should contain tar_init_header function"
    );
}

#[test]
fn c593_zip_local_file_header() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define ZIP_LOCAL_MAGIC 0x04034B50

typedef struct {
    uint32_t signature;
    uint16_t version_needed;
    uint16_t flags;
    uint16_t compression;
    uint16_t mod_time;
    uint16_t mod_date;
    uint32_t crc32;
    uint32_t compressed_size;
    uint32_t uncompressed_size;
    uint16_t filename_len;
    uint16_t extra_len;
} zip_local_header_t;

uint16_t zip_le16(const uint8_t *d) {
    return (uint16_t)d[0] | ((uint16_t)d[1] << 8);
}

uint32_t zip_le32(const uint8_t *d) {
    return (uint32_t)d[0] | ((uint32_t)d[1] << 8)
         | ((uint32_t)d[2] << 16) | ((uint32_t)d[3] << 24);
}

int zip_parse_local_header(const uint8_t *data, int len, zip_local_header_t *hdr) {
    if (len < 30) {
        return -1;
    }
    hdr->signature = zip_le32(data);
    if (hdr->signature != ZIP_LOCAL_MAGIC) {
        return -2;
    }
    hdr->version_needed = zip_le16(data + 4);
    hdr->flags = zip_le16(data + 6);
    hdr->compression = zip_le16(data + 8);
    hdr->mod_time = zip_le16(data + 10);
    hdr->mod_date = zip_le16(data + 12);
    hdr->crc32 = zip_le32(data + 14);
    hdr->compressed_size = zip_le32(data + 18);
    hdr->uncompressed_size = zip_le32(data + 22);
    hdr->filename_len = zip_le16(data + 26);
    hdr->extra_len = zip_le16(data + 28);
    return 0;
}

int zip_data_offset(const zip_local_header_t *hdr) {
    return 30 + hdr->filename_len + hdr->extra_len;
}

int zip_is_stored(const zip_local_header_t *hdr) {
    return hdr->compression == 0;
}

int zip_is_deflated(const zip_local_header_t *hdr) {
    return hdr->compression == 8;
}

int zip_has_data_descriptor(const zip_local_header_t *hdr) {
    return (hdr->flags & 0x0008) != 0;
}

uint32_t zip_crc32_update(uint32_t crc, uint8_t byte) {
    uint32_t c = crc ^ 0xFFFFFFFF;
    int k;
    c = c ^ (uint32_t)byte;
    for (k = 0; k < 8; k++) {
        if (c & 1) {
            c = (c >> 1) ^ 0xEDB88320;
        } else {
            c = c >> 1;
        }
    }
    return c ^ 0xFFFFFFFF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C593: ZIP local file header should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C593: Output should not be empty");
    assert!(
        code.contains("fn zip_parse_local_header"),
        "C593: Should contain zip_parse_local_header function"
    );
}

#[test]
fn c594_gif_lzw_decompression() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define LZW_MAX_CODES 4096
#define LZW_MAX_STACK 4096

typedef struct {
    uint16_t prefix[LZW_MAX_CODES];
    uint8_t suffix[LZW_MAX_CODES];
    uint8_t stack[LZW_MAX_STACK];
    int stack_top;
    int next_code;
    int code_size;
    int clear_code;
    int end_code;
    int min_code_size;
} lzw_state_t;

void lzw_init(lzw_state_t *st, int min_code_size) {
    int i;
    st->min_code_size = min_code_size;
    st->clear_code = 1 << min_code_size;
    st->end_code = st->clear_code + 1;
    st->next_code = st->end_code + 1;
    st->code_size = min_code_size + 1;
    st->stack_top = 0;
    for (i = 0; i < st->clear_code; i++) {
        st->prefix[i] = 0xFFFF;
        st->suffix[i] = (uint8_t)i;
    }
}

void lzw_reset(lzw_state_t *st) {
    st->next_code = st->end_code + 1;
    st->code_size = st->min_code_size + 1;
}

void lzw_decode_string(lzw_state_t *st, int code) {
    st->stack_top = 0;
    while (code >= st->clear_code && st->stack_top < LZW_MAX_STACK) {
        if (code >= LZW_MAX_CODES) {
            break;
        }
        st->stack[st->stack_top] = st->suffix[code];
        st->stack_top++;
        code = st->prefix[code];
    }
    if (st->stack_top < LZW_MAX_STACK && code < st->clear_code) {
        st->stack[st->stack_top] = (uint8_t)code;
        st->stack_top++;
    }
}

void lzw_add_code(lzw_state_t *st, int prefix_code, uint8_t suffix_byte) {
    if (st->next_code < LZW_MAX_CODES) {
        st->prefix[st->next_code] = (uint16_t)prefix_code;
        st->suffix[st->next_code] = suffix_byte;
        st->next_code++;
        if (st->next_code > (1 << st->code_size) && st->code_size < 12) {
            st->code_size++;
        }
    }
}

int lzw_get_stack_value(const lzw_state_t *st, int idx) {
    if (idx >= 0 && idx < st->stack_top) {
        return st->stack[st->stack_top - 1 - idx];
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C594: GIF LZW decompression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C594: Output should not be empty");
    assert!(
        code.contains("fn lzw_init"),
        "C594: Should contain lzw_init function"
    );
}

#[test]
fn c595_png_chunk_reader() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define PNG_SIG_LEN 8

typedef struct {
    uint32_t length;
    char type[4];
    uint32_t crc;
    int data_offset;
} png_chunk_t;

uint32_t png_read_be32(const uint8_t *d) {
    return ((uint32_t)d[0] << 24) | ((uint32_t)d[1] << 16)
         | ((uint32_t)d[2] << 8) | (uint32_t)d[3];
}

int png_check_signature(const uint8_t *data, int len) {
    uint8_t sig[8];
    sig[0] = 137; sig[1] = 80; sig[2] = 78; sig[3] = 71;
    sig[4] = 13; sig[5] = 10; sig[6] = 26; sig[7] = 10;
    if (len < 8) {
        return 0;
    }
    int i;
    for (i = 0; i < 8; i++) {
        if (data[i] != sig[i]) {
            return 0;
        }
    }
    return 1;
}

int png_read_chunk(const uint8_t *data, int offset, int len, png_chunk_t *chunk) {
    if (offset + 12 > len) {
        return -1;
    }
    chunk->length = png_read_be32(data + offset);
    chunk->type[0] = (char)data[offset + 4];
    chunk->type[1] = (char)data[offset + 5];
    chunk->type[2] = (char)data[offset + 6];
    chunk->type[3] = (char)data[offset + 7];
    chunk->data_offset = offset + 8;
    if ((int)(offset + 12 + chunk->length) > len) {
        return -2;
    }
    chunk->crc = png_read_be32(data + offset + 8 + chunk->length);
    return (int)(offset + 12 + chunk->length);
}

int png_chunk_is_critical(const png_chunk_t *chunk) {
    return (chunk->type[0] & 0x20) == 0;
}

int png_chunk_is_type(const png_chunk_t *chunk, const char *type_name) {
    return chunk->type[0] == type_name[0]
        && chunk->type[1] == type_name[1]
        && chunk->type[2] == type_name[2]
        && chunk->type[3] == type_name[3];
}

uint32_t png_crc32_update(uint32_t crc, const uint8_t *data, int len) {
    uint32_t c = crc ^ 0xFFFFFFFF;
    int i;
    for (i = 0; i < len; i++) {
        c = c ^ (uint32_t)data[i];
        int k;
        for (k = 0; k < 8; k++) {
            if (c & 1) {
                c = (c >> 1) ^ 0xEDB88320;
            } else {
                c = c >> 1;
            }
        }
    }
    return c ^ 0xFFFFFFFF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C595: PNG chunk reader should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C595: Output should not be empty");
    assert!(
        code.contains("fn png_read_chunk"),
        "C595: Should contain png_read_chunk function"
    );
}

// ============================================================================
// C596-C600: Document and Encoding Formats (TIFF, PDF, XML, S-exp, Protobuf)
// ============================================================================

#[test]
fn c596_tiff_tag_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define TIFF_LE 0x4949
#define TIFF_BE 0x4D4D
#define TIFF_MAGIC 42

typedef struct {
    uint16_t tag;
    uint16_t type;
    uint32_t count;
    uint32_t value_offset;
} tiff_ifd_entry_t;

typedef struct {
    int is_le;
    uint16_t num_entries;
    int ifd_offset;
} tiff_ifd_t;

uint16_t tiff_read16(const uint8_t *d, int is_le) {
    if (is_le) {
        return (uint16_t)d[0] | ((uint16_t)d[1] << 8);
    }
    return ((uint16_t)d[0] << 8) | (uint16_t)d[1];
}

uint32_t tiff_read32(const uint8_t *d, int is_le) {
    if (is_le) {
        return (uint32_t)d[0] | ((uint32_t)d[1] << 8)
             | ((uint32_t)d[2] << 16) | ((uint32_t)d[3] << 24);
    }
    return ((uint32_t)d[0] << 24) | ((uint32_t)d[1] << 16)
         | ((uint32_t)d[2] << 8) | (uint32_t)d[3];
}

int tiff_parse_header(const uint8_t *data, int len, tiff_ifd_t *ifd) {
    if (len < 8) {
        return -1;
    }
    uint16_t byte_order = (uint16_t)data[0] | ((uint16_t)data[1] << 8);
    if (byte_order == TIFF_LE) {
        ifd->is_le = 1;
    } else if (byte_order == TIFF_BE) {
        ifd->is_le = 0;
    } else {
        return -2;
    }
    uint16_t magic = tiff_read16(data + 2, ifd->is_le);
    if (magic != TIFF_MAGIC) {
        return -3;
    }
    ifd->ifd_offset = (int)tiff_read32(data + 4, ifd->is_le);
    return 0;
}

int tiff_parse_ifd_entry(const uint8_t *data, int offset, int is_le,
                         tiff_ifd_entry_t *entry) {
    entry->tag = tiff_read16(data + offset, is_le);
    entry->type = tiff_read16(data + offset + 2, is_le);
    entry->count = tiff_read32(data + offset + 4, is_le);
    entry->value_offset = tiff_read32(data + offset + 8, is_le);
    return 0;
}

int tiff_type_size(uint16_t type) {
    if (type == 1 || type == 2 || type == 6 || type == 7) { return 1; }
    if (type == 3 || type == 8) { return 2; }
    if (type == 4 || type == 9 || type == 11) { return 4; }
    if (type == 5 || type == 10 || type == 12) { return 8; }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C596: TIFF tag parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C596: Output should not be empty");
    assert!(
        code.contains("fn tiff_parse_header"),
        "C596: Should contain tiff_parse_header function"
    );
}

#[test]
fn c597_pdf_object_parser() {
    let c_code = r#"
#define PDF_OBJ_NULL    0
#define PDF_OBJ_BOOL    1
#define PDF_OBJ_INT     2
#define PDF_OBJ_REAL    3
#define PDF_OBJ_STRING  4
#define PDF_OBJ_NAME    5
#define PDF_OBJ_ARRAY   6
#define PDF_OBJ_DICT    7
#define PDF_OBJ_REF     8

typedef struct {
    int type;
    int int_val;
    float real_val;
    int str_start;
    int str_len;
} pdf_object_t;

int pdf_is_whitespace(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f';
}

int pdf_is_delimiter(char c) {
    return c == '(' || c == ')' || c == '<' || c == '>'
        || c == '[' || c == ']' || c == '{' || c == '}'
        || c == '/' || c == '%';
}

int pdf_skip_whitespace(const char *data, int pos, int len) {
    while (pos < len && pdf_is_whitespace(data[pos])) {
        pos++;
    }
    if (pos < len && data[pos] == '%') {
        while (pos < len && data[pos] != '\n' && data[pos] != '\r') {
            pos++;
        }
        while (pos < len && pdf_is_whitespace(data[pos])) {
            pos++;
        }
    }
    return pos;
}

int pdf_parse_name(const char *data, int pos, int len, pdf_object_t *obj) {
    if (pos >= len || data[pos] != '/') {
        return -1;
    }
    obj->type = PDF_OBJ_NAME;
    obj->str_start = pos + 1;
    pos++;
    while (pos < len && !pdf_is_whitespace(data[pos])
           && !pdf_is_delimiter(data[pos])) {
        pos++;
    }
    obj->str_len = pos - obj->str_start;
    return pos;
}

int pdf_parse_integer(const char *data, int pos, int len, pdf_object_t *obj) {
    int sign = 1;
    int val = 0;
    if (pos < len && data[pos] == '-') {
        sign = -1;
        pos++;
    } else if (pos < len && data[pos] == '+') {
        pos++;
    }
    int started = 0;
    while (pos < len && data[pos] >= '0' && data[pos] <= '9') {
        val = val * 10 + (data[pos] - '0');
        pos++;
        started = 1;
    }
    if (!started) {
        return -1;
    }
    obj->type = PDF_OBJ_INT;
    obj->int_val = sign * val;
    return pos;
}

int pdf_parse_literal_string(const char *data, int pos, int len, pdf_object_t *obj) {
    if (pos >= len || data[pos] != '(') {
        return -1;
    }
    pos++;
    obj->type = PDF_OBJ_STRING;
    obj->str_start = pos;
    int depth = 1;
    while (pos < len && depth > 0) {
        if (data[pos] == '\\') {
            pos++;
        } else if (data[pos] == '(') {
            depth++;
        } else if (data[pos] == ')') {
            depth--;
        }
        if (depth > 0) {
            pos++;
        }
    }
    obj->str_len = pos - obj->str_start;
    if (pos < len) {
        pos++;
    }
    return pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C597: PDF object parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C597: Output should not be empty");
    assert!(
        code.contains("fn pdf_parse_name"),
        "C597: Should contain pdf_parse_name function"
    );
}

#[test]
fn c598_xml_tag_tokenizer() {
    let c_code = r#"
#define XML_TOK_TEXT     0
#define XML_TOK_OPEN     1
#define XML_TOK_CLOSE    2
#define XML_TOK_SELF     3
#define XML_TOK_COMMENT  4
#define XML_TOK_PI       5
#define XML_TOK_EOF      6

typedef struct {
    int type;
    int start;
    int len;
    int name_start;
    int name_len;
} xml_token_t;

int xml_is_name_char(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
        || (c >= '0' && c <= '9') || c == '_' || c == '-'
        || c == '.' || c == ':';
}

int xml_is_name_start(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
        || c == '_' || c == ':';
}

int xml_next_token(const char *data, int pos, int len, xml_token_t *tok) {
    if (pos >= len) {
        tok->type = XML_TOK_EOF;
        tok->start = pos;
        tok->len = 0;
        return pos;
    }
    tok->start = pos;
    if (data[pos] == '<') {
        if (pos + 3 < len && data[pos + 1] == '!'
            && data[pos + 2] == '-' && data[pos + 3] == '-') {
            tok->type = XML_TOK_COMMENT;
            pos += 4;
            while (pos + 2 < len) {
                if (data[pos] == '-' && data[pos + 1] == '-'
                    && data[pos + 2] == '>') {
                    pos += 3;
                    break;
                }
                pos++;
            }
            tok->len = pos - tok->start;
            tok->name_start = 0;
            tok->name_len = 0;
            return pos;
        }
        if (pos + 1 < len && data[pos + 1] == '?') {
            tok->type = XML_TOK_PI;
            pos += 2;
            while (pos + 1 < len) {
                if (data[pos] == '?' && data[pos + 1] == '>') {
                    pos += 2;
                    break;
                }
                pos++;
            }
            tok->len = pos - tok->start;
            tok->name_start = 0;
            tok->name_len = 0;
            return pos;
        }
        if (pos + 1 < len && data[pos + 1] == '/') {
            tok->type = XML_TOK_CLOSE;
            pos += 2;
            tok->name_start = pos;
            while (pos < len && xml_is_name_char(data[pos])) {
                pos++;
            }
            tok->name_len = pos - tok->name_start;
            while (pos < len && data[pos] != '>') {
                pos++;
            }
            if (pos < len) { pos++; }
            tok->len = pos - tok->start;
            return pos;
        }
        tok->type = XML_TOK_OPEN;
        pos++;
        tok->name_start = pos;
        while (pos < len && xml_is_name_char(data[pos])) {
            pos++;
        }
        tok->name_len = pos - tok->name_start;
        while (pos < len && data[pos] != '>' && data[pos] != '/') {
            pos++;
        }
        if (pos < len && data[pos] == '/') {
            tok->type = XML_TOK_SELF;
            pos++;
        }
        if (pos < len && data[pos] == '>') {
            pos++;
        }
        tok->len = pos - tok->start;
        return pos;
    }
    tok->type = XML_TOK_TEXT;
    while (pos < len && data[pos] != '<') {
        pos++;
    }
    tok->len = pos - tok->start;
    tok->name_start = tok->start;
    tok->name_len = tok->len;
    return pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C598: XML tag tokenizer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C598: Output should not be empty");
    assert!(
        code.contains("fn xml_next_token"),
        "C598: Should contain xml_next_token function"
    );
}

#[test]
fn c599_s_expression_parser() {
    let c_code = r#"
#define SEXP_ATOM   0
#define SEXP_LIST   1
#define SEXP_STRING 2
#define SEXP_NUMBER 3
#define SEXP_EOF    4
#define SEXP_ERROR  5

typedef struct {
    int type;
    int start;
    int len;
    int depth;
} sexp_token_t;

int sexp_is_whitespace(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

int sexp_is_atom_char(char c) {
    return c != '\0' && !sexp_is_whitespace(c)
        && c != '(' && c != ')' && c != '"' && c != ';';
}

int sexp_skip_whitespace(const char *data, int pos, int len) {
    while (pos < len) {
        if (sexp_is_whitespace(data[pos])) {
            pos++;
        } else if (data[pos] == ';') {
            while (pos < len && data[pos] != '\n') {
                pos++;
            }
        } else {
            break;
        }
    }
    return pos;
}

int sexp_next_token(const char *data, int pos, int len,
                    int depth, sexp_token_t *tok) {
    pos = sexp_skip_whitespace(data, pos, len);
    if (pos >= len) {
        tok->type = SEXP_EOF;
        tok->start = pos;
        tok->len = 0;
        tok->depth = depth;
        return pos;
    }
    tok->depth = depth;
    tok->start = pos;
    if (data[pos] == '(') {
        tok->type = SEXP_LIST;
        tok->len = 1;
        return pos + 1;
    }
    if (data[pos] == ')') {
        tok->type = SEXP_LIST;
        tok->len = 1;
        tok->depth = depth - 1;
        return pos + 1;
    }
    if (data[pos] == '"') {
        tok->type = SEXP_STRING;
        pos++;
        while (pos < len && data[pos] != '"') {
            if (data[pos] == '\\') {
                pos++;
            }
            pos++;
        }
        if (pos < len) {
            pos++;
        }
        tok->len = pos - tok->start;
        return pos;
    }
    if ((data[pos] >= '0' && data[pos] <= '9')
        || (data[pos] == '-' && pos + 1 < len
            && data[pos + 1] >= '0' && data[pos + 1] <= '9')) {
        tok->type = SEXP_NUMBER;
        if (data[pos] == '-') { pos++; }
        while (pos < len && data[pos] >= '0' && data[pos] <= '9') {
            pos++;
        }
        if (pos < len && data[pos] == '.') {
            pos++;
            while (pos < len && data[pos] >= '0' && data[pos] <= '9') {
                pos++;
            }
        }
        tok->len = pos - tok->start;
        return pos;
    }
    tok->type = SEXP_ATOM;
    while (pos < len && sexp_is_atom_char(data[pos])) {
        pos++;
    }
    tok->len = pos - tok->start;
    return pos;
}

int sexp_count_top_level(const char *data, int len) {
    int count = 0;
    int pos = 0;
    int depth = 0;
    sexp_token_t tok;
    while (pos < len) {
        pos = sexp_next_token(data, pos, len, depth, &tok);
        if (tok.type == SEXP_EOF) {
            break;
        }
        if (tok.type == SEXP_LIST && data[tok.start] == '(') {
            if (depth == 0) {
                count++;
            }
            depth++;
        } else if (tok.type == SEXP_LIST && data[tok.start] == ')') {
            depth--;
        } else if (depth == 0) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C599: S-expression parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C599: Output should not be empty");
    assert!(
        code.contains("fn sexp_next_token"),
        "C599: Should contain sexp_next_token function"
    );
}

#[test]
fn c600_protocol_buffer_varint() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

int pb_encode_varint(uint8_t *buf, int max_len, uint64_t value) {
    int i = 0;
    while (value > 0x7F && i < max_len) {
        buf[i] = (uint8_t)((value & 0x7F) | 0x80);
        value = value >> 7;
        i++;
    }
    if (i < max_len) {
        buf[i] = (uint8_t)(value & 0x7F);
        i++;
    }
    return i;
}

int pb_decode_varint(const uint8_t *buf, int len, uint64_t *value) {
    *value = 0;
    int i = 0;
    int shift = 0;
    while (i < len && i < 10) {
        uint64_t byte = (uint64_t)buf[i];
        *value = *value | ((byte & 0x7F) << shift);
        i++;
        if ((byte & 0x80) == 0) {
            return i;
        }
        shift += 7;
    }
    return -1;
}

int pb_varint_size(uint64_t value) {
    int size = 1;
    while (value > 0x7F) {
        value = value >> 7;
        size++;
    }
    return size;
}

uint32_t pb_zigzag_encode(int value) {
    return (uint32_t)((value << 1) ^ (value >> 31));
}

int pb_zigzag_decode(uint32_t value) {
    return (int)((value >> 1) ^ (uint32_t)(-(int)(value & 1)));
}

int pb_read_tag(const uint8_t *buf, int len, uint32_t *field_num,
                uint32_t *wire_type) {
    uint64_t tag_val;
    int consumed = pb_decode_varint(buf, len, &tag_val);
    if (consumed <= 0) {
        return -1;
    }
    *field_num = (uint32_t)(tag_val >> 3);
    *wire_type = (uint32_t)(tag_val & 0x07);
    return consumed;
}

int pb_encode_tag(uint8_t *buf, int max_len, uint32_t field_num,
                  uint32_t wire_type) {
    uint64_t tag_val = ((uint64_t)field_num << 3) | (uint64_t)(wire_type & 0x07);
    return pb_encode_varint(buf, max_len, tag_val);
}

int pb_skip_field(const uint8_t *buf, int pos, int len, uint32_t wire_type) {
    if (wire_type == 0) {
        uint64_t dummy;
        int consumed = pb_decode_varint(buf + pos, len - pos, &dummy);
        if (consumed <= 0) { return -1; }
        return pos + consumed;
    }
    if (wire_type == 1) {
        return pos + 8;
    }
    if (wire_type == 5) {
        return pos + 4;
    }
    if (wire_type == 2) {
        uint64_t field_len;
        int consumed = pb_decode_varint(buf + pos, len - pos, &field_len);
        if (consumed <= 0) { return -1; }
        return pos + consumed + (int)field_len;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C600: Protocol buffer varint should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C600: Output should not be empty");
    assert!(
        code.contains("fn pb_encode_varint"),
        "C600: Should contain pb_encode_varint function"
    );
}
