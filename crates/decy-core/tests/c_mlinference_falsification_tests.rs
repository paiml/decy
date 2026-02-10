//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C776-C800: ML/Inference domain patterns -- the kind of C code found in
//! inference engines, ML model runtimes, and numerical computing libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world ML inference patterns commonly found
//! in TensorFlow Lite, ONNX Runtime, llama.cpp, and custom inference
//! engines -- all expressed as valid self-contained C99.
//!
//! Organization:
//! - C776-C780: Core layer operations (dense, relu, softmax, cross-entropy, batchnorm)
//! - C781-C785: Convolution and pooling (conv1d, conv2d, maxpool, avgpool, dropout)
//! - C786-C790: Sequence models and normalization (LSTM, GRU, attention, layernorm, embedding)
//! - C791-C795: Quantization and classical ML (quantize, dequantize, pruning, KNN, decision tree)
//! - C796-C800: Ensemble and activation (random forest, kmeans, PCA, sigmoid, tanh)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C776-C780: Core Layer Operations
// ============================================================================

#[test]
fn c776_dense_layer_forward_pass() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_dense_forward(const float *input, const float *weights,
                       const float *bias, float *output,
                       int in_features, int out_features) {
    int i, j;
    for (i = 0; i < out_features; i++) {
        float sum = 0.0f;
        for (j = 0; j < in_features; j++) {
            sum += weights[i * in_features + j] * input[j];
        }
        output[i] = sum + bias[i];
    }
}

void mli_dense_batch_forward(const float *input, const float *weights,
                             const float *bias, float *output,
                             int batch_size, int in_features, int out_features) {
    int b, i, j;
    for (b = 0; b < batch_size; b++) {
        for (i = 0; i < out_features; i++) {
            float sum = 0.0f;
            for (j = 0; j < in_features; j++) {
                sum += weights[i * in_features + j] * input[b * in_features + j];
            }
            output[b * out_features + i] = sum + bias[i];
        }
    }
}

int main(void) {
    float input[4] = {1.0f, 2.0f, 3.0f, 4.0f};
    float weights[8] = {0.1f, 0.2f, 0.3f, 0.4f, 0.5f, 0.6f, 0.7f, 0.8f};
    float bias[2] = {0.01f, 0.02f};
    float output[2];
    mli_dense_forward(input, weights, bias, output, 4, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C776: Dense layer forward pass should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C776: should produce non-empty output");
    assert!(
        code.contains("fn mli_dense_forward"),
        "C776: Should contain mli_dense_forward function"
    );
}

#[test]
fn c777_relu_activation_function() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_relu(float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (data[i] < 0.0f) {
            data[i] = 0.0f;
        }
    }
}

void mli_leaky_relu(float *data, int len, float alpha) {
    int i;
    for (i = 0; i < len; i++) {
        if (data[i] < 0.0f) {
            data[i] = alpha * data[i];
        }
    }
}

float mli_relu_scalar(float x) {
    return x > 0.0f ? x : 0.0f;
}

void mli_relu6(float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (data[i] < 0.0f) {
            data[i] = 0.0f;
        } else if (data[i] > 6.0f) {
            data[i] = 6.0f;
        }
    }
}

int main(void) {
    float arr[4] = {-1.0f, 2.0f, -3.0f, 4.0f};
    mli_relu(arr, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C777: ReLU activation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C777: should produce non-empty output");
    assert!(
        code.contains("fn mli_relu"),
        "C777: Should contain mli_relu function"
    );
}

#[test]
fn c778_softmax_numerical_stability() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_exp_approx(float x) {
    if (x > 88.0f) return 1.0e38f;
    if (x < -88.0f) return 0.0f;
    float result = 1.0f;
    float term = 1.0f;
    int i;
    for (i = 1; i <= 10; i++) {
        term *= x / (float)i;
        result += term;
    }
    return result;
}

void mli_softmax(const float *input, float *output, int len) {
    int i;
    float max_val = input[0];
    for (i = 1; i < len; i++) {
        if (input[i] > max_val) {
            max_val = input[i];
        }
    }

    float sum = 0.0f;
    for (i = 0; i < len; i++) {
        output[i] = mli_exp_approx(input[i] - max_val);
        sum += output[i];
    }

    for (i = 0; i < len; i++) {
        output[i] /= sum;
    }
}

int main(void) {
    float logits[4] = {1.0f, 2.0f, 3.0f, 4.0f};
    float probs[4];
    mli_softmax(logits, probs, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C778: Softmax with numerical stability should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C778: should produce non-empty output");
    assert!(
        code.contains("fn mli_softmax"),
        "C778: Should contain mli_softmax function"
    );
}

#[test]
fn c779_cross_entropy_loss() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_log_approx(float x) {
    if (x <= 0.0f) return -1.0e38f;
    float result = 0.0f;
    float y = (x - 1.0f) / (x + 1.0f);
    float y2 = y * y;
    float term = y;
    int i;
    for (i = 0; i < 10; i++) {
        result += term / (float)(2 * i + 1);
        term *= y2;
    }
    return 2.0f * result;
}

float mli_cross_entropy(const float *predictions, const int *targets,
                        int batch_size, int num_classes) {
    float total_loss = 0.0f;
    int b, c;
    for (b = 0; b < batch_size; b++) {
        int target_class = targets[b];
        float pred = predictions[b * num_classes + target_class];
        if (pred < 1.0e-7f) pred = 1.0e-7f;
        total_loss -= mli_log_approx(pred);
    }
    return total_loss / (float)batch_size;
}

float mli_binary_cross_entropy(const float *pred, const float *target, int len) {
    float loss = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        float p = pred[i];
        if (p < 1.0e-7f) p = 1.0e-7f;
        if (p > 1.0f - 1.0e-7f) p = 1.0f - 1.0e-7f;
        loss -= target[i] * mli_log_approx(p) + (1.0f - target[i]) * mli_log_approx(1.0f - p);
    }
    return loss / (float)len;
}

int main(void) {
    float preds[6] = {0.7f, 0.2f, 0.1f, 0.1f, 0.1f, 0.8f};
    int targets[2] = {0, 2};
    float loss = mli_cross_entropy(preds, targets, 2, 3);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C779: Cross-entropy loss should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C779: should produce non-empty output");
    assert!(
        code.contains("fn mli_cross_entropy"),
        "C779: Should contain mli_cross_entropy function"
    );
}

#[test]
fn c780_batch_normalization_inference() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_batchnorm_infer(const float *input, float *output,
                         const float *gamma, const float *beta,
                         const float *running_mean, const float *running_var,
                         int batch_size, int channels, int spatial_size,
                         float epsilon) {
    int b, c, s;
    for (b = 0; b < batch_size; b++) {
        for (c = 0; c < channels; c++) {
            float inv_std = 1.0f;
            float var_val = running_var[c] + epsilon;
            float approx = var_val;
            int iter;
            for (iter = 0; iter < 5; iter++) {
                approx = 0.5f * (approx + var_val / approx);
            }
            inv_std = 1.0f / approx;

            for (s = 0; s < spatial_size; s++) {
                int idx = b * channels * spatial_size + c * spatial_size + s;
                float normalized = (input[idx] - running_mean[c]) * inv_std;
                output[idx] = gamma[c] * normalized + beta[c];
            }
        }
    }
}

int main(void) {
    float input[8] = {1.0f, 2.0f, 3.0f, 4.0f, 5.0f, 6.0f, 7.0f, 8.0f};
    float output[8];
    float gamma[2] = {1.0f, 1.0f};
    float beta[2] = {0.0f, 0.0f};
    float mean[2] = {2.5f, 6.5f};
    float var[2] = {1.25f, 1.25f};
    mli_batchnorm_infer(input, output, gamma, beta, mean, var, 1, 2, 4, 1.0e-5f);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C780: Batch normalization inference should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C780: should produce non-empty output");
    assert!(
        code.contains("fn mli_batchnorm_infer"),
        "C780: Should contain mli_batchnorm_infer function"
    );
}

// ============================================================================
// C781-C785: Convolution and Pooling
// ============================================================================

#[test]
fn c781_conv1d_forward_pass() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_conv1d(const float *input, const float *kernel, const float *bias,
                float *output, int in_len, int kernel_size,
                int in_channels, int out_channels) {
    int out_len = in_len - kernel_size + 1;
    int oc, pos, ic, k;
    for (oc = 0; oc < out_channels; oc++) {
        for (pos = 0; pos < out_len; pos++) {
            float sum = bias[oc];
            for (ic = 0; ic < in_channels; ic++) {
                for (k = 0; k < kernel_size; k++) {
                    int w_idx = oc * in_channels * kernel_size + ic * kernel_size + k;
                    int i_idx = ic * in_len + pos + k;
                    sum += kernel[w_idx] * input[i_idx];
                }
            }
            output[oc * out_len + pos] = sum;
        }
    }
}

int main(void) {
    float input[8] = {1.0f, 2.0f, 3.0f, 4.0f, 5.0f, 6.0f, 7.0f, 8.0f};
    float kernel[6] = {0.1f, 0.2f, 0.3f, 0.4f, 0.5f, 0.6f};
    float bias[2] = {0.01f, 0.02f};
    float output[12];
    mli_conv1d(input, kernel, bias, output, 8, 3, 1, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C781: 1D convolution forward pass should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C781: should produce non-empty output");
    assert!(
        code.contains("fn mli_conv1d"),
        "C781: Should contain mli_conv1d function"
    );
}

#[test]
fn c782_conv2d_im2col_forward() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_im2col(const float *input, float *col_buf,
                int height, int width, int channels,
                int kh, int kw, int stride) {
    int out_h = (height - kh) / stride + 1;
    int out_w = (width - kw) / stride + 1;
    int c, h, w, khi, kwi;
    int col_idx = 0;
    for (c = 0; c < channels; c++) {
        for (khi = 0; khi < kh; khi++) {
            for (kwi = 0; kwi < kw; kwi++) {
                for (h = 0; h < out_h; h++) {
                    for (w = 0; w < out_w; w++) {
                        int in_h = h * stride + khi;
                        int in_w = w * stride + kwi;
                        col_buf[col_idx] = input[c * height * width + in_h * width + in_w];
                        col_idx++;
                    }
                }
            }
        }
    }
}

void mli_conv2d_forward(const float *input, const float *weights,
                        const float *bias, float *output,
                        int h, int w, int ic, int oc,
                        int kh, int kw, int stride) {
    int out_h = (h - kh) / stride + 1;
    int out_w = (w - kw) / stride + 1;
    int patch_size = ic * kh * kw;
    float col_buf[256];
    int o, p, pos;

    mli_im2col(input, col_buf, h, w, ic, kh, kw, stride);

    for (o = 0; o < oc; o++) {
        for (pos = 0; pos < out_h * out_w; pos++) {
            float sum = bias[o];
            for (p = 0; p < patch_size; p++) {
                sum += weights[o * patch_size + p] * col_buf[p * out_h * out_w + pos];
            }
            output[o * out_h * out_w + pos] = sum;
        }
    }
}

int main(void) {
    float input[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    float weights[4] = {0.1f, 0.2f, 0.3f, 0.4f};
    float bias_arr[1] = {0.0f};
    float output[9];
    mli_conv2d_forward(input, weights, bias_arr, output, 4, 4, 1, 1, 2, 2, 1);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C782: 2D convolution im2col forward should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C782: should produce non-empty output");
    assert!(
        code.contains("fn mli_conv2d_forward"),
        "C782: Should contain mli_conv2d_forward function"
    );
}

#[test]
fn c783_max_pooling_2d() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_maxpool2d(const float *input, float *output,
                   int height, int width, int channels,
                   int pool_h, int pool_w, int stride) {
    int out_h = (height - pool_h) / stride + 1;
    int out_w = (width - pool_w) / stride + 1;
    int c, oh, ow, ph, pw;

    for (c = 0; c < channels; c++) {
        for (oh = 0; oh < out_h; oh++) {
            for (ow = 0; ow < out_w; ow++) {
                float max_val = -1.0e38f;
                for (ph = 0; ph < pool_h; ph++) {
                    for (pw = 0; pw < pool_w; pw++) {
                        int ih = oh * stride + ph;
                        int iw = ow * stride + pw;
                        float val = input[c * height * width + ih * width + iw];
                        if (val > max_val) {
                            max_val = val;
                        }
                    }
                }
                output[c * out_h * out_w + oh * out_w + ow] = max_val;
            }
        }
    }
}

int main(void) {
    float input[16] = {1,5,2,6,3,7,4,8,9,13,10,14,11,15,12,16};
    float output[4];
    mli_maxpool2d(input, output, 4, 4, 1, 2, 2, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C783: Max pooling 2D should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C783: should produce non-empty output");
    assert!(
        code.contains("fn mli_maxpool2d"),
        "C783: Should contain mli_maxpool2d function"
    );
}

#[test]
fn c784_average_pooling_2d() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_avgpool2d(const float *input, float *output,
                   int height, int width, int channels,
                   int pool_h, int pool_w, int stride) {
    int out_h = (height - pool_h) / stride + 1;
    int out_w = (width - pool_w) / stride + 1;
    int c, oh, ow, ph, pw;
    float pool_area = (float)(pool_h * pool_w);

    for (c = 0; c < channels; c++) {
        for (oh = 0; oh < out_h; oh++) {
            for (ow = 0; ow < out_w; ow++) {
                float sum = 0.0f;
                for (ph = 0; ph < pool_h; ph++) {
                    for (pw = 0; pw < pool_w; pw++) {
                        int ih = oh * stride + ph;
                        int iw = ow * stride + pw;
                        sum += input[c * height * width + ih * width + iw];
                    }
                }
                output[c * out_h * out_w + oh * out_w + ow] = sum / pool_area;
            }
        }
    }
}

void mli_global_avgpool(const float *input, float *output,
                        int height, int width, int channels) {
    int c, h, w;
    float spatial = (float)(height * width);
    for (c = 0; c < channels; c++) {
        float sum = 0.0f;
        for (h = 0; h < height; h++) {
            for (w = 0; w < width; w++) {
                sum += input[c * height * width + h * width + w];
            }
        }
        output[c] = sum / spatial;
    }
}

int main(void) {
    float input[16] = {1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16};
    float output[4];
    mli_avgpool2d(input, output, 4, 4, 1, 2, 2, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C784: Average pooling 2D should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C784: should produce non-empty output");
    assert!(
        code.contains("fn mli_avgpool2d"),
        "C784: Should contain mli_avgpool2d function"
    );
}

#[test]
fn c785_dropout_inference_passthrough() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_dropout_inference(const float *input, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = input[i];
    }
}

void mli_dropout_train(const float *input, float *output,
                       const int *mask, int len, float keep_prob) {
    int i;
    float scale = 1.0f / keep_prob;
    for (i = 0; i < len; i++) {
        if (mask[i]) {
            output[i] = input[i] * scale;
        } else {
            output[i] = 0.0f;
        }
    }
}

int mli_is_training = 0;

void mli_dropout(const float *input, float *output,
                 const int *mask, int len, float keep_prob) {
    if (mli_is_training) {
        mli_dropout_train(input, output, mask, len, keep_prob);
    } else {
        mli_dropout_inference(input, output, len);
    }
}

int main(void) {
    float input[4] = {1.0f, 2.0f, 3.0f, 4.0f};
    float output[4];
    int mask[4] = {1, 0, 1, 1};
    mli_dropout(input, output, mask, 4, 0.5f);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C785: Dropout inference passthrough should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C785: should produce non-empty output");
    assert!(
        code.contains("fn mli_dropout"),
        "C785: Should contain mli_dropout function"
    );
}

// ============================================================================
// C786-C790: Sequence Models and Normalization
// ============================================================================

#[test]
fn c786_lstm_cell_forward() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_sigmoid_s(float x) {
    if (x > 20.0f) return 1.0f;
    if (x < -20.0f) return 0.0f;
    float ex = 1.0f;
    float term = 1.0f;
    float neg_x = -x;
    int i;
    for (i = 1; i <= 10; i++) {
        term *= neg_x / (float)i;
        ex += term;
    }
    return 1.0f / (1.0f + ex);
}

float mli_tanh_s(float x) {
    float pos = mli_sigmoid_s(2.0f * x);
    return 2.0f * pos - 1.0f;
}

void mli_lstm_cell(const float *x, const float *h_prev, const float *c_prev,
                   const float *Wf, const float *Wi, const float *Wc, const float *Wo,
                   const float *bf, const float *bi, const float *bc, const float *bo,
                   float *h_out, float *c_out, int input_size, int hidden_size) {
    int total = input_size + hidden_size;
    float concat[32];
    int i, j;

    for (i = 0; i < input_size; i++) concat[i] = x[i];
    for (i = 0; i < hidden_size; i++) concat[input_size + i] = h_prev[i];

    for (i = 0; i < hidden_size; i++) {
        float ft = bf[i], it = bi[i], ct = bc[i], ot = bo[i];
        for (j = 0; j < total; j++) {
            ft += Wf[i * total + j] * concat[j];
            it += Wi[i * total + j] * concat[j];
            ct += Wc[i * total + j] * concat[j];
            ot += Wo[i * total + j] * concat[j];
        }
        ft = mli_sigmoid_s(ft);
        it = mli_sigmoid_s(it);
        ct = mli_tanh_s(ct);
        ot = mli_sigmoid_s(ot);

        c_out[i] = ft * c_prev[i] + it * ct;
        h_out[i] = ot * mli_tanh_s(c_out[i]);
    }
}

int main(void) {
    float x[2] = {0.5f, 0.3f};
    float h[4] = {0.0f, 0.0f, 0.0f, 0.0f};
    float c[4] = {0.0f, 0.0f, 0.0f, 0.0f};
    float Wf[24] = {0};
    float Wi[24] = {0};
    float Wc[24] = {0};
    float Wo[24] = {0};
    float bf[4] = {0};
    float bi[4] = {0};
    float bc[4] = {0};
    float bo[4] = {0};
    float h_out[4], c_out[4];
    mli_lstm_cell(x, h, c, Wf, Wi, Wc, Wo, bf, bi, bc, bo, h_out, c_out, 2, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C786: LSTM cell forward pass should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C786: should produce non-empty output");
    assert!(
        code.contains("fn mli_lstm_cell"),
        "C786: Should contain mli_lstm_cell function"
    );
}

#[test]
fn c787_gru_cell_forward() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_gru_sigmoid(float x) {
    if (x > 20.0f) return 1.0f;
    if (x < -20.0f) return 0.0f;
    float ex = 1.0f;
    float term = 1.0f;
    float neg_x = -x;
    int i;
    for (i = 1; i <= 10; i++) {
        term *= neg_x / (float)i;
        ex += term;
    }
    return 1.0f / (1.0f + ex);
}

float mli_gru_tanh(float x) {
    float s = mli_gru_sigmoid(2.0f * x);
    return 2.0f * s - 1.0f;
}

void mli_gru_cell(const float *x, const float *h_prev,
                  const float *Wz, const float *Wr, const float *Wh,
                  const float *bz, const float *br, const float *bh,
                  float *h_out, int input_size, int hidden_size) {
    int total = input_size + hidden_size;
    float concat[32];
    int i, j;

    for (i = 0; i < input_size; i++) concat[i] = x[i];
    for (i = 0; i < hidden_size; i++) concat[input_size + i] = h_prev[i];

    for (i = 0; i < hidden_size; i++) {
        float z = bz[i], r = br[i];
        for (j = 0; j < total; j++) {
            z += Wz[i * total + j] * concat[j];
            r += Wr[i * total + j] * concat[j];
        }
        z = mli_gru_sigmoid(z);
        r = mli_gru_sigmoid(r);

        float rh_concat[32];
        int k;
        for (k = 0; k < input_size; k++) rh_concat[k] = x[k];
        for (k = 0; k < hidden_size; k++) rh_concat[input_size + k] = r * h_prev[k];

        float h_cand = bh[i];
        for (j = 0; j < total; j++) {
            h_cand += Wh[i * total + j] * rh_concat[j];
        }
        h_cand = mli_gru_tanh(h_cand);

        h_out[i] = (1.0f - z) * h_prev[i] + z * h_cand;
    }
}

int main(void) {
    float x[2] = {0.5f, 0.3f};
    float h[4] = {0.0f, 0.0f, 0.0f, 0.0f};
    float Wz[24] = {0};
    float Wr[24] = {0};
    float Wh[24] = {0};
    float bz[4] = {0};
    float br[4] = {0};
    float bh[4] = {0};
    float h_out[4];
    mli_gru_cell(x, h, Wz, Wr, Wh, bz, br, bh, h_out, 2, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C787: GRU cell forward pass should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C787: should produce non-empty output");
    assert!(
        code.contains("fn mli_gru_cell"),
        "C787: Should contain mli_gru_cell function"
    );
}

#[test]
fn c788_scaled_dot_product_attention() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_attn_exp(float x) {
    if (x > 88.0f) return 1.0e38f;
    if (x < -88.0f) return 0.0f;
    float r = 1.0f;
    float t = 1.0f;
    int i;
    for (i = 1; i <= 10; i++) {
        t *= x / (float)i;
        r += t;
    }
    return r;
}

void mli_attention(const float *Q, const float *K, const float *V,
                   float *output, int seq_len, int d_k) {
    float scores[64];
    int i, j, k;

    float scale = 1.0f;
    float dk_f = (float)d_k;
    float approx = dk_f;
    int iter;
    for (iter = 0; iter < 10; iter++) {
        approx = 0.5f * (approx + dk_f / approx);
    }
    scale = 1.0f / approx;

    for (i = 0; i < seq_len; i++) {
        for (j = 0; j < seq_len; j++) {
            float dot = 0.0f;
            for (k = 0; k < d_k; k++) {
                dot += Q[i * d_k + k] * K[j * d_k + k];
            }
            scores[i * seq_len + j] = dot * scale;
        }
    }

    for (i = 0; i < seq_len; i++) {
        float max_s = scores[i * seq_len];
        for (j = 1; j < seq_len; j++) {
            if (scores[i * seq_len + j] > max_s) max_s = scores[i * seq_len + j];
        }
        float sum = 0.0f;
        for (j = 0; j < seq_len; j++) {
            scores[i * seq_len + j] = mli_attn_exp(scores[i * seq_len + j] - max_s);
            sum += scores[i * seq_len + j];
        }
        for (j = 0; j < seq_len; j++) {
            scores[i * seq_len + j] /= sum;
        }
    }

    for (i = 0; i < seq_len; i++) {
        for (k = 0; k < d_k; k++) {
            float val = 0.0f;
            for (j = 0; j < seq_len; j++) {
                val += scores[i * seq_len + j] * V[j * d_k + k];
            }
            output[i * d_k + k] = val;
        }
    }
}

int main(void) {
    float Q[8] = {1,0,0,1, 0,1,1,0};
    float K[8] = {1,0,0,1, 0,1,1,0};
    float V[8] = {1,2,3,4, 5,6,7,8};
    float out[8];
    mli_attention(Q, K, V, out, 2, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C788: Scaled dot-product attention should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C788: should produce non-empty output");
    assert!(
        code.contains("fn mli_attention"),
        "C788: Should contain mli_attention function"
    );
}

#[test]
fn c789_layer_normalization() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_layer_norm(const float *input, float *output,
                    const float *gamma, const float *beta,
                    int batch_size, int features, float epsilon) {
    int b, f;
    for (b = 0; b < batch_size; b++) {
        float mean = 0.0f;
        for (f = 0; f < features; f++) {
            mean += input[b * features + f];
        }
        mean /= (float)features;

        float var = 0.0f;
        for (f = 0; f < features; f++) {
            float diff = input[b * features + f] - mean;
            var += diff * diff;
        }
        var /= (float)features;

        float inv_std = var + epsilon;
        float approx = inv_std;
        int iter;
        for (iter = 0; iter < 5; iter++) {
            approx = 0.5f * (approx + inv_std / approx);
        }
        inv_std = 1.0f / approx;

        for (f = 0; f < features; f++) {
            float norm = (input[b * features + f] - mean) * inv_std;
            output[b * features + f] = gamma[f] * norm + beta[f];
        }
    }
}

int main(void) {
    float input[6] = {1.0f, 2.0f, 3.0f, 4.0f, 5.0f, 6.0f};
    float output[6];
    float gamma[3] = {1.0f, 1.0f, 1.0f};
    float beta[3] = {0.0f, 0.0f, 0.0f};
    mli_layer_norm(input, output, gamma, beta, 2, 3, 1.0e-5f);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C789: Layer normalization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C789: should produce non-empty output");
    assert!(
        code.contains("fn mli_layer_norm"),
        "C789: Should contain mli_layer_norm function"
    );
}

#[test]
fn c790_embedding_lookup() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_embedding_lookup(const float *embedding_table,
                          const int *token_ids, float *output,
                          int num_tokens, int embed_dim) {
    int t, d;
    for (t = 0; t < num_tokens; t++) {
        int idx = token_ids[t];
        for (d = 0; d < embed_dim; d++) {
            output[t * embed_dim + d] = embedding_table[idx * embed_dim + d];
        }
    }
}

void mli_positional_encoding(float *embeddings, int seq_len, int embed_dim) {
    int pos, i;
    for (pos = 0; pos < seq_len; pos++) {
        for (i = 0; i < embed_dim; i++) {
            float angle = (float)pos / 1.0f;
            int k;
            for (k = 0; k < i; k++) {
                angle /= 10000.0f;
            }
            if (i % 2 == 0) {
                float sin_val = angle;
                float term = angle;
                int n;
                for (n = 1; n < 8; n++) {
                    term *= -angle * angle / (float)(2 * n * (2 * n + 1));
                    sin_val += term;
                }
                embeddings[pos * embed_dim + i] += sin_val;
            } else {
                float cos_val = 1.0f;
                float term2 = 1.0f;
                int n;
                for (n = 1; n < 8; n++) {
                    term2 *= -angle * angle / (float)(2 * n * (2 * n - 1));
                    cos_val += term2;
                }
                embeddings[pos * embed_dim + i] += cos_val;
            }
        }
    }
}

int main(void) {
    float table[12] = {0.1f,0.2f,0.3f, 0.4f,0.5f,0.6f, 0.7f,0.8f,0.9f, 1.0f,1.1f,1.2f};
    int tokens[3] = {0, 2, 1};
    float output[9];
    mli_embedding_lookup(table, tokens, output, 3, 3);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C790: Embedding lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C790: should produce non-empty output");
    assert!(
        code.contains("fn mli_embedding_lookup"),
        "C790: Should contain mli_embedding_lookup function"
    );
}

// ============================================================================
// C791-C795: Quantization and Classical ML
// ============================================================================

#[test]
fn c791_quantization_float_to_int8() {
    let c_code = r#"
typedef unsigned long size_t;
typedef signed char int8_t;

void mli_find_minmax(const float *data, int len, float *min_val, float *max_val) {
    int i;
    *min_val = data[0];
    *max_val = data[0];
    for (i = 1; i < len; i++) {
        if (data[i] < *min_val) *min_val = data[i];
        if (data[i] > *max_val) *max_val = data[i];
    }
}

void mli_quantize_symmetric(const float *input, int8_t *output,
                            float *scale, int len) {
    float min_val, max_val;
    mli_find_minmax(input, len, &min_val, &max_val);

    float abs_max = max_val;
    if (-min_val > abs_max) abs_max = -min_val;

    *scale = abs_max / 127.0f;
    if (*scale < 1.0e-10f) *scale = 1.0e-10f;

    float inv_scale = 127.0f / abs_max;
    int i;
    for (i = 0; i < len; i++) {
        float val = input[i] * inv_scale;
        if (val > 127.0f) val = 127.0f;
        if (val < -128.0f) val = -128.0f;
        output[i] = (int8_t)val;
    }
}

int main(void) {
    float input[4] = {-1.0f, 0.5f, 0.0f, 1.0f};
    int8_t output[4];
    float scale;
    mli_quantize_symmetric(input, output, &scale, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C791: Float to int8 quantization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C791: should produce non-empty output");
    assert!(
        code.contains("fn mli_quantize_symmetric"),
        "C791: Should contain mli_quantize_symmetric function"
    );
}

#[test]
fn c792_dequantization_int8_to_float() {
    let c_code = r#"
typedef unsigned long size_t;
typedef signed char int8_t;

void mli_dequantize_symmetric(const int8_t *input, float *output,
                              float scale, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = (float)input[i] * scale;
    }
}

void mli_dequantize_affine(const int8_t *input, float *output,
                           float scale, int zero_point, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = ((float)input[i] - (float)zero_point) * scale;
    }
}

float mli_quantized_dot(const int8_t *a, const int8_t *b, int len,
                        float scale_a, float scale_b) {
    int acc = 0;
    int i;
    for (i = 0; i < len; i++) {
        acc += (int)a[i] * (int)b[i];
    }
    return (float)acc * scale_a * scale_b;
}

int main(void) {
    int8_t qdata[4] = {-128, 0, 64, 127};
    float output[4];
    mli_dequantize_symmetric(qdata, output, 0.00787f, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C792: Int8 to float dequantization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C792: should produce non-empty output");
    assert!(
        code.contains("fn mli_dequantize_symmetric"),
        "C792: Should contain mli_dequantize_symmetric function"
    );
}

#[test]
fn c793_weight_pruning_magnitude() {
    let c_code = r#"
typedef unsigned long size_t;

int mli_count_nonzero(const float *weights, int len) {
    int count = 0;
    int i;
    for (i = 0; i < len; i++) {
        if (weights[i] != 0.0f) count++;
    }
    return count;
}

float mli_abs_f(float x) {
    return x < 0.0f ? -x : x;
}

void mli_prune_magnitude(float *weights, int len, float threshold) {
    int i;
    for (i = 0; i < len; i++) {
        if (mli_abs_f(weights[i]) < threshold) {
            weights[i] = 0.0f;
        }
    }
}

float mli_compute_sparsity(const float *weights, int len) {
    int zeros = 0;
    int i;
    for (i = 0; i < len; i++) {
        if (weights[i] == 0.0f) zeros++;
    }
    return (float)zeros / (float)len;
}

void mli_prune_topk(float *weights, int len, float target_sparsity) {
    float magnitudes[256];
    int i, j;
    for (i = 0; i < len; i++) {
        magnitudes[i] = mli_abs_f(weights[i]);
    }

    for (i = 0; i < len - 1; i++) {
        for (j = 0; j < len - i - 1; j++) {
            if (magnitudes[j] > magnitudes[j + 1]) {
                float tmp = magnitudes[j];
                magnitudes[j] = magnitudes[j + 1];
                magnitudes[j + 1] = tmp;
            }
        }
    }

    int cutoff = (int)((float)len * target_sparsity);
    if (cutoff >= len) cutoff = len - 1;
    float thresh = magnitudes[cutoff];
    mli_prune_magnitude(weights, len, thresh);
}

int main(void) {
    float weights[8] = {0.1f, -0.5f, 0.02f, 0.8f, -0.03f, 0.6f, -0.01f, 0.4f};
    mli_prune_magnitude(weights, 8, 0.05f);
    float sparsity = mli_compute_sparsity(weights, 8);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C793: Magnitude-based weight pruning should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C793: should produce non-empty output");
    assert!(
        code.contains("fn mli_prune_magnitude"),
        "C793: Should contain mli_prune_magnitude function"
    );
}

#[test]
fn c794_knn_classifier() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    float features[8];
    int label;
} mli_sample_t;

float mli_euclidean_dist(const float *a, const float *b, int dims) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < dims; i++) {
        float d = a[i] - b[i];
        sum += d * d;
    }
    return sum;
}

int mli_knn_classify(const mli_sample_t *train_data, int train_size,
                     const float *query, int dims, int k) {
    float distances[128];
    int indices[128];
    int i, j;

    for (i = 0; i < train_size; i++) {
        distances[i] = mli_euclidean_dist(train_data[i].features, query, dims);
        indices[i] = i;
    }

    for (i = 0; i < k; i++) {
        int min_idx = i;
        for (j = i + 1; j < train_size; j++) {
            if (distances[j] < distances[min_idx]) {
                min_idx = j;
            }
        }
        float tmp_d = distances[i];
        distances[i] = distances[min_idx];
        distances[min_idx] = tmp_d;
        int tmp_i = indices[i];
        indices[i] = indices[min_idx];
        indices[min_idx] = tmp_i;
    }

    int votes[16] = {0};
    for (i = 0; i < k; i++) {
        int label = train_data[indices[i]].label;
        if (label >= 0 && label < 16) votes[label]++;
    }

    int best_label = 0;
    int best_count = votes[0];
    for (i = 1; i < 16; i++) {
        if (votes[i] > best_count) {
            best_count = votes[i];
            best_label = i;
        }
    }
    return best_label;
}

int main(void) {
    mli_sample_t data[4];
    data[0].features[0] = 1.0f; data[0].features[1] = 2.0f; data[0].label = 0;
    data[1].features[0] = 2.0f; data[1].features[1] = 3.0f; data[1].label = 0;
    data[2].features[0] = 5.0f; data[2].features[1] = 6.0f; data[2].label = 1;
    data[3].features[0] = 6.0f; data[3].features[1] = 7.0f; data[3].label = 1;
    float query[2] = {4.0f, 5.0f};
    int result = mli_knn_classify(data, 4, query, 2, 3);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C794: KNN classifier should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C794: should produce non-empty output");
    assert!(
        code.contains("fn mli_knn_classify"),
        "C794: Should contain mli_knn_classify function"
    );
}

#[test]
fn c795_decision_tree_inference() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int feature_idx;
    float threshold;
    int left_child;
    int right_child;
    int class_label;
    int is_leaf;
} mli_tree_node_t;

int mli_tree_predict(const mli_tree_node_t *tree, const float *features) {
    int node_idx = 0;
    while (!tree[node_idx].is_leaf) {
        int feat = tree[node_idx].feature_idx;
        if (features[feat] <= tree[node_idx].threshold) {
            node_idx = tree[node_idx].left_child;
        } else {
            node_idx = tree[node_idx].right_child;
        }
    }
    return tree[node_idx].class_label;
}

float mli_tree_predict_proba(const mli_tree_node_t *tree, const float *features,
                             int target_class) {
    int predicted = mli_tree_predict(tree, features);
    return predicted == target_class ? 1.0f : 0.0f;
}

int mli_tree_depth(const mli_tree_node_t *tree, int node_idx) {
    if (tree[node_idx].is_leaf) return 0;
    int left_depth = mli_tree_depth(tree, tree[node_idx].left_child);
    int right_depth = mli_tree_depth(tree, tree[node_idx].right_child);
    return 1 + (left_depth > right_depth ? left_depth : right_depth);
}

int main(void) {
    mli_tree_node_t tree[5];
    tree[0].feature_idx = 0; tree[0].threshold = 2.5f;
    tree[0].left_child = 1; tree[0].right_child = 2; tree[0].is_leaf = 0;
    tree[1].is_leaf = 1; tree[1].class_label = 0;
    tree[2].feature_idx = 1; tree[2].threshold = 5.0f;
    tree[2].left_child = 3; tree[2].right_child = 4; tree[2].is_leaf = 0;
    tree[3].is_leaf = 1; tree[3].class_label = 1;
    tree[4].is_leaf = 1; tree[4].class_label = 2;
    float features[2] = {3.0f, 4.0f};
    int pred = mli_tree_predict(tree, features);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C795: Decision tree inference should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C795: should produce non-empty output");
    assert!(
        code.contains("fn mli_tree_predict"),
        "C795: Should contain mli_tree_predict function"
    );
}

// ============================================================================
// C796-C800: Ensemble, Clustering, and Activation Functions
// ============================================================================

#[test]
fn c796_random_forest_ensemble() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int feature_idx;
    float threshold;
    int left_child;
    int right_child;
    int class_label;
    int is_leaf;
} mli_rf_node_t;

int mli_rf_single_tree(const mli_rf_node_t *tree, const float *features) {
    int idx = 0;
    while (!tree[idx].is_leaf) {
        if (features[tree[idx].feature_idx] <= tree[idx].threshold) {
            idx = tree[idx].left_child;
        } else {
            idx = tree[idx].right_child;
        }
    }
    return tree[idx].class_label;
}

int mli_random_forest_predict(const mli_rf_node_t *forest,
                              const int *tree_offsets,
                              int num_trees, int num_classes,
                              const float *features) {
    int votes[16] = {0};
    int t;

    for (t = 0; t < num_trees; t++) {
        const mli_rf_node_t *tree = &forest[tree_offsets[t]];
        int pred = mli_rf_single_tree(tree, features);
        if (pred >= 0 && pred < num_classes && pred < 16) {
            votes[pred]++;
        }
    }

    int best = 0;
    int best_votes = votes[0];
    int c;
    for (c = 1; c < num_classes; c++) {
        if (votes[c] > best_votes) {
            best_votes = votes[c];
            best = c;
        }
    }
    return best;
}

int main(void) {
    mli_rf_node_t forest[6];
    forest[0].feature_idx = 0; forest[0].threshold = 2.0f;
    forest[0].left_child = 1; forest[0].right_child = 2; forest[0].is_leaf = 0;
    forest[1].is_leaf = 1; forest[1].class_label = 0;
    forest[2].is_leaf = 1; forest[2].class_label = 1;
    forest[3].feature_idx = 1; forest[3].threshold = 3.0f;
    forest[3].left_child = 4; forest[3].right_child = 5; forest[3].is_leaf = 0;
    forest[4].is_leaf = 1; forest[4].class_label = 0;
    forest[5].is_leaf = 1; forest[5].class_label = 1;
    int offsets[2] = {0, 3};
    float features[2] = {1.0f, 4.0f};
    int pred = mli_random_forest_predict(forest, offsets, 2, 2, features);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C796: Random forest ensemble should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C796: should produce non-empty output");
    assert!(
        code.contains("fn mli_random_forest_predict"),
        "C796: Should contain mli_random_forest_predict function"
    );
}

#[test]
fn c797_kmeans_single_iteration() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_km_dist_sq(const float *a, const float *b, int dims) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < dims; i++) {
        float d = a[i] - b[i];
        sum += d * d;
    }
    return sum;
}

int mli_km_assign(const float *point, const float *centroids,
                  int k, int dims) {
    int best = 0;
    float best_dist = mli_km_dist_sq(point, centroids, dims);
    int c;
    for (c = 1; c < k; c++) {
        float d = mli_km_dist_sq(point, &centroids[c * dims], dims);
        if (d < best_dist) {
            best_dist = d;
            best = c;
        }
    }
    return best;
}

void mli_kmeans_iteration(const float *data, float *centroids,
                          int *assignments, int n_samples,
                          int k, int dims) {
    int i, c, d;

    for (i = 0; i < n_samples; i++) {
        assignments[i] = mli_km_assign(&data[i * dims], centroids, k, dims);
    }

    float new_centroids[64] = {0};
    int counts[16] = {0};

    for (i = 0; i < n_samples; i++) {
        int cluster = assignments[i];
        counts[cluster]++;
        for (d = 0; d < dims; d++) {
            new_centroids[cluster * dims + d] += data[i * dims + d];
        }
    }

    for (c = 0; c < k; c++) {
        if (counts[c] > 0) {
            for (d = 0; d < dims; d++) {
                centroids[c * dims + d] = new_centroids[c * dims + d] / (float)counts[c];
            }
        }
    }
}

int main(void) {
    float data[8] = {1.0f, 2.0f, 3.0f, 4.0f, 8.0f, 9.0f, 7.0f, 8.0f};
    float centroids[4] = {1.0f, 2.0f, 8.0f, 9.0f};
    int assignments[4];
    mli_kmeans_iteration(data, centroids, assignments, 4, 2, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C797: K-means single iteration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C797: should produce non-empty output");
    assert!(
        code.contains("fn mli_kmeans_iteration"),
        "C797: Should contain mli_kmeans_iteration function"
    );
}

#[test]
fn c798_pca_dimensionality_reduction() {
    let c_code = r#"
typedef unsigned long size_t;

void mli_pca_mean(const float *data, float *mean,
                  int n_samples, int n_features) {
    int i, f;
    for (f = 0; f < n_features; f++) {
        mean[f] = 0.0f;
    }
    for (i = 0; i < n_samples; i++) {
        for (f = 0; f < n_features; f++) {
            mean[f] += data[i * n_features + f];
        }
    }
    for (f = 0; f < n_features; f++) {
        mean[f] /= (float)n_samples;
    }
}

void mli_pca_covariance(const float *data, const float *mean,
                        float *cov, int n_samples, int n_features) {
    int i, j, k;
    for (i = 0; i < n_features; i++) {
        for (j = 0; j < n_features; j++) {
            cov[i * n_features + j] = 0.0f;
        }
    }
    for (k = 0; k < n_samples; k++) {
        for (i = 0; i < n_features; i++) {
            for (j = 0; j < n_features; j++) {
                cov[i * n_features + j] +=
                    (data[k * n_features + i] - mean[i]) *
                    (data[k * n_features + j] - mean[j]);
            }
        }
    }
    for (i = 0; i < n_features; i++) {
        for (j = 0; j < n_features; j++) {
            cov[i * n_features + j] /= (float)(n_samples - 1);
        }
    }
}

void mli_pca_project(const float *data, const float *mean,
                     const float *components, float *projected,
                     int n_samples, int n_features, int n_components) {
    int i, j, k;
    for (i = 0; i < n_samples; i++) {
        for (j = 0; j < n_components; j++) {
            float val = 0.0f;
            for (k = 0; k < n_features; k++) {
                val += (data[i * n_features + k] - mean[k]) * components[j * n_features + k];
            }
            projected[i * n_components + j] = val;
        }
    }
}

int main(void) {
    float data[12] = {1,2,3, 4,5,6, 7,8,9, 10,11,12};
    float mean[3];
    float cov[9];
    mli_pca_mean(data, mean, 4, 3);
    mli_pca_covariance(data, mean, cov, 4, 3);
    float components[6] = {0.577f, 0.577f, 0.577f, -0.707f, 0.707f, 0.0f};
    float projected[8];
    mli_pca_project(data, mean, components, projected, 4, 3, 2);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C798: PCA dimensionality reduction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C798: should produce non-empty output");
    assert!(
        code.contains("fn mli_pca_project"),
        "C798: Should contain mli_pca_project function"
    );
}

#[test]
fn c799_sigmoid_activation() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_sig_exp(float x) {
    if (x > 88.0f) return 1.0e38f;
    if (x < -88.0f) return 0.0f;
    float result = 1.0f;
    float term = 1.0f;
    int i;
    for (i = 1; i <= 12; i++) {
        term *= x / (float)i;
        result += term;
    }
    return result;
}

float mli_sigmoid(float x) {
    if (x >= 20.0f) return 1.0f;
    if (x <= -20.0f) return 0.0f;
    return 1.0f / (1.0f + mli_sig_exp(-x));
}

void mli_sigmoid_vec(const float *input, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = mli_sigmoid(input[i]);
    }
}

void mli_sigmoid_derivative(const float *sigmoid_output, float *grad, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float s = sigmoid_output[i];
        grad[i] = s * (1.0f - s);
    }
}

float mli_sigmoid_focal_loss(const float *pred, const float *target,
                             int len, float gamma) {
    float loss = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        float p = mli_sigmoid(pred[i]);
        float pt = target[i] * p + (1.0f - target[i]) * (1.0f - p);
        float focal_weight = 1.0f;
        int g;
        for (g = 0; g < (int)gamma; g++) {
            focal_weight *= (1.0f - pt);
        }
        float log_pt = 0.0f;
        if (pt > 1.0e-7f) {
            float y = (pt - 1.0f) / (pt + 1.0f);
            float y2 = y * y;
            float t = y;
            int n;
            for (n = 0; n < 10; n++) {
                log_pt += t / (float)(2 * n + 1);
                t *= y2;
            }
            log_pt *= 2.0f;
        }
        loss -= focal_weight * log_pt;
    }
    return loss / (float)len;
}

int main(void) {
    float input[4] = {-2.0f, -1.0f, 0.0f, 1.0f};
    float output[4];
    mli_sigmoid_vec(input, output, 4);
    float grad[4];
    mli_sigmoid_derivative(output, grad, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C799: Sigmoid activation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C799: should produce non-empty output");
    assert!(
        code.contains("fn mli_sigmoid"),
        "C799: Should contain mli_sigmoid function"
    );
}

#[test]
fn c800_tanh_activation() {
    let c_code = r#"
typedef unsigned long size_t;

float mli_tanh_exp(float x) {
    if (x > 88.0f) return 1.0e38f;
    if (x < -88.0f) return 0.0f;
    float result = 1.0f;
    float term = 1.0f;
    int i;
    for (i = 1; i <= 12; i++) {
        term *= x / (float)i;
        result += term;
    }
    return result;
}

float mli_tanh(float x) {
    if (x > 10.0f) return 1.0f;
    if (x < -10.0f) return -1.0f;
    float ep = mli_tanh_exp(x);
    float en = mli_tanh_exp(-x);
    return (ep - en) / (ep + en);
}

void mli_tanh_vec(const float *input, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = mli_tanh(input[i]);
    }
}

void mli_tanh_derivative(const float *tanh_output, float *grad, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float t = tanh_output[i];
        grad[i] = 1.0f - t * t;
    }
}

void mli_hardtanh(float *data, int len, float min_val, float max_val) {
    int i;
    for (i = 0; i < len; i++) {
        if (data[i] < min_val) data[i] = min_val;
        if (data[i] > max_val) data[i] = max_val;
    }
}

float mli_gelu_approx(float x) {
    float tanh_arg = 0.7978845608f * (x + 0.044715f * x * x * x);
    float t = mli_tanh(tanh_arg);
    return 0.5f * x * (1.0f + t);
}

void mli_gelu_vec(const float *input, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = mli_gelu_approx(input[i]);
    }
}

int main(void) {
    float input[4] = {-2.0f, -1.0f, 0.0f, 1.0f};
    float output[4];
    mli_tanh_vec(input, output, 4);
    float grad[4];
    mli_tanh_derivative(output, grad, 4);
    float gelu_out[4];
    mli_gelu_vec(input, gelu_out, 4);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C800: Tanh activation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C800: should produce non-empty output");
    assert!(
        code.contains("fn mli_tanh"),
        "C800: Should contain mli_tanh function"
    );
}
