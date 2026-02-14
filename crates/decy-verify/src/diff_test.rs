//! Differential testing against GCC semantics (S5).
//!
//! Compiles original C with gcc, compiles transpiled Rust with rustc,
//! runs both binaries, and compares stdout + exit codes to prove
//! behavioral equivalence.
//!
//! # Example
//!
//! ```no_run
//! use decy_verify::diff_test::{diff_test, DiffTestConfig};
//!
//! let c_code = "int main() { return 0; }";
//! let rust_code = "fn main() {}";
//! let config = DiffTestConfig::default();
//! let result = diff_test(c_code, rust_code, &config).unwrap();
//! assert!(result.stdout_matches);
//! assert!(result.exit_code_matches);
//! ```

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Output captured from running a compiled binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Process exit code (0 = success)
    pub exit_code: i32,
}

/// Result of a differential test comparing C and Rust execution.
#[derive(Debug, Clone)]
pub struct DiffTestResult {
    /// Output from the compiled C binary
    pub c_output: ExecutionOutput,
    /// Output from the compiled Rust binary
    pub rust_output: ExecutionOutput,
    /// Whether stdout is identical
    pub stdout_matches: bool,
    /// Whether exit codes are identical
    pub exit_code_matches: bool,
    /// List of specific divergences found
    pub divergences: Vec<String>,
}

impl DiffTestResult {
    /// Returns true when both stdout and exit code match.
    pub fn passed(&self) -> bool {
        self.stdout_matches && self.exit_code_matches
    }
}

/// Configuration for differential testing.
#[derive(Debug, Clone)]
pub struct DiffTestConfig {
    /// Timeout in seconds for each binary execution
    pub timeout_secs: u64,
    /// Path to the gcc compiler
    pub gcc_path: String,
    /// Path to the rustc compiler
    pub rustc_path: String,
    /// Whether to also compare stderr output
    pub compare_stderr: bool,
}

impl Default for DiffTestConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 5,
            gcc_path: "gcc".to_string(),
            rustc_path: "rustc".to_string(),
            compare_stderr: false,
        }
    }
}

/// Compile C source code with gcc and return the temp directory + binary path.
///
/// The caller owns the returned `TempDir`; dropping it cleans up all files.
pub fn compile_c(c_code: &str, config: &DiffTestConfig) -> Result<(TempDir, PathBuf)> {
    let tmp = TempDir::new().context("Failed to create temp directory for C compilation")?;
    let src = tmp.path().join("input.c");
    let bin = tmp.path().join("c_binary");

    std::fs::write(&src, c_code).context("Failed to write C source to temp file")?;

    let output = Command::new(&config.gcc_path)
        .arg("-o")
        .arg(&bin)
        .arg("-x")
        .arg("c")
        .arg("-std=c99")
        .arg("-lm")
        .arg(&src)
        .output()
        .with_context(|| format!("Failed to run gcc ({})", config.gcc_path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gcc compilation failed:\n{}", stderr);
    }

    Ok((tmp, bin))
}

/// Compile Rust source code with rustc and return the temp directory + binary path.
///
/// The caller owns the returned `TempDir`; dropping it cleans up all files.
pub fn compile_rust(rust_code: &str, config: &DiffTestConfig) -> Result<(TempDir, PathBuf)> {
    let tmp = TempDir::new().context("Failed to create temp directory for Rust compilation")?;
    let src = tmp.path().join("input.rs");
    let bin = tmp.path().join("rust_binary");

    std::fs::write(&src, rust_code).context("Failed to write Rust source to temp file")?;

    let output = Command::new(&config.rustc_path)
        .arg("--edition=2021")
        .arg("-o")
        .arg(&bin)
        .arg(&src)
        .output()
        .with_context(|| format!("Failed to run rustc ({})", config.rustc_path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rustc compilation failed:\n{}", stderr);
    }

    Ok((tmp, bin))
}

/// Run a compiled binary and capture its output.
pub fn run_binary(binary: &Path) -> Result<ExecutionOutput> {
    let output = Command::new(binary)
        .output()
        .with_context(|| format!("Failed to execute binary: {}", binary.display()))?;

    Ok(ExecutionOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Run a full differential test: compile C with gcc, compile Rust with rustc,
/// execute both, and compare outputs.
pub fn diff_test(
    c_code: &str,
    rust_code: &str,
    config: &DiffTestConfig,
) -> Result<DiffTestResult> {
    // Compile both
    let (_c_dir, c_bin) =
        compile_c(c_code, config).context("C compilation failed during diff test")?;
    let (_rs_dir, rs_bin) =
        compile_rust(rust_code, config).context("Rust compilation failed during diff test")?;

    // Run both
    let c_output = run_binary(&c_bin).context("Failed to run C binary")?;
    let rust_output = run_binary(&rs_bin).context("Failed to run Rust binary")?;

    // Compare
    let stdout_matches = c_output.stdout == rust_output.stdout;
    let exit_code_matches = c_output.exit_code == rust_output.exit_code;

    let mut divergences = Vec::new();

    if !stdout_matches {
        divergences.push(format!(
            "stdout differs:\n  C:    {:?}\n  Rust: {:?}",
            c_output.stdout, rust_output.stdout
        ));
    }

    if !exit_code_matches {
        divergences.push(format!(
            "exit code differs: C={}, Rust={}",
            c_output.exit_code, rust_output.exit_code
        ));
    }

    if config.compare_stderr && c_output.stderr != rust_output.stderr {
        divergences.push(format!(
            "stderr differs:\n  C:    {:?}\n  Rust: {:?}",
            c_output.stderr, rust_output.stderr
        ));
    }

    Ok(DiffTestResult {
        c_output,
        rust_output,
        stdout_matches,
        exit_code_matches,
        divergences,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Compilation tests
    // ========================================================================

    #[test]
    fn test_compile_c_valid() {
        let config = DiffTestConfig::default();
        let result = compile_c("int main() { return 0; }", &config);
        assert!(result.is_ok(), "Valid C should compile: {:?}", result.err());
        let (_dir, bin) = result.unwrap();
        assert!(bin.exists(), "Binary should exist after compilation");
    }

    #[test]
    fn test_compile_c_invalid() {
        let config = DiffTestConfig::default();
        let result = compile_c("int main( { }", &config);
        assert!(result.is_err(), "Invalid C should fail compilation");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("gcc compilation failed"),
            "Error should mention gcc: {}",
            err_msg
        );
    }

    #[test]
    fn test_compile_c_bad_gcc_path() {
        let config = DiffTestConfig {
            gcc_path: "/nonexistent/gcc".to_string(),
            ..Default::default()
        };
        let result = compile_c("int main() { return 0; }", &config);
        assert!(result.is_err(), "Bad gcc path should error");
    }

    #[test]
    fn test_compile_rust_valid() {
        let config = DiffTestConfig::default();
        let result = compile_rust("fn main() {}", &config);
        assert!(
            result.is_ok(),
            "Valid Rust should compile: {:?}",
            result.err()
        );
        let (_dir, bin) = result.unwrap();
        assert!(bin.exists(), "Binary should exist after compilation");
    }

    #[test]
    fn test_compile_rust_invalid() {
        let config = DiffTestConfig::default();
        let result = compile_rust("fn main( {}", &config);
        assert!(result.is_err(), "Invalid Rust should fail compilation");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("rustc compilation failed"),
            "Error should mention rustc: {}",
            err_msg
        );
    }

    #[test]
    fn test_compile_rust_bad_rustc_path() {
        let config = DiffTestConfig {
            rustc_path: "/nonexistent/rustc".to_string(),
            ..Default::default()
        };
        let result = compile_rust("fn main() {}", &config);
        assert!(result.is_err(), "Bad rustc path should error");
    }

    // ========================================================================
    // S5 Prediction tests: behavioral equivalence
    // ========================================================================

    #[test]
    fn test_s5_p1_integer_arithmetic() {
        let c_code = r#"
#include <stdio.h>
int add(int a, int b) { return a + b; }
int main() {
    printf("%d\n", add(2, 3));
    return 0;
}
"#;
        let rust_code = r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn main() {
    println!("{}", add(2, 3));
}
"#;
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(
            result.stdout_matches,
            "Integer arithmetic stdout should match: {:?}",
            result.divergences
        );
        assert!(
            result.exit_code_matches,
            "Exit codes should match: {:?}",
            result.divergences
        );
        assert!(result.passed());
    }

    #[test]
    fn test_s5_p2_array_indexing() {
        let c_code = r#"
#include <stdio.h>
int main() {
    int arr[] = {10, 20, 30};
    printf("%d\n", arr[1]);
    return 0;
}
"#;
        let rust_code = r#"
fn main() {
    let arr = [10, 20, 30];
    println!("{}", arr[1]);
}
"#;
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(
            result.stdout_matches,
            "Array indexing stdout should match: {:?}",
            result.divergences
        );
        assert!(result.passed());
    }

    #[test]
    fn test_s5_p3_string_output() {
        let c_code = r#"
#include <stdio.h>
int main() {
    printf("hello world\n");
    return 0;
}
"#;
        let rust_code = r#"
fn main() {
    println!("hello world");
}
"#;
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(
            result.stdout_matches,
            "String output should match: {:?}",
            result.divergences
        );
        assert!(result.passed());
    }

    // ========================================================================
    // Edge cases
    // ========================================================================

    #[test]
    fn test_empty_stdout() {
        let c_code = "int main() { return 0; }";
        let rust_code = "fn main() {}";
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(result.stdout_matches, "Empty stdout should match");
        assert_eq!(result.c_output.stdout, "");
        assert_eq!(result.rust_output.stdout, "");
        assert!(result.passed());
    }

    #[test]
    fn test_multiline_output() {
        let c_code = r#"
#include <stdio.h>
int main() {
    printf("line1\n");
    printf("line2\n");
    printf("line3\n");
    return 0;
}
"#;
        let rust_code = r#"
fn main() {
    println!("line1");
    println!("line2");
    println!("line3");
}
"#;
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(
            result.stdout_matches,
            "Multiline output should match: {:?}",
            result.divergences
        );
        assert!(result.passed());
    }

    #[test]
    fn test_nonzero_exit_code() {
        let c_code = "int main() { return 42; }";
        let rust_code = "fn main() { std::process::exit(42); }";
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert_eq!(result.c_output.exit_code, 42);
        assert_eq!(result.rust_output.exit_code, 42);
        assert!(result.exit_code_matches);
        assert!(result.passed());
    }

    #[test]
    fn test_stdout_divergence_detected() {
        let c_code = r#"
#include <stdio.h>
int main() { printf("from C\n"); return 0; }
"#;
        let rust_code = r#"
fn main() { println!("from Rust"); }
"#;
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(!result.stdout_matches, "Different outputs should diverge");
        assert!(!result.divergences.is_empty());
        assert!(!result.passed());
    }

    #[test]
    fn test_exit_code_divergence_detected() {
        let c_code = "int main() { return 0; }";
        let rust_code = "fn main() { std::process::exit(1); }";
        let config = DiffTestConfig::default();
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(
            !result.exit_code_matches,
            "Different exit codes should diverge"
        );
        assert!(!result.passed());
    }

    // ========================================================================
    // Config tests
    // ========================================================================

    #[test]
    fn test_default_config() {
        let config = DiffTestConfig::default();
        assert_eq!(config.timeout_secs, 5);
        assert_eq!(config.gcc_path, "gcc");
        assert_eq!(config.rustc_path, "rustc");
        assert!(!config.compare_stderr);
    }

    #[test]
    fn test_compare_stderr_flag() {
        let c_code = "int main() { return 0; }";
        let rust_code = "fn main() {}";
        let config = DiffTestConfig {
            compare_stderr: true,
            ..Default::default()
        };
        let result = diff_test(c_code, rust_code, &config).expect("diff_test should succeed");
        assert!(result.passed());
    }

    #[test]
    fn test_diff_test_result_passed() {
        let result = DiffTestResult {
            c_output: ExecutionOutput {
                stdout: "ok\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
            rust_output: ExecutionOutput {
                stdout: "ok\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
            stdout_matches: true,
            exit_code_matches: true,
            divergences: vec![],
        };
        assert!(result.passed());
    }

    #[test]
    fn test_diff_test_result_failed() {
        let result = DiffTestResult {
            c_output: ExecutionOutput {
                stdout: "a\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
            rust_output: ExecutionOutput {
                stdout: "b\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
            stdout_matches: false,
            exit_code_matches: true,
            divergences: vec!["stdout differs".to_string()],
        };
        assert!(!result.passed());
    }
}
