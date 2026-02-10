//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C826-C850: Image Processing domain -- the kind of C code found in image
//! processing libraries, computer vision systems, and graphics pipelines.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world image processing patterns commonly
//! found in OpenCV, stb_image, ImageMagick, GIMP, and similar projects --
//! all expressed as valid C99.
//!
//! Organization:
//! - C826-C830: Basic operations (grayscale, histogram, equalization, blur, Sobel)
//! - C831-C835: Detection/morphology (Canny, Otsu, erosion, dilation, connected components)
//! - C836-C840: Geometric/filter (rotation, scaling, RGB-HSV, convolution, median)
//! - C841-C845: Advanced filters (LoG, Harris, template matching, RLE, dithering)
//! - C846-C850: Advanced algorithms (seam carving, Hough, pyramid, alpha blend, gamma)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C826-C830: Basic Image Operations
// ============================================================================

#[test]
fn c826_grayscale_conversion_rgb_to_luminance() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} img_rgb_pixel_t;

uint8_t img_rgb_to_gray(img_rgb_pixel_t pixel) {
    int luminance = (int)(0.299f * pixel.r + 0.587f * pixel.g + 0.114f * pixel.b);
    if (luminance > 255) luminance = 255;
    if (luminance < 0) luminance = 0;
    return (uint8_t)luminance;
}

void img_grayscale_convert(const img_rgb_pixel_t *src, uint8_t *dst, int width, int height) {
    int i;
    int total = width * height;
    for (i = 0; i < total; i++) {
        dst[i] = img_rgb_to_gray(src[i]);
    }
}

float img_average_brightness(const uint8_t *gray, int count) {
    int i;
    float sum = 0.0f;
    for (i = 0; i < count; i++) {
        sum = sum + (float)gray[i];
    }
    return sum / (float)count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C826: Grayscale conversion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C826: Output should not be empty");
    assert!(
        code.contains("fn img_rgb_to_gray"),
        "C826: Should contain img_rgb_to_gray function"
    );
    assert!(
        code.contains("fn img_grayscale_convert"),
        "C826: Should contain img_grayscale_convert function"
    );
    assert!(
        code.contains("fn img_average_brightness"),
        "C826: Should contain img_average_brightness function"
    );
}

#[test]
fn c827_image_histogram_computation() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int bins[256];
    int total_pixels;
} img_histogram_t;

void img_histogram_init(img_histogram_t *hist) {
    int i;
    for (i = 0; i < 256; i++) {
        hist->bins[i] = 0;
    }
    hist->total_pixels = 0;
}

void img_histogram_compute(img_histogram_t *hist, const uint8_t *pixels, int count) {
    int i;
    img_histogram_init(hist);
    for (i = 0; i < count; i++) {
        hist->bins[pixels[i]]++;
    }
    hist->total_pixels = count;
}

int img_histogram_max_bin(const img_histogram_t *hist) {
    int max_val = 0;
    int max_idx = 0;
    int i;
    for (i = 0; i < 256; i++) {
        if (hist->bins[i] > max_val) {
            max_val = hist->bins[i];
            max_idx = i;
        }
    }
    return max_idx;
}

float img_histogram_mean(const img_histogram_t *hist) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < 256; i++) {
        sum = sum + (float)(i * hist->bins[i]);
    }
    return sum / (float)hist->total_pixels;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C827: Image histogram should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C827: Output should not be empty");
    assert!(
        code.contains("fn img_histogram_init"),
        "C827: Should contain img_histogram_init function"
    );
    assert!(
        code.contains("fn img_histogram_compute"),
        "C827: Should contain img_histogram_compute function"
    );
    assert!(
        code.contains("fn img_histogram_max_bin"),
        "C827: Should contain img_histogram_max_bin function"
    );
}

#[test]
fn c828_histogram_equalization() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_equalize_compute_cdf(const int *histogram, float *cdf, int total_pixels) {
    int i;
    float cumulative = 0.0f;
    for (i = 0; i < 256; i++) {
        cumulative = cumulative + (float)histogram[i];
        cdf[i] = cumulative / (float)total_pixels;
    }
}

void img_equalize_apply(const uint8_t *src, uint8_t *dst, const float *cdf, int count) {
    int i;
    for (i = 0; i < count; i++) {
        int val = (int)(cdf[src[i]] * 255.0f);
        if (val > 255) val = 255;
        if (val < 0) val = 0;
        dst[i] = (uint8_t)val;
    }
}

float img_equalize_contrast_measure(const uint8_t *pixels, int count) {
    int i;
    float mean = 0.0f;
    float variance = 0.0f;
    float diff;
    for (i = 0; i < count; i++) {
        mean = mean + (float)pixels[i];
    }
    mean = mean / (float)count;
    for (i = 0; i < count; i++) {
        diff = (float)pixels[i] - mean;
        variance = variance + diff * diff;
    }
    return variance / (float)count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C828: Histogram equalization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C828: Output should not be empty");
    assert!(
        code.contains("fn img_equalize_compute_cdf"),
        "C828: Should contain img_equalize_compute_cdf function"
    );
    assert!(
        code.contains("fn img_equalize_apply"),
        "C828: Should contain img_equalize_apply function"
    );
}

#[test]
fn c829_gaussian_blur_3x3_and_5x5() {
    let c_code = r#"
typedef unsigned char uint8_t;

uint8_t img_clamp_u8(int val) {
    if (val < 0) return 0;
    if (val > 255) return 255;
    return (uint8_t)val;
}

void img_gaussian_blur_3x3(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int sum = 0;
            sum = sum + 1 * src[(y-1)*w + (x-1)];
            sum = sum + 2 * src[(y-1)*w + x];
            sum = sum + 1 * src[(y-1)*w + (x+1)];
            sum = sum + 2 * src[y*w + (x-1)];
            sum = sum + 4 * src[y*w + x];
            sum = sum + 2 * src[y*w + (x+1)];
            sum = sum + 1 * src[(y+1)*w + (x-1)];
            sum = sum + 2 * src[(y+1)*w + x];
            sum = sum + 1 * src[(y+1)*w + (x+1)];
            dst[y*w + x] = img_clamp_u8(sum / 16);
        }
    }
}

void img_gaussian_blur_5x5(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    int kernel[5][5];
    kernel[0][0] = 1; kernel[0][1] = 4; kernel[0][2] = 6; kernel[0][3] = 4; kernel[0][4] = 1;
    kernel[1][0] = 4; kernel[1][1] = 16; kernel[1][2] = 24; kernel[1][3] = 16; kernel[1][4] = 4;
    kernel[2][0] = 6; kernel[2][1] = 24; kernel[2][2] = 36; kernel[2][3] = 24; kernel[2][4] = 6;
    kernel[3][0] = 4; kernel[3][1] = 16; kernel[3][2] = 24; kernel[3][3] = 16; kernel[3][4] = 4;
    kernel[4][0] = 1; kernel[4][1] = 4; kernel[4][2] = 6; kernel[4][3] = 4; kernel[4][4] = 1;

    for (y = 2; y < h - 2; y++) {
        for (x = 2; x < w - 2; x++) {
            int sum = 0;
            for (ky = -2; ky <= 2; ky++) {
                for (kx = -2; kx <= 2; kx++) {
                    sum = sum + kernel[ky+2][kx+2] * src[(y+ky)*w + (x+kx)];
                }
            }
            dst[y*w + x] = img_clamp_u8(sum / 256);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C829: Gaussian blur should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C829: Output should not be empty");
    assert!(
        code.contains("fn img_gaussian_blur_3x3"),
        "C829: Should contain img_gaussian_blur_3x3 function"
    );
    assert!(
        code.contains("fn img_gaussian_blur_5x5"),
        "C829: Should contain img_gaussian_blur_5x5 function"
    );
}

#[test]
fn c830_sobel_edge_detection() {
    let c_code = r#"
typedef unsigned char uint8_t;

int img_sobel_abs(int val) {
    if (val < 0) return -val;
    return val;
}

uint8_t img_sobel_clamp(int val) {
    if (val < 0) return 0;
    if (val > 255) return 255;
    return (uint8_t)val;
}

void img_sobel_detect(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int gx = 0;
            int gy = 0;
            gx = gx - 1 * src[(y-1)*w + (x-1)];
            gx = gx + 1 * src[(y-1)*w + (x+1)];
            gx = gx - 2 * src[y*w + (x-1)];
            gx = gx + 2 * src[y*w + (x+1)];
            gx = gx - 1 * src[(y+1)*w + (x-1)];
            gx = gx + 1 * src[(y+1)*w + (x+1)];

            gy = gy - 1 * src[(y-1)*w + (x-1)];
            gy = gy - 2 * src[(y-1)*w + x];
            gy = gy - 1 * src[(y-1)*w + (x+1)];
            gy = gy + 1 * src[(y+1)*w + (x-1)];
            gy = gy + 2 * src[(y+1)*w + x];
            gy = gy + 1 * src[(y+1)*w + (x+1)];

            int mag = img_sobel_abs(gx) + img_sobel_abs(gy);
            dst[y*w + x] = img_sobel_clamp(mag);
        }
    }
}

int img_sobel_edge_count(const uint8_t *edges, int count, uint8_t threshold) {
    int i;
    int edge_count = 0;
    for (i = 0; i < count; i++) {
        if (edges[i] > threshold) {
            edge_count++;
        }
    }
    return edge_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C830: Sobel edge detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C830: Output should not be empty");
    assert!(
        code.contains("fn img_sobel_detect"),
        "C830: Should contain img_sobel_detect function"
    );
    assert!(
        code.contains("fn img_sobel_edge_count"),
        "C830: Should contain img_sobel_edge_count function"
    );
}

// ============================================================================
// C831-C835: Detection and Morphology
// ============================================================================

#[test]
fn c831_canny_edge_detector_simplified() {
    let c_code = r#"
typedef unsigned char uint8_t;

uint8_t img_canny_clamp(int val) {
    if (val < 0) return 0;
    if (val > 255) return 255;
    return (uint8_t)val;
}

void img_canny_gradient(const uint8_t *src, int *gx_out, int *gy_out, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int idx = y * w + x;
            gx_out[idx] = (int)src[y*w + (x+1)] - (int)src[y*w + (x-1)];
            gy_out[idx] = (int)src[(y+1)*w + x] - (int)src[(y-1)*w + x];
        }
    }
}

void img_canny_magnitude(const int *gx, const int *gy, uint8_t *mag, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int idx = y * w + x;
            int abs_gx = gx[idx];
            int abs_gy = gy[idx];
            if (abs_gx < 0) abs_gx = -abs_gx;
            if (abs_gy < 0) abs_gy = -abs_gy;
            mag[idx] = img_canny_clamp(abs_gx + abs_gy);
        }
    }
}

void img_canny_threshold(const uint8_t *mag, uint8_t *edges, int count,
                         uint8_t low_thresh, uint8_t high_thresh) {
    int i;
    for (i = 0; i < count; i++) {
        if (mag[i] >= high_thresh) {
            edges[i] = 255;
        } else if (mag[i] >= low_thresh) {
            edges[i] = 128;
        } else {
            edges[i] = 0;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C831: Canny edge detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C831: Output should not be empty");
    assert!(
        code.contains("fn img_canny_gradient"),
        "C831: Should contain img_canny_gradient function"
    );
    assert!(
        code.contains("fn img_canny_magnitude"),
        "C831: Should contain img_canny_magnitude function"
    );
    assert!(
        code.contains("fn img_canny_threshold"),
        "C831: Should contain img_canny_threshold function"
    );
}

#[test]
fn c832_image_thresholding_otsu_method() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_otsu_compute_histogram(const uint8_t *pixels, int count, int *histogram) {
    int i;
    for (i = 0; i < 256; i++) {
        histogram[i] = 0;
    }
    for (i = 0; i < count; i++) {
        histogram[pixels[i]]++;
    }
}

uint8_t img_otsu_find_threshold(const int *histogram, int total_pixels) {
    int i;
    float sum = 0.0f;
    float sum_b = 0.0f;
    int w_b = 0;
    int w_f = 0;
    float max_var = 0.0f;
    uint8_t threshold = 0;

    for (i = 0; i < 256; i++) {
        sum = sum + (float)(i * histogram[i]);
    }

    for (i = 0; i < 256; i++) {
        w_b = w_b + histogram[i];
        if (w_b == 0) continue;
        w_f = total_pixels - w_b;
        if (w_f == 0) break;

        sum_b = sum_b + (float)(i * histogram[i]);
        float mean_b = sum_b / (float)w_b;
        float mean_f = (sum - sum_b) / (float)w_f;
        float diff = mean_b - mean_f;
        float between_var = (float)w_b * (float)w_f * diff * diff;

        if (between_var > max_var) {
            max_var = between_var;
            threshold = (uint8_t)i;
        }
    }
    return threshold;
}

void img_otsu_apply_threshold(const uint8_t *src, uint8_t *dst, int count, uint8_t thresh) {
    int i;
    for (i = 0; i < count; i++) {
        if (src[i] >= thresh) {
            dst[i] = 255;
        } else {
            dst[i] = 0;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C832: Otsu thresholding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C832: Output should not be empty");
    assert!(
        code.contains("fn img_otsu_compute_histogram"),
        "C832: Should contain img_otsu_compute_histogram function"
    );
    assert!(
        code.contains("fn img_otsu_find_threshold"),
        "C832: Should contain img_otsu_find_threshold function"
    );
    assert!(
        code.contains("fn img_otsu_apply_threshold"),
        "C832: Should contain img_otsu_apply_threshold function"
    );
}

#[test]
fn c833_morphological_erosion() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_erode_3x3(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            uint8_t min_val = 255;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    uint8_t val = src[(y + ky) * w + (x + kx)];
                    if (val < min_val) {
                        min_val = val;
                    }
                }
            }
            dst[y * w + x] = min_val;
        }
    }
}

void img_erode_binary(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int all_set = 1;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    if (src[(y + ky) * w + (x + kx)] == 0) {
                        all_set = 0;
                    }
                }
            }
            dst[y * w + x] = all_set ? 255 : 0;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C833: Morphological erosion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C833: Output should not be empty");
    assert!(
        code.contains("fn img_erode_3x3"),
        "C833: Should contain img_erode_3x3 function"
    );
    assert!(
        code.contains("fn img_erode_binary"),
        "C833: Should contain img_erode_binary function"
    );
}

#[test]
fn c834_morphological_dilation() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_dilate_3x3(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            uint8_t max_val = 0;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    uint8_t val = src[(y + ky) * w + (x + kx)];
                    if (val > max_val) {
                        max_val = val;
                    }
                }
            }
            dst[y * w + x] = max_val;
        }
    }
}

void img_dilate_binary(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int any_set = 0;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    if (src[(y + ky) * w + (x + kx)] != 0) {
                        any_set = 1;
                    }
                }
            }
            dst[y * w + x] = any_set ? 255 : 0;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C834: Morphological dilation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C834: Output should not be empty");
    assert!(
        code.contains("fn img_dilate_3x3"),
        "C834: Should contain img_dilate_3x3 function"
    );
    assert!(
        code.contains("fn img_dilate_binary"),
        "C834: Should contain img_dilate_binary function"
    );
}

#[test]
fn c835_connected_component_labeling() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_ccl_init_labels(int *labels, int count) {
    int i;
    for (i = 0; i < count; i++) {
        labels[i] = 0;
    }
}

int img_ccl_find_root(int *parent, int x) {
    while (parent[x] != x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    return x;
}

void img_ccl_union(int *parent, int a, int b) {
    int ra = img_ccl_find_root(parent, a);
    int rb = img_ccl_find_root(parent, b);
    if (ra != rb) {
        if (ra < rb) {
            parent[rb] = ra;
        } else {
            parent[ra] = rb;
        }
    }
}

int img_ccl_count_components(const int *labels, int count) {
    int max_label = 0;
    int i;
    for (i = 0; i < count; i++) {
        if (labels[i] > max_label) {
            max_label = labels[i];
        }
    }
    return max_label;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C835: Connected component labeling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C835: Output should not be empty");
    assert!(
        code.contains("fn img_ccl_find_root"),
        "C835: Should contain img_ccl_find_root function"
    );
    assert!(
        code.contains("fn img_ccl_union"),
        "C835: Should contain img_ccl_union function"
    );
    assert!(
        code.contains("fn img_ccl_count_components"),
        "C835: Should contain img_ccl_count_components function"
    );
}

// ============================================================================
// C836-C840: Geometric Transforms and Filters
// ============================================================================

#[test]
fn c836_image_rotation_nearest_neighbor() {
    let c_code = r#"
typedef unsigned char uint8_t;

float img_rot_cos_approx(float angle) {
    float a2 = angle * angle;
    return 1.0f - a2 * 0.5f + a2 * a2 * 0.041667f;
}

float img_rot_sin_approx(float angle) {
    float a2 = angle * angle;
    return angle - angle * a2 * 0.166667f + angle * a2 * a2 * 0.008333f;
}

void img_rotate_nearest(const uint8_t *src, uint8_t *dst, int w, int h, float angle) {
    int x;
    int y;
    float cos_a = img_rot_cos_approx(angle);
    float sin_a = img_rot_sin_approx(angle);
    float cx = (float)w * 0.5f;
    float cy = (float)h * 0.5f;

    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            float dx = (float)x - cx;
            float dy = (float)y - cy;
            int sx = (int)(dx * cos_a + dy * sin_a + cx);
            int sy = (int)(-dx * sin_a + dy * cos_a + cy);

            if (sx >= 0 && sx < w && sy >= 0 && sy < h) {
                dst[y * w + x] = src[sy * w + sx];
            } else {
                dst[y * w + x] = 0;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C836: Image rotation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C836: Output should not be empty");
    assert!(
        code.contains("fn img_rotate_nearest"),
        "C836: Should contain img_rotate_nearest function"
    );
    assert!(
        code.contains("fn img_rot_cos_approx"),
        "C836: Should contain img_rot_cos_approx function"
    );
}

#[test]
fn c837_image_scaling_bilinear_interpolation() {
    let c_code = r#"
typedef unsigned char uint8_t;

uint8_t img_bilinear_sample(const uint8_t *src, int w, int h, float fx, float fy) {
    int x0 = (int)fx;
    int y0 = (int)fy;
    int x1 = x0 + 1;
    int y1 = y0 + 1;
    float dx = fx - (float)x0;
    float dy = fy - (float)y0;

    if (x1 >= w) x1 = w - 1;
    if (y1 >= h) y1 = h - 1;
    if (x0 < 0) x0 = 0;
    if (y0 < 0) y0 = 0;

    float top = (float)src[y0 * w + x0] * (1.0f - dx) + (float)src[y0 * w + x1] * dx;
    float bot = (float)src[y1 * w + x0] * (1.0f - dx) + (float)src[y1 * w + x1] * dx;
    float val = top * (1.0f - dy) + bot * dy;

    if (val > 255.0f) val = 255.0f;
    if (val < 0.0f) val = 0.0f;
    return (uint8_t)val;
}

void img_scale_bilinear(const uint8_t *src, uint8_t *dst,
                        int src_w, int src_h, int dst_w, int dst_h) {
    int x;
    int y;
    float x_ratio = (float)src_w / (float)dst_w;
    float y_ratio = (float)src_h / (float)dst_h;
    for (y = 0; y < dst_h; y++) {
        for (x = 0; x < dst_w; x++) {
            float fx = (float)x * x_ratio;
            float fy = (float)y * y_ratio;
            dst[y * dst_w + x] = img_bilinear_sample(src, src_w, src_h, fx, fy);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C837: Image scaling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C837: Output should not be empty");
    assert!(
        code.contains("fn img_bilinear_sample"),
        "C837: Should contain img_bilinear_sample function"
    );
    assert!(
        code.contains("fn img_scale_bilinear"),
        "C837: Should contain img_scale_bilinear function"
    );
}

#[test]
fn c838_color_space_rgb_to_hsv() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    float h;
    float s;
    float v;
} img_hsv_t;

img_hsv_t img_rgb_to_hsv(uint8_t r, uint8_t g, uint8_t b) {
    img_hsv_t hsv;
    float rf = (float)r / 255.0f;
    float gf = (float)g / 255.0f;
    float bf = (float)b / 255.0f;

    float max_c = rf;
    float min_c = rf;
    if (gf > max_c) max_c = gf;
    if (bf > max_c) max_c = bf;
    if (gf < min_c) min_c = gf;
    if (bf < min_c) min_c = bf;

    float delta = max_c - min_c;
    hsv.v = max_c;

    if (delta < 0.00001f) {
        hsv.s = 0.0f;
        hsv.h = 0.0f;
        return hsv;
    }

    hsv.s = delta / max_c;

    if (rf >= max_c) {
        hsv.h = (gf - bf) / delta;
    } else if (gf >= max_c) {
        hsv.h = 2.0f + (bf - rf) / delta;
    } else {
        hsv.h = 4.0f + (rf - gf) / delta;
    }

    hsv.h = hsv.h * 60.0f;
    if (hsv.h < 0.0f) {
        hsv.h = hsv.h + 360.0f;
    }
    return hsv;
}

int img_hsv_is_red(img_hsv_t hsv) {
    return (hsv.h < 30.0f || hsv.h > 330.0f) && hsv.s > 0.5f && hsv.v > 0.3f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C838: RGB to HSV conversion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C838: Output should not be empty");
    assert!(
        code.contains("fn img_rgb_to_hsv"),
        "C838: Should contain img_rgb_to_hsv function"
    );
    assert!(
        code.contains("fn img_hsv_is_red"),
        "C838: Should contain img_hsv_is_red function"
    );
}

#[test]
fn c839_image_convolution_generic_kernel() {
    let c_code = r#"
typedef unsigned char uint8_t;

uint8_t img_conv_clamp(int val) {
    if (val < 0) return 0;
    if (val > 255) return 255;
    return (uint8_t)val;
}

void img_convolve_3x3(const uint8_t *src, uint8_t *dst, int w, int h,
                      const float *kernel, float divisor) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            float sum = 0.0f;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    float pixel = (float)src[(y + ky) * w + (x + kx)];
                    float weight = kernel[(ky + 1) * 3 + (kx + 1)];
                    sum = sum + pixel * weight;
                }
            }
            dst[y * w + x] = img_conv_clamp((int)(sum / divisor));
        }
    }
}

void img_sharpen_kernel(float *kernel) {
    kernel[0] = 0.0f;  kernel[1] = -1.0f; kernel[2] = 0.0f;
    kernel[3] = -1.0f; kernel[4] = 5.0f;  kernel[5] = -1.0f;
    kernel[6] = 0.0f;  kernel[7] = -1.0f; kernel[8] = 0.0f;
}

void img_emboss_kernel(float *kernel) {
    kernel[0] = -2.0f; kernel[1] = -1.0f; kernel[2] = 0.0f;
    kernel[3] = -1.0f; kernel[4] = 1.0f;  kernel[5] = 1.0f;
    kernel[6] = 0.0f;  kernel[7] = 1.0f;  kernel[8] = 2.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C839: Image convolution should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C839: Output should not be empty");
    assert!(
        code.contains("fn img_convolve_3x3"),
        "C839: Should contain img_convolve_3x3 function"
    );
    assert!(
        code.contains("fn img_sharpen_kernel"),
        "C839: Should contain img_sharpen_kernel function"
    );
}

#[test]
fn c840_median_filter_salt_and_pepper() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_median_sort_insert(uint8_t *arr, int len) {
    int i;
    int j;
    for (i = 1; i < len; i++) {
        uint8_t key = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j] > key) {
            arr[j + 1] = arr[j];
            j = j - 1;
        }
        arr[j + 1] = key;
    }
}

void img_median_filter_3x3(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    int ky;
    int kx;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            uint8_t window[9];
            int idx = 0;
            for (ky = -1; ky <= 1; ky++) {
                for (kx = -1; kx <= 1; kx++) {
                    window[idx] = src[(y + ky) * w + (x + kx)];
                    idx++;
                }
            }
            img_median_sort_insert(window, 9);
            dst[y * w + x] = window[4];
        }
    }
}

int img_median_count_noise(const uint8_t *pixels, int count) {
    int noise = 0;
    int i;
    for (i = 0; i < count; i++) {
        if (pixels[i] == 0 || pixels[i] == 255) {
            noise++;
        }
    }
    return noise;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C840: Median filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C840: Output should not be empty");
    assert!(
        code.contains("fn img_median_filter_3x3"),
        "C840: Should contain img_median_filter_3x3 function"
    );
    assert!(
        code.contains("fn img_median_sort_insert"),
        "C840: Should contain img_median_sort_insert function"
    );
}

// ============================================================================
// C841-C845: Advanced Filters and Analysis
// ============================================================================

#[test]
fn c841_laplacian_of_gaussian() {
    let c_code = r#"
typedef unsigned char uint8_t;

int img_log_abs(int val) {
    if (val < 0) return -val;
    return val;
}

uint8_t img_log_clamp(int val) {
    if (val < 0) return 0;
    if (val > 255) return 255;
    return (uint8_t)val;
}

void img_laplacian_3x3(const uint8_t *src, uint8_t *dst, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int lap = 0;
            lap = lap - (int)src[(y-1)*w + x];
            lap = lap - (int)src[y*w + (x-1)];
            lap = lap + 4 * (int)src[y*w + x];
            lap = lap - (int)src[y*w + (x+1)];
            lap = lap - (int)src[(y+1)*w + x];
            dst[y * w + x] = img_log_clamp(img_log_abs(lap));
        }
    }
}

int img_log_zero_crossings(const uint8_t *laplacian, int w, int h, uint8_t threshold) {
    int x;
    int y;
    int crossings = 0;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            if (laplacian[y*w + x] > threshold) {
                crossings++;
            }
        }
    }
    return crossings;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C841: Laplacian of Gaussian should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C841: Output should not be empty");
    assert!(
        code.contains("fn img_laplacian_3x3"),
        "C841: Should contain img_laplacian_3x3 function"
    );
    assert!(
        code.contains("fn img_log_zero_crossings"),
        "C841: Should contain img_log_zero_crossings function"
    );
}

#[test]
fn c842_harris_corner_detector() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    float ix2;
    float iy2;
    float ixy;
} img_harris_tensor_t;

void img_harris_compute_gradients(const uint8_t *src, float *ix, float *iy, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int idx = y * w + x;
            ix[idx] = (float)((int)src[y*w + (x+1)] - (int)src[y*w + (x-1)]) * 0.5f;
            iy[idx] = (float)((int)src[(y+1)*w + x] - (int)src[(y-1)*w + x]) * 0.5f;
        }
    }
}

float img_harris_response(float ix2, float iy2, float ixy, float k) {
    float det = ix2 * iy2 - ixy * ixy;
    float trace = ix2 + iy2;
    return det - k * trace * trace;
}

int img_harris_count_corners(const float *response, int count, float threshold) {
    int i;
    int corners = 0;
    for (i = 0; i < count; i++) {
        if (response[i] > threshold) {
            corners++;
        }
    }
    return corners;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C842: Harris corner detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C842: Output should not be empty");
    assert!(
        code.contains("fn img_harris_compute_gradients"),
        "C842: Should contain img_harris_compute_gradients function"
    );
    assert!(
        code.contains("fn img_harris_response"),
        "C842: Should contain img_harris_response function"
    );
    assert!(
        code.contains("fn img_harris_count_corners"),
        "C842: Should contain img_harris_count_corners function"
    );
}

#[test]
fn c843_template_matching_sad() {
    let c_code = r#"
typedef unsigned char uint8_t;

int img_tmatch_abs(int val) {
    if (val < 0) return -val;
    return val;
}

int img_tmatch_sad(const uint8_t *img, int img_w,
                   const uint8_t *tmpl, int tmpl_w, int tmpl_h,
                   int offset_x, int offset_y) {
    int tx;
    int ty;
    int sad = 0;
    for (ty = 0; ty < tmpl_h; ty++) {
        for (tx = 0; tx < tmpl_w; tx++) {
            int img_val = (int)img[(offset_y + ty) * img_w + (offset_x + tx)];
            int tmpl_val = (int)tmpl[ty * tmpl_w + tx];
            sad = sad + img_tmatch_abs(img_val - tmpl_val);
        }
    }
    return sad;
}

void img_tmatch_find_best(const uint8_t *img, int img_w, int img_h,
                          const uint8_t *tmpl, int tmpl_w, int tmpl_h,
                          int *best_x, int *best_y) {
    int x;
    int y;
    int min_sad = 2147483647;
    *best_x = 0;
    *best_y = 0;
    for (y = 0; y <= img_h - tmpl_h; y++) {
        for (x = 0; x <= img_w - tmpl_w; x++) {
            int sad = img_tmatch_sad(img, img_w, tmpl, tmpl_w, tmpl_h, x, y);
            if (sad < min_sad) {
                min_sad = sad;
                *best_x = x;
                *best_y = y;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C843: Template matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C843: Output should not be empty");
    assert!(
        code.contains("fn img_tmatch_sad"),
        "C843: Should contain img_tmatch_sad function"
    );
    assert!(
        code.contains("fn img_tmatch_find_best"),
        "C843: Should contain img_tmatch_find_best function"
    );
}

#[test]
fn c844_run_length_encoding_for_images() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t value;
    uint16_t count;
} img_rle_pair_t;

int img_rle_encode(const uint8_t *pixels, int pixel_count,
                   img_rle_pair_t *runs, int max_runs) {
    int i;
    int run_count = 0;
    if (pixel_count == 0) return 0;

    uint8_t current = pixels[0];
    uint16_t count = 1;

    for (i = 1; i < pixel_count; i++) {
        if (pixels[i] == current && count < 65535) {
            count++;
        } else {
            if (run_count < max_runs) {
                runs[run_count].value = current;
                runs[run_count].count = count;
                run_count++;
            }
            current = pixels[i];
            count = 1;
        }
    }
    if (run_count < max_runs) {
        runs[run_count].value = current;
        runs[run_count].count = count;
        run_count++;
    }
    return run_count;
}

int img_rle_decode(const img_rle_pair_t *runs, int run_count,
                   uint8_t *pixels, int max_pixels) {
    int i;
    int j;
    int pos = 0;
    for (i = 0; i < run_count; i++) {
        for (j = 0; j < runs[i].count && pos < max_pixels; j++) {
            pixels[pos] = runs[i].value;
            pos++;
        }
    }
    return pos;
}

float img_rle_compression_ratio(int original_size, int run_count) {
    int rle_size = run_count * 3;
    return (float)original_size / (float)rle_size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C844: Run-length encoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C844: Output should not be empty");
    assert!(
        code.contains("fn img_rle_encode"),
        "C844: Should contain img_rle_encode function"
    );
    assert!(
        code.contains("fn img_rle_decode"),
        "C844: Should contain img_rle_decode function"
    );
}

#[test]
fn c845_image_dithering_floyd_steinberg() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_dither_floyd_steinberg(uint8_t *pixels, int w, int h) {
    int x;
    int y;
    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            int old_pixel = (int)pixels[y * w + x];
            int new_pixel;
            int error;
            if (old_pixel > 127) {
                new_pixel = 255;
            } else {
                new_pixel = 0;
            }
            pixels[y * w + x] = (uint8_t)new_pixel;
            error = old_pixel - new_pixel;

            if (x + 1 < w) {
                int val = (int)pixels[y * w + (x + 1)] + error * 7 / 16;
                if (val < 0) val = 0;
                if (val > 255) val = 255;
                pixels[y * w + (x + 1)] = (uint8_t)val;
            }
            if (y + 1 < h && x > 0) {
                int val = (int)pixels[(y + 1) * w + (x - 1)] + error * 3 / 16;
                if (val < 0) val = 0;
                if (val > 255) val = 255;
                pixels[(y + 1) * w + (x - 1)] = (uint8_t)val;
            }
            if (y + 1 < h) {
                int val = (int)pixels[(y + 1) * w + x] + error * 5 / 16;
                if (val < 0) val = 0;
                if (val > 255) val = 255;
                pixels[(y + 1) * w + x] = (uint8_t)val;
            }
            if (y + 1 < h && x + 1 < w) {
                int val = (int)pixels[(y + 1) * w + (x + 1)] + error * 1 / 16;
                if (val < 0) val = 0;
                if (val > 255) val = 255;
                pixels[(y + 1) * w + (x + 1)] = (uint8_t)val;
            }
        }
    }
}

int img_dither_count_black(const uint8_t *pixels, int count) {
    int i;
    int black = 0;
    for (i = 0; i < count; i++) {
        if (pixels[i] == 0) black++;
    }
    return black;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C845: Floyd-Steinberg dithering should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C845: Output should not be empty");
    assert!(
        code.contains("fn img_dither_floyd_steinberg"),
        "C845: Should contain img_dither_floyd_steinberg function"
    );
    assert!(
        code.contains("fn img_dither_count_black"),
        "C845: Should contain img_dither_count_black function"
    );
}

// ============================================================================
// C846-C850: Advanced Algorithms
// ============================================================================

#[test]
fn c846_seam_carving_energy_map() {
    let c_code = r#"
typedef unsigned char uint8_t;

int img_seam_abs(int val) {
    if (val < 0) return -val;
    return val;
}

void img_seam_energy_map(const uint8_t *gray, int *energy, int w, int h) {
    int x;
    int y;
    for (y = 1; y < h - 1; y++) {
        for (x = 1; x < w - 1; x++) {
            int dx = (int)gray[y*w + (x+1)] - (int)gray[y*w + (x-1)];
            int dy = (int)gray[(y+1)*w + x] - (int)gray[(y-1)*w + x];
            energy[y * w + x] = img_seam_abs(dx) + img_seam_abs(dy);
        }
    }
}

void img_seam_cumulative_energy(const int *energy, int *cumulative, int w, int h) {
    int x;
    int y;
    for (x = 0; x < w; x++) {
        cumulative[x] = energy[x];
    }
    for (y = 1; y < h; y++) {
        for (x = 0; x < w; x++) {
            int min_above = cumulative[(y-1)*w + x];
            if (x > 0 && cumulative[(y-1)*w + (x-1)] < min_above) {
                min_above = cumulative[(y-1)*w + (x-1)];
            }
            if (x < w - 1 && cumulative[(y-1)*w + (x+1)] < min_above) {
                min_above = cumulative[(y-1)*w + (x+1)];
            }
            cumulative[y*w + x] = energy[y*w + x] + min_above;
        }
    }
}

int img_seam_find_min_col(const int *cumulative, int w, int last_row_offset) {
    int x;
    int min_energy = cumulative[last_row_offset];
    int min_col = 0;
    for (x = 1; x < w; x++) {
        if (cumulative[last_row_offset + x] < min_energy) {
            min_energy = cumulative[last_row_offset + x];
            min_col = x;
        }
    }
    return min_col;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C846: Seam carving energy map should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C846: Output should not be empty");
    assert!(
        code.contains("fn img_seam_energy_map"),
        "C846: Should contain img_seam_energy_map function"
    );
    assert!(
        code.contains("fn img_seam_cumulative_energy"),
        "C846: Should contain img_seam_cumulative_energy function"
    );
    assert!(
        code.contains("fn img_seam_find_min_col"),
        "C846: Should contain img_seam_find_min_col function"
    );
}

#[test]
fn c847_hough_transform_line_detection() {
    let c_code = r#"
typedef unsigned char uint8_t;

float img_hough_cos_approx(float angle) {
    float a2 = angle * angle;
    return 1.0f - a2 * 0.5f + a2 * a2 * 0.041667f;
}

float img_hough_sin_approx(float angle) {
    float a2 = angle * angle;
    return angle - angle * a2 * 0.166667f + angle * a2 * a2 * 0.008333f;
}

void img_hough_accumulate(const uint8_t *edges, int w, int h,
                          int *accumulator, int num_rho, int num_theta,
                          float rho_step, float theta_step) {
    int x;
    int y;
    int t;
    for (y = 0; y < h; y++) {
        for (x = 0; x < w; x++) {
            if (edges[y * w + x] > 128) {
                for (t = 0; t < num_theta; t++) {
                    float theta = (float)t * theta_step;
                    float rho = (float)x * img_hough_cos_approx(theta) +
                                (float)y * img_hough_sin_approx(theta);
                    int rho_idx = (int)(rho / rho_step) + num_rho / 2;
                    if (rho_idx >= 0 && rho_idx < num_rho) {
                        accumulator[rho_idx * num_theta + t]++;
                    }
                }
            }
        }
    }
}

int img_hough_find_peaks(const int *accumulator, int num_rho, int num_theta, int threshold) {
    int r;
    int t;
    int peaks = 0;
    for (r = 0; r < num_rho; r++) {
        for (t = 0; t < num_theta; t++) {
            if (accumulator[r * num_theta + t] > threshold) {
                peaks++;
            }
        }
    }
    return peaks;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C847: Hough transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C847: Output should not be empty");
    assert!(
        code.contains("fn img_hough_accumulate"),
        "C847: Should contain img_hough_accumulate function"
    );
    assert!(
        code.contains("fn img_hough_find_peaks"),
        "C847: Should contain img_hough_find_peaks function"
    );
}

#[test]
fn c848_image_pyramid_downscale() {
    let c_code = r#"
typedef unsigned char uint8_t;

void img_pyramid_downsample_2x(const uint8_t *src, uint8_t *dst,
                                int src_w, int src_h) {
    int x;
    int y;
    int dst_w = src_w / 2;
    int dst_h = src_h / 2;
    for (y = 0; y < dst_h; y++) {
        for (x = 0; x < dst_w; x++) {
            int sx = x * 2;
            int sy = y * 2;
            int sum = (int)src[sy * src_w + sx] +
                      (int)src[sy * src_w + (sx + 1)] +
                      (int)src[(sy + 1) * src_w + sx] +
                      (int)src[(sy + 1) * src_w + (sx + 1)];
            dst[y * dst_w + x] = (uint8_t)(sum / 4);
        }
    }
}

int img_pyramid_num_levels(int width, int height, int min_size) {
    int levels = 0;
    int w = width;
    int h = height;
    while (w >= min_size && h >= min_size) {
        levels++;
        w = w / 2;
        h = h / 2;
    }
    return levels;
}

float img_pyramid_detail_ratio(const uint8_t *fine, const uint8_t *coarse,
                               int fine_count, int coarse_count) {
    int i;
    float fine_sum = 0.0f;
    float coarse_sum = 0.0f;
    for (i = 0; i < fine_count; i++) {
        fine_sum = fine_sum + (float)fine[i];
    }
    for (i = 0; i < coarse_count; i++) {
        coarse_sum = coarse_sum + (float)coarse[i];
    }
    float fine_avg = fine_sum / (float)fine_count;
    float coarse_avg = coarse_sum / (float)coarse_count;
    if (coarse_avg < 0.001f) return 0.0f;
    return fine_avg / coarse_avg;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C848: Image pyramid should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C848: Output should not be empty");
    assert!(
        code.contains("fn img_pyramid_downsample_2x"),
        "C848: Should contain img_pyramid_downsample_2x function"
    );
    assert!(
        code.contains("fn img_pyramid_num_levels"),
        "C848: Should contain img_pyramid_num_levels function"
    );
}

#[test]
fn c849_alpha_blending() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} img_rgba_pixel_t;

img_rgba_pixel_t img_alpha_blend_pixel(img_rgba_pixel_t fg, img_rgba_pixel_t bg) {
    img_rgba_pixel_t result;
    float alpha_f = (float)fg.a / 255.0f;
    float inv_alpha = 1.0f - alpha_f;

    int r = (int)((float)fg.r * alpha_f + (float)bg.r * inv_alpha);
    int g = (int)((float)fg.g * alpha_f + (float)bg.g * inv_alpha);
    int b = (int)((float)fg.b * alpha_f + (float)bg.b * inv_alpha);

    if (r > 255) r = 255;
    if (g > 255) g = 255;
    if (b > 255) b = 255;

    result.r = (uint8_t)r;
    result.g = (uint8_t)g;
    result.b = (uint8_t)b;
    result.a = 255;
    return result;
}

void img_alpha_blend_row(const img_rgba_pixel_t *fg, const img_rgba_pixel_t *bg,
                         img_rgba_pixel_t *dst, int width) {
    int i;
    for (i = 0; i < width; i++) {
        dst[i] = img_alpha_blend_pixel(fg[i], bg[i]);
    }
}

float img_alpha_coverage(const img_rgba_pixel_t *pixels, int count) {
    int i;
    int opaque = 0;
    for (i = 0; i < count; i++) {
        if (pixels[i].a > 128) {
            opaque++;
        }
    }
    return (float)opaque / (float)count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C849: Alpha blending should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C849: Output should not be empty");
    assert!(
        code.contains("fn img_alpha_blend_pixel"),
        "C849: Should contain img_alpha_blend_pixel function"
    );
    assert!(
        code.contains("fn img_alpha_blend_row"),
        "C849: Should contain img_alpha_blend_row function"
    );
    assert!(
        code.contains("fn img_alpha_coverage"),
        "C849: Should contain img_alpha_coverage function"
    );
}

#[test]
fn c850_gamma_correction() {
    let c_code = r#"
typedef unsigned char uint8_t;

float img_gamma_pow_approx(float base, float exp) {
    float result = 1.0f;
    float term = 1.0f;
    float log_base = base - 1.0f;
    int i;
    for (i = 1; i <= 8; i++) {
        term = term * (exp - (float)(i - 1)) * log_base / (float)i;
        result = result + term;
    }
    if (result < 0.0f) result = 0.0f;
    return result;
}

void img_gamma_build_lut(uint8_t *lut, float gamma) {
    int i;
    float inv_gamma = 1.0f / gamma;
    for (i = 0; i < 256; i++) {
        float normalized = (float)i / 255.0f;
        float corrected = img_gamma_pow_approx(normalized, inv_gamma);
        int val = (int)(corrected * 255.0f);
        if (val > 255) val = 255;
        if (val < 0) val = 0;
        lut[i] = (uint8_t)val;
    }
}

void img_gamma_apply_lut(const uint8_t *src, uint8_t *dst, int count, const uint8_t *lut) {
    int i;
    for (i = 0; i < count; i++) {
        dst[i] = lut[src[i]];
    }
}

float img_gamma_mean_brightness(const uint8_t *pixels, int count) {
    int i;
    float sum = 0.0f;
    for (i = 0; i < count; i++) {
        sum = sum + (float)pixels[i];
    }
    return sum / (float)count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C850: Gamma correction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C850: Output should not be empty");
    assert!(
        code.contains("fn img_gamma_build_lut"),
        "C850: Should contain img_gamma_build_lut function"
    );
    assert!(
        code.contains("fn img_gamma_apply_lut"),
        "C850: Should contain img_gamma_apply_lut function"
    );
    assert!(
        code.contains("fn img_gamma_mean_brightness"),
        "C850: Should contain img_gamma_mean_brightness function"
    );
}
