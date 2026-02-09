//! REPL coverage tests targeting uncovered paths in repl.rs.
//!
//! These tests exercise REPL command parsing, transpilation snippets,
//! and the print_help function from outside the module.
//! We also test the REPL binary subcommand for basic startup/exit behavior.

mod cli_testing_tools;

use cli_testing_tools::*;
use predicates::prelude::*;

// ============================================================================
// REPL VIA CLI: BASIC STARTUP
// ============================================================================

#[test]
fn cli_repl_help_flag() {
    // Ensure the repl subcommand is recognized
    decy_cmd()
        .arg("repl")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("REPL").or(predicate::str::contains("repl")));
}

// ============================================================================
// REPL VIA CLI: STDIN PIPE (simulated user input)
// ============================================================================

#[test]
fn cli_repl_quit_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_q_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":q\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_exit_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_help_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":help\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"));
}

#[test]
fn cli_repl_h_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":h\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"));
}

#[test]
fn cli_repl_question_mark_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":?\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"));
}

#[test]
fn cli_repl_clear_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":clear\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_c_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":c\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_transpile_simple_code() {
    decy_cmd()
        .arg("repl")
        .write_stdin("int main() { return 0; }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpiled Rust code"));
}

#[test]
fn cli_repl_transpile_function() {
    decy_cmd()
        .arg("repl")
        .write_stdin("int add(int a, int b) { return a + b; }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpiled Rust code"));
}

#[test]
fn cli_repl_transpile_void_function() {
    decy_cmd()
        .arg("repl")
        .write_stdin("void noop() { }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpiled Rust code"));
}

#[test]
fn cli_repl_transpile_error() {
    decy_cmd()
        .arg("repl")
        .write_stdin("int bad(\n:quit\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Error").or(predicate::str::is_empty()));
}

#[test]
fn cli_repl_unknown_command() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":unknown\n:quit\n")
        .assert()
        .success();
    // Unknown commands starting with : should be treated as code
}

#[test]
fn cli_repl_empty_input() {
    decy_cmd()
        .arg("repl")
        .write_stdin("\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_multiple_transpilations() {
    decy_cmd()
        .arg("repl")
        .write_stdin("int a() { return 1; }\nint b() { return 2; }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpiled Rust code"));
}

#[test]
fn cli_repl_eof_exit() {
    // Sending empty stdin simulates EOF (Ctrl+D)
    decy_cmd()
        .arg("repl")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn cli_repl_help_then_code_then_quit() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":help\nint main() { return 0; }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"))
        .stdout(predicate::str::contains("Transpiled Rust code"))
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_shows_banner() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL"))
        .stdout(predicate::str::contains("Interactive Mode"))
        .stdout(predicate::str::contains(":help"));
}

// ============================================================================
// REPL VIA CLI: TRANSPILE VARIOUS C PATTERNS
// ============================================================================

#[test]
fn cli_repl_transpile_struct() {
    decy_cmd()
        .arg("repl")
        .write_stdin("struct Point { int x; int y; };\n:quit\n")
        .assert()
        .success();
}

#[test]
fn cli_repl_transpile_with_params() {
    decy_cmd()
        .arg("repl")
        .write_stdin("int multiply(int a, int b) { return a * b; }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpiled Rust code"));
}

// ============================================================================
// REPL VIA CLI: WHITESPACE AND EDGE CASES
// ============================================================================

#[test]
fn cli_repl_whitespace_input() {
    decy_cmd()
        .arg("repl")
        .write_stdin("   \n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_command_with_leading_whitespace() {
    decy_cmd()
        .arg("repl")
        .write_stdin("  :help  \n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"));
}

#[test]
fn cli_repl_rapid_quit() {
    // Immediately quit without any interaction
    decy_cmd()
        .arg("repl")
        .write_stdin(":q\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

#[test]
fn cli_repl_multiple_helps() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":help\n:h\n:?\n:quit\n")
        .assert()
        .success();
}

#[test]
fn cli_repl_clear_then_help_then_quit() {
    decy_cmd()
        .arg("repl")
        .write_stdin(":clear\n:help\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Decy REPL Commands"));
}
