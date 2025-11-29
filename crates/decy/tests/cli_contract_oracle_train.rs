//! CLI Contract Tests for `decy oracle train`
//!
//! DECY-106: CITL training command with feedback loop
//!
//! These tests verify the contract for the oracle train command:
//! - Exit codes
//! - stdout/stderr behavior
//! - Error messages
//! - Edge cases

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: Create decy command with oracle feature
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp directory with C training file
fn create_training_corpus(dir: &TempDir) -> std::path::PathBuf {
    let corpus = dir.path().join("corpus");
    std::fs::create_dir_all(&corpus).expect("Failed to create corpus dir");

    // Create a simple C file that will produce errors when transpiled
    let c_file = corpus.join("test.c");
    std::fs::write(
        &c_file,
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
    .expect("Failed to write C file");

    corpus
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_oracle_train_valid_corpus_exits_zero() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_oracle_train_missing_corpus_exits_nonzero() {
    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg("/nonexistent/path")
        .assert()
        .failure() // Exit code != 0
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}

#[test]
fn cli_oracle_train_empty_corpus_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let empty_corpus = temp.path().join("empty");
    std::fs::create_dir_all(&empty_corpus).unwrap();

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&empty_corpus)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No C files").or(predicate::str::contains("empty")));
}

#[test]
fn cli_oracle_train_missing_corpus_arg_exits_nonzero() {
    decy_cmd()
        .arg("oracle")
        .arg("train")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--corpus"));
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT/STDERR
// ============================================================================

#[test]
fn cli_oracle_train_shows_progress() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success()
        .stdout(predicate::str::contains("Training")); // Shows progress
}

#[test]
fn cli_oracle_train_shows_file_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success()
        .stdout(predicate::str::contains("file").or(predicate::str::contains("File")));
}

#[test]
fn cli_oracle_train_shows_error_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn cli_oracle_train_shows_pattern_count() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success()
        .stdout(predicate::str::contains("pattern").or(predicate::str::contains("Pattern")));
}

// ============================================================================
// CLI CONTRACT TESTS: TIER OPTIONS
// ============================================================================

#[test]
fn cli_oracle_train_accepts_tier_p0() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--tier")
        .arg("P0")
        .assert()
        .success();
}

#[test]
fn cli_oracle_train_accepts_tier_p1() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--tier")
        .arg("P1")
        .assert()
        .success();
}

#[test]
fn cli_oracle_train_accepts_tier_p2() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--tier")
        .arg("P2")
        .assert()
        .success();
}

#[test]
fn cli_oracle_train_invalid_tier_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
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
fn cli_oracle_train_dry_run_no_side_effects() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    // Check patterns before
    let before = decy_cmd()
        .arg("oracle")
        .arg("stats")
        .output()
        .expect("stats failed");

    // Run dry-run training
    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN").or(predicate::str::contains("dry")));

    // Check patterns after - should be unchanged
    let after = decy_cmd()
        .arg("oracle")
        .arg("stats")
        .output()
        .expect("stats failed");

    assert_eq!(before.stdout, after.stdout, "Dry run should not modify patterns");
}

// ============================================================================
// CLI CONTRACT TESTS: METRICS OUTPUT
// ============================================================================

#[test]
fn cli_oracle_train_shows_summary() {
    let temp = TempDir::new().unwrap();
    let corpus = create_training_corpus(&temp);

    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--corpus")
        .arg(&corpus)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Summary")
                .or(predicate::str::contains("Results"))
                .or(predicate::str::contains("Complete")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_oracle_train_help_shows_usage() {
    decy_cmd()
        .arg("oracle")
        .arg("train")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--corpus"))
        .stdout(predicate::str::contains("train"));
}
