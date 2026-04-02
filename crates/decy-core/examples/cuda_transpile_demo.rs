//! CUDA Transpilation Demonstration
//!
//! This example shows how Decy transpiles CUDA .cu source code:
//! - `__global__` kernels become `extern "C"` FFI declarations
//! - `__device__` functions are annotated as GPU-only
//! - Host code transpiles normally to safe Rust
//!
//! Run with: `cargo run -p decy-core --example cuda_transpile_demo`

use decy_core::transpile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Decy CUDA Transpilation Demo ===\n");

    demo_kernel_ffi()?;
    demo_host_code()?;
    demo_mixed()?;

    println!("\n=== All CUDA demos completed successfully ===");
    Ok(())
}

fn demo_kernel_ffi() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 1: __global__ kernel -> extern \"C\" FFI ---\n");

    let cuda_code = r#"
__global__ void vector_add(float* a, float* b, float* c, int n) {
    int i = 0;
    if (i < n) {
        c[i] = a[i] + b[i];
    }
}
"#;

    println!("CUDA Input:");
    println!("{}", cuda_code.trim());
    println!();

    let rust_code = transpile(cuda_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("extern \"C\""), "Kernel should generate extern C FFI");
    assert!(rust_code.contains("fn vector_add("), "Should contain kernel name");
    println!("  [PASS] CUDA kernel FFI generation verified\n");
    Ok(())
}

fn demo_host_code() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 2: Host code transpiles normally ---\n");

    let cuda_code = r#"
__global__ void dummy_kernel(int x) { }

void setup(int n) {
    int block_size = 256;
    int grid_size = (n + block_size - 1) / block_size;
}
"#;

    println!("CUDA Input:");
    println!("{}", cuda_code.trim());
    println!();

    let rust_code = transpile(cuda_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("fn setup("), "Host function should transpile normally");
    assert!(rust_code.contains("block_size"), "Host variables should be present");
    println!("  [PASS] Host code transpilation verified\n");
    Ok(())
}

fn demo_mixed() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 3: Mixed kernel + host ---\n");

    let cuda_code = r#"
__global__ void matmul_kernel(float* A, float* B, float* C, int N) {
    int row = 0;
    int col = 0;
}

int compute(int a, int b) {
    return a * b + a;
}

int main() {
    int result = compute(3, 4);
    return result;
}
"#;

    println!("CUDA Input:");
    println!("{}", cuda_code.trim());
    println!();

    let rust_code = transpile(cuda_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);

    assert!(
        rust_code.contains("extern \"C\"") && rust_code.contains("matmul_kernel"),
        "Kernel should be extern C"
    );
    assert!(rust_code.contains("fn compute("), "Host function should transpile");
    assert!(rust_code.contains("fn main()"), "Main should transpile");
    println!("  [PASS] Mixed CUDA transpilation verified\n");
    Ok(())
}
