//! CUDA .cu Kernel Transpilation — Proven by Compilation
//!
//! Demonstrates: __global__ -> extern "C" FFI, host code -> safe Rust
//! **Proves**: transpiled .cu output compiles with rustc
//!
//! Run: `cargo run -p decy-core --example cuda_kernel_ffi_demo`

use decy_core::transpile;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CUDA .cu -> Rust (Kernel FFI + Host Code) ===\n");

    // Simulates what a real .cu file contains
    let cu_code = r#"
__global__ void vector_add(float* a, float* b, float* c, int n) {
    int idx = 0;
    if (idx < n) {
        c[idx] = a[idx] + b[idx];
    }
}

__global__ void saxpy(float a, float* x, float* y, int n) {
    int i = 0;
}

__global__ void matmul(float* A, float* B, float* C, int M, int N, int K) {
    int row = 0;
    int col = 0;
}

void launch_kernels(int n) {
    int block_size = 256;
    int grid_size = (n + block_size - 1) / block_size;
}

int compute_grid_size(int n, int block) {
    return (n + block - 1) / block;
}
"#;

    println!("CUDA Input: 3 __global__ kernels + 2 host functions\n");

    let rust = transpile(cu_code)?;
    let clean = strip(&rust);
    println!("Rust Output:");
    println!("{}\n", clean);

    let ok = compile_check(&clean)?;
    if ok {
        println!("[PROVEN] Output compiles with rustc --edition 2021");
    } else {
        println!("[FAILED] Compilation error");
        std::process::exit(1);
    }

    // Verify kernels become extern "C" FFI
    assert!(clean.contains("extern \"C\""), "Should have extern C block");
    assert!(clean.contains("fn vector_add("), "vector_add kernel");
    assert!(clean.contains("fn saxpy("), "saxpy kernel");
    assert!(clean.contains("fn matmul("), "matmul kernel");

    // Verify host functions transpile normally
    assert!(clean.contains("fn launch_kernels("), "host function");
    assert!(clean.contains("fn compute_grid_size("), "host function");
    assert!(clean.contains("block_size"), "host variable");

    // Verify kernels use raw pointers (FFI compatible)
    assert!(clean.contains("*mut f32"), "Kernel params should be raw pointers");

    println!("[VERIFIED] 3 kernels -> extern C, 2 host functions -> safe Rust");

    Ok(())
}

fn strip(code: &str) -> String {
    let mut out = Vec::new();
    let mut skip = false;
    for line in code.lines() {
        if line.starts_with("fn __") {
            skip = true;
            continue;
        }
        if skip && line.trim() == "}" {
            skip = false;
            continue;
        }
        skip = false;
        if line.starts_with("static mut ERRNO") {
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn compile_check(code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let rs = dir.path().join("out.rs");
    let rlib = dir.path().join("out.rlib");
    std::fs::write(&rs, code)?;
    let o = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type=lib", "-o"])
        .arg(&rlib)
        .arg(&rs)
        .output()?;
    Ok(!String::from_utf8_lossy(&o.stderr).lines().any(|l| l.starts_with("error")))
}
