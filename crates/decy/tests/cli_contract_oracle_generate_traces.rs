//! CLI Contract Tests for `decy oracle generate-traces`
//!
//! DECY-109: Golden Trace generation pipeline
//!
//! Per unified spec Section 6.3, this command generates Golden Traces
//! from a C corpus for model training. These tests define the contract:
//! - Exit codes
//! - stdout/stderr behavior
//! - Output file format
//! - Progress reporting
//! - Error handling

// Gate entire test module on oracle feature - tests expect oracle functionality
#![cfg(feature = "oracle")]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp directory with C training corpus
fn create_c_corpus(dir: &TempDir) -> std::path::PathBuf {
    let corpus = dir.path().join("corpus");
    std::fs::create_dir_all(&corpus).expect("Failed to create corpus dir");

    // P0: Simple type patterns
    let simple_c = corpus.join("simple.c");
    std::fs::write(
        &simple_c,
        r#"
int add(int a, int b) {
    return a + b;
}

int main() {
    int x = 10;
    int y = 20;
    return add(x, y);
}
"#,
    )
    .expect("Failed to write simple.c");

    // P1: Pointer patterns
    let ptr_c = corpus.join("pointers.c");
    std::fs::write(
        &ptr_c,
        r#"
void swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x = 10;
    int y = 20;
    swap(&x, &y);
    return x;
}
"#,
    )
    .expect("Failed to write pointers.c");

    corpus
}

/// Helper: Create complex corpus with P2 patterns
fn create_complex_corpus(dir: &TempDir) -> std::path::PathBuf {
    let corpus = dir.path().join("complex_corpus");
    std::fs::create_dir_all(&corpus).expect("Failed to create corpus dir");

    // P2: Complex patterns with malloc/free
    let alloc_c = corpus.join("alloc.c");
    std::fs::write(
        &alloc_c,
        r#"
#include <stdlib.h>

int* create_array(int size) {
    int *arr = malloc(size * sizeof(int));
    if (arr == NULL) return NULL;
    for (int i = 0; i < size; i++) {
        arr[i] = i;
    }
    return arr;
}

void free_array(int *arr) {
    free(arr);
}

int main() {
    int *arr = create_array(10);
    if (arr != NULL) {
        free_array(arr);
    }
    return 0;
}
"#,
    )
    .expect("Failed to write alloc.c");

    corpus
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_oracle_generate_traces_valid_corpus_exits_zero() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_oracle_generate_traces_missing_corpus_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg("/nonexistent/path")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure() // Exit code != 0
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}

#[test]
fn cli_oracle_generate_traces_empty_corpus_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let empty_corpus = temp.path().join("empty");
    std::fs::create_dir_all(&empty_corpus).unwrap();
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&empty_corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No C files").or(predicate::str::contains("empty")));
}

#[test]
fn cli_oracle_generate_traces_missing_corpus_arg_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--corpus"));
}

#[test]
fn cli_oracle_generate_traces_missing_output_arg_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--output"));
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FILE
// ============================================================================

#[test]
fn cli_oracle_generate_traces_creates_output_file() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Output file should be created");
}

#[test]
fn cli_oracle_generate_traces_output_is_valid_jsonl() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Read output and verify it's valid JSONL
    let content = std::fs::read_to_string(&output).expect("Failed to read output");
    for line in content.lines() {
        if !line.trim().is_empty() {
            assert!(
                serde_json::from_str::<serde_json::Value>(line).is_ok(),
                "Each line should be valid JSON: {}",
                line
            );
        }
    }
}

#[test]
fn cli_oracle_generate_traces_output_contains_c_snippet() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output).expect("Failed to read output");
    assert!(
        content.contains("c_snippet"),
        "Output should contain c_snippet field"
    );
}

#[test]
fn cli_oracle_generate_traces_output_contains_rust_snippet() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output).expect("Failed to read output");
    assert!(
        content.contains("rust_snippet"),
        "Output should contain rust_snippet field"
    );
}

#[test]
fn cli_oracle_generate_traces_output_contains_safety_explanation() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    let content = std::fs::read_to_string(&output).expect("Failed to read output");
    assert!(
        content.contains("safety_explanation"),
        "Output should contain safety_explanation field"
    );
}

// ============================================================================
// CLI CONTRACT TESTS: PROGRESS REPORTING
// ============================================================================

#[test]
fn cli_oracle_generate_traces_shows_progress() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Generating")
                .or(predicate::str::contains("Processing"))
                .or(predicate::str::contains("Trace")),
        );
}

#[test]
fn cli_oracle_generate_traces_shows_file_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("file").or(predicate::str::contains("File")));
}

#[test]
fn cli_oracle_generate_traces_shows_success_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("success")
                .or(predicate::str::contains("Success"))
                .or(predicate::str::contains("generated"))
                .or(predicate::str::contains("Generated")),
        );
}

#[test]
fn cli_oracle_generate_traces_shows_failure_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("fail")
                .or(predicate::str::contains("Fail"))
                .or(predicate::str::contains("skip"))
                .or(predicate::str::contains("Skip")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: TIER OPTIONS
// ============================================================================

#[test]
fn cli_oracle_generate_traces_accepts_tier_p0() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--tier")
        .arg("P0")
        .assert()
        .success();
}

#[test]
fn cli_oracle_generate_traces_accepts_tier_p1() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--tier")
        .arg("P1")
        .assert()
        .success();
}

#[test]
fn cli_oracle_generate_traces_accepts_tier_p2() {
    let temp = TempDir::new().unwrap();
    let corpus = create_complex_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--tier")
        .arg("P2")
        .assert()
        .success();
}

#[test]
fn cli_oracle_generate_traces_invalid_tier_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--tier")
        .arg("INVALID")
        .assert()
        .failure()
        .stderr(predicate::str::contains("P0").or(predicate::str::contains("tier")));
}

// ============================================================================
// CLI CONTRACT TESTS: DRY RUN
// ============================================================================

#[test]
fn cli_oracle_generate_traces_dry_run_no_output_file() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN").or(predicate::str::contains("dry")));

    // Output file should NOT be created in dry-run mode
    assert!(
        !output.exists(),
        "Output file should not be created in dry-run mode"
    );
}

#[test]
fn cli_oracle_generate_traces_dry_run_shows_what_would_happen() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("would").or(predicate::str::contains("Would")));
}

// ============================================================================
// CLI CONTRACT TESTS: VERIFICATION
// ============================================================================

#[test]
fn cli_oracle_generate_traces_only_verified_traces() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("verified").or(predicate::str::contains("Verified")));
}

// ============================================================================
// CLI CONTRACT TESTS: SUMMARY
// ============================================================================

#[test]
fn cli_oracle_generate_traces_shows_summary() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Summary")
                .or(predicate::str::contains("Results"))
                .or(predicate::str::contains("Complete")),
        );
}

#[test]
fn cli_oracle_generate_traces_shows_trace_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_c_corpus(&temp);
    let output = temp.path().join("traces.jsonl");

    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("trace").or(predicate::str::contains("Trace")));
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_oracle_generate_traces_help_shows_usage() {
    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--corpus"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("generate-traces"));
}

#[test]
fn cli_oracle_generate_traces_help_shows_tier_option() {
    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--tier"));
}

#[test]
fn cli_oracle_generate_traces_help_shows_dry_run_option() {
    decy_cmd()
        .arg("oracle")
        .arg("generate-traces")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}
