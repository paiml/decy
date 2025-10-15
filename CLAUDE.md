# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Decy is a C-to-Rust transpiler that generates idiomatic, safe Rust code with minimal `unsafe` blocks (<5 per 1000 LOC). The project uses **EXTREME TDD**, **Toyota Way principles**, and **PMAT (Project Management and Automation Toolkit)** qualification for roadmap-driven development.

**Critical Goal**: Minimize unsafe code through advanced ownership and lifetime inference algorithms.

## Development Commands

### Essential Commands

```bash
# Complete installation (Rust + LLVM 14 + tools)
make install

# Build workspace
make build

# Run all tests
make test

# Run quality gates (pre-commit checks)
make quality-gates

# Run specific crate tests
cargo test -p decy-parser

# Run single test
cargo test -p decy-parser test_parse_simple_main_function

# Coverage report
make coverage

# See all available commands
make help
```

### PMAT Roadmap Commands

```bash
# Sync roadmap.yaml with GitHub Issues
make sync-roadmap

# Check roadmap state
make check-roadmap

# Show current sprint status
make roadmap-status
```

### Testing Levels

```bash
# Unit tests only (fast)
make test-fast

# Unit + integration + doc tests
make test-all

# Property tests
cargo test --features proptest-tests

# Mutation tests (slow, run in CI)
make mutation
```

## Architecture: Multi-Stage Transpilation Pipeline

Decy uses a **6-stage pipeline** where each stage has a dedicated crate:

```
C Source → Parser → HIR → Analyzer → Ownership → Verify → Codegen → Rust Output
                                         ↓
                                    Lifetime
```

### Critical Crates (ordered by data flow)

1. **decy-parser** (`crates/decy-parser/`)
   - Uses `clang-sys` (LLVM/Clang bindings) to parse C into AST
   - **Note**: Only crate allowed to use `unsafe` code (for FFI)
   - Environment: Requires `LLVM_CONFIG_PATH` and `LIBCLANG_PATH` set

2. **decy-hir** (`crates/decy-hir/`)
   - High-level Intermediate Representation (Rust-oriented)
   - Bridges C AST → Rust concepts
   - Serializable for debugging/analysis

3. **decy-analyzer** (`crates/decy-analyzer/`)
   - Static analysis: control flow, data flow, type inference
   - Uses `petgraph` for dependency graphs
   - Foundation for ownership inference

4. **decy-ownership** (`crates/decy-ownership/`) ⚠️ **CRITICAL**
   - **Most important crate** for unsafe code reduction
   - Infers ownership patterns from C pointer usage
   - Classifies pointers: owning vs borrowing
   - Detects patterns: `malloc/free` → `Box`, arrays → `Vec`
   - **Target**: 90% coverage (higher than other crates)

5. **decy-verify** (`crates/decy-verify/`)
   - Safety property verification before codegen
   - Validates: memory safety, type safety, borrow checker rules
   - Uses `syn` to analyze generated Rust AST

6. **decy-codegen** (`crates/decy-codegen/`)
   - Generates idiomatic Rust from HIR + ownership/lifetime info
   - Uses `quote` and `proc-macro2` for code generation
   - **Goal**: <5 unsafe blocks per 1000 LOC

### Supporting Crates

- **decy-core**: Orchestrates the pipeline
- **decy-book**: Book-based verification (mdBook + compile + lint)
- **decy-agent**: Background daemon for incremental transpilation
- **decy-mcp**: MCP server for Claude Code integration
- **decy-repo**: GitHub repository transpilation (parallel processing with `rayon`)
- **decy**: CLI binary

## EXTREME TDD Workflow

**Every ticket MUST follow RED-GREEN-REFACTOR**:

### 1. RED Phase (Write Failing Tests)
```bash
# Write failing tests first
cargo test -p decy-parser  # Tests should FAIL

# Commit with --no-verify (quality gates will block)
git commit --no-verify -m "[RED] DECY-XXX: Add failing tests"

# Update roadmap.yaml: phase: RED
```

### 2. GREEN Phase (Minimal Implementation)
```bash
# Implement minimal solution to pass tests
cargo test -p decy-parser  # Tests should PASS

git commit -m "[GREEN] DECY-XXX: Implement feature"

# Update roadmap.yaml: phase: GREEN
```

### 3. REFACTOR Phase (Meet Quality Gates)
```bash
# Clean up, add docs, meet quality standards
make quality-gates  # Must PASS

git commit -m "[REFACTOR] DECY-XXX: Meet quality gates"

# Update roadmap.yaml: phase: REFACTOR
```

### 4. Final Commit (Squash and Close)
```bash
# Squash RED/GREEN/REFACTOR commits
git rebase -i HEAD~3

# Final commit message format:
git commit -m "DECY-XXX: Description

- Coverage: 85% ✅
- Mutation score: 92% ✅
- Quality grade: A ✅

Closes #XXX"

# Update roadmap.yaml: status: done, phase: DONE
```

## Quality Standards (ENFORCED)

### Zero Tolerance Policies

- **Coverage**: ≥80% minimum (85% target, 90% for `decy-ownership`)
- **Clippy warnings**: 0 (enforced with `-D warnings`)
- **SATD comments**: 0 (no `TODO`, `FIXME`, `HACK`, `XXX`, `TEMP`, `WIP`, `BROKEN`)
- **Unsafe blocks**: <5 per 1000 LOC (tracked per sprint)
- **Cyclomatic complexity**: ≤10 per function
- **Cognitive complexity**: ≤15 per function

### Pre-Commit Hook

The pre-commit hook (`./git/hooks/pre-commit`) runs automatically and checks:
1. Code formatting (`cargo fmt`)
2. Linting (`cargo clippy -- -D warnings`)
3. SATD comments (grep for forbidden patterns)
4. All tests passing
5. Coverage ≥80%
6. Documentation builds

**Important**: Use `git commit --no-verify` ONLY during RED phase when tests should fail.

### Testing Requirements (Per Module)

Every module needs 4 types of tests:

1. **Unit tests** (≥5 per module)
   - Location: `#[cfg(test)] mod tests` or separate `*_tests.rs`
   - Target: 85% coverage

2. **Property tests** (≥3 per module)
   - Location: `tests/*_property_tests.rs`
   - Uses `proptest` crate
   - Target: 100+ properties × 1000 cases = 100K+ total tests

3. **Doctests** (≥2 per public function)
   - Location: In `///` doc comments
   - Must compile and pass

4. **Examples** (≥1 per module)
   - Location: `examples/*_demo.rs`
   - Working, runnable examples demonstrating usage

### CLI Contract Testing (MANDATORY)

**CRITICAL**: Following ruchy's proven pattern, ALL CLI commands MUST have comprehensive contract tests.

**Why CLI Testing Matters**:
- **User-facing**: CLI is the primary interface - bugs directly impact users
- **Regression prevention**: Changes to internal code must not break CLI contracts
- **Documentation validation**: Tests prove CLI documentation is accurate
- **Exit code contracts**: Non-zero exit codes must be consistent and documented

**Testing Pattern** (from ../ruchy/tests/cli_contract_*.rs):

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

#[test]
fn cli_transpile_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.c", "int main() { return 0; }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_transpile_invalid_syntax_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }"); // Malformed

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure() // Exit code != 0
        .stderr(predicate::str::is_empty().not()); // stderr has error
}

#[test]
fn cli_transpile_missing_file_exits_nonzero() {
    decy_cmd()
        .arg("transpile")
        .arg("nonexistent_file.c")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
        );
}
```

**CLI Test Requirements** (MANDATORY for each command):

1. **Exit Codes**:
   - Success case → exit code 0
   - Invalid input → exit code 1
   - File not found → exit code 1
   - Internal error → exit code 2

2. **stdout/stderr**:
   - Success messages go to stdout
   - Error messages go to stderr
   - Machine-readable output on stdout (JSON, etc.)
   - Human-readable errors on stderr

3. **Error Messages**:
   - Include filename when applicable
   - Include line:column for syntax errors
   - Actionable guidance ("Try X" or "See Y")
   - No cryptic stack traces

4. **Edge Cases**:
   - Empty file handling
   - Whitespace-only files
   - Comment-only files
   - Multiple input files
   - Stdin input (when applicable)

**Test Organization**:

```
tests/
├── cli_contract_transpile.rs   # decy transpile <file>
├── cli_contract_check.rs        # decy check <file>
├── cli_contract_parse.rs        # decy parse <file>
├── cli_contract_analyze.rs      # decy analyze <file>
└── cli_testing_tools.rs         # Shared helpers
```

**Example Test Suite** (decy transpile):

```rust
// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_transpile_valid_c_file_exits_zero();

#[test]
fn cli_transpile_invalid_syntax_exits_nonzero();

#[test]
fn cli_transpile_missing_file_exits_nonzero();

// ============================================================================
// CLI CONTRACT TESTS: STDOUT/STDERR
// ============================================================================

#[test]
fn cli_transpile_valid_file_writes_rust_to_stdout();

#[test]
fn cli_transpile_error_writes_stderr();

#[test]
fn cli_transpile_warning_writes_stderr_but_succeeds();

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_transpile_error_includes_filename();

#[test]
fn cli_transpile_error_includes_line_number();

#[test]
fn cli_transpile_error_suggests_fix();

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_transpile_empty_file_errors();

#[test]
fn cli_transpile_whitespace_only_errors();

#[test]
fn cli_transpile_comment_only_valid();

#[test]
fn cli_transpile_multiple_files_transpiles_all();

#[test]
fn cli_transpile_stdin_works();
```

**Never Use**: `std::process::Command` directly - always use `assert_cmd` for CLI testing.

**Reference**: See `../ruchy/tests/cli_contract_*.rs` (2,116 lines of CLI tests) for complete examples.

## C Language Validation (NORTH STAR)

### Validation Reference: C99 + K&R C

**CRITICAL**: DECY uses **C99/K&R C as the validation north star**, following the same approach as the bashrs sister project (which uses GNU Bash Manual).

See `docs/C-VALIDATION-ROADMAP.yaml` for:
- 150 C language constructs mapped to Rust
- ISO C99 and K&R C section references for each construct
- STOP THE LINE (Andon Cord) protocol for validation bugs
- Unsafe minimization tracking per construct
- EXTREME TDD validation workflow

### STOP THE LINE Protocol (Andon Cord)

When validation against C99/K&R reveals a bug:

1. **STOP** all feature development immediately
2. **Create P0 ticket** in `roadmap.yaml` with:
   - C99/K&R reference section
   - Failing C code
   - Expected vs actual Rust output
   - Safety impact
3. **Apply EXTREME TDD fix** (RED-GREEN-REFACTOR)
4. **Verify** unsafe count didn't increase
5. **Resume** validation only after fix verified

**Example P0 ticket template** (from `C-VALIDATION-ROADMAP.yaml`):
```yaml
P0-<CONSTRUCT>-<NUMBER>:
  title: "[P0] Fix <construct> transpilation bug"
  type: bug
  priority: critical
  discovered_during_validation: true
  validation_reference: "ISO C99 §6.x.x or K&R Chapter X"
  c_input: |
    <failing C code>
  expected_rust: |
    <correct Rust output>
  actual_rust: |
    <buggy Rust output or error>
  safety_impact: "<memory safety, undefined behavior, etc.>"
```

## PMAT: Roadmap-Driven Development

### Single Source of Truth: `roadmap.yaml`

All development is ticket-driven. **Never write code without a ticket in `roadmap.yaml`**.

**CRITICAL**: All tickets MUST be in YAML format in `roadmap.yaml`. Markdown tickets in `docs/` are NOT allowed and will be rejected by `pmat` validation.

```yaml
DECY-XXX:
  title: "Short title"
  status: not_started | in_progress | done
  phase: RED | GREEN | REFACTOR | DONE
  github_issue: null | <issue_number>
  story_points: <number>
  priority: critical | high | medium | low
  type: feature | bug | refactor | quality | docs
  sprint: <number>
  description: |
    Multi-line description of the ticket.
    What needs to be done and why.
  acceptance_criteria:
    - Criterion 1
    - Criterion 2
  test_strategy: |
    How this will be tested (TDD approach)
  files_modified:
    - path/to/file1.rs
    - path/to/file2.rs
```

### State Changes MUST Be Committed

Every status or phase change in `roadmap.yaml` must be committed to git:

```bash
# Starting a ticket
vim roadmap.yaml  # Set status: in_progress, phase: RED
git commit -m "Start DECY-XXX: RED phase"

# Transitioning phases
vim roadmap.yaml  # Set phase: GREEN
git commit -m "DECY-XXX: Transition to GREEN phase"
```

### GitHub Issue Integration

```bash
# Create GitHub issues from roadmap
make sync-roadmap

# This creates issues with:
# - Labels: ticket, sprint-X, priority
# - Links back to roadmap.yaml
# - RED-GREEN-REFACTOR checklist
```

## Key Technical Concepts

### Unsafe Code Minimization (4-Phase Strategy)

Decy reduces unsafe code through progressive refinement:

**Phase 1: Pattern-Based** (100% → 50%)
- Detect `malloc/free` → Generate `Box::new()`
- Detect array allocation → Generate `Vec::with_capacity()`

**Phase 2: Ownership Inference** (50% → 20%)
- Classify pointers as owning vs borrowing
- Generate `&T` and `&mut T` from pointer usage patterns
- Implemented in `decy-ownership` crate

**Phase 3: Lifetime Inference** (20% → 10%)
- Analyze C variable scopes
- Generate `<'a, 'b>` lifetime annotations
- Validate lifetime constraints

**Phase 4: Safe Wrappers** (10% → <5%)
- Generate safe abstractions around remaining unsafe
- Add `SAFETY` comments for audit trail

See `docs/specifications/decy-unsafe-minimization-strategy.md` for full details.

### Ownership Inference Algorithm (CRITICAL)

The `decy-ownership` crate is the **most critical component**:

1. Build pointer dataflow graph (`petgraph`)
2. Track pointer assignments and usage
3. Classify each pointer:
   - **Owning**: `Box<T>`, `Vec<T>`, moved values
   - **Immutable borrow**: `&T`
   - **Mutable borrow**: `&mut T`
4. Detect patterns:
   - Single allocation + single free → `Box`
   - Array allocation → `Vec`
   - Read-only access → `&T`
   - Mutating access → `&mut T`

Property tests verify:
- Unique owner per allocation
- Borrows don't outlive owner
- Exclusive mutable borrows

### HIR (High-Level Intermediate Representation)

The HIR bridges C and Rust concepts:

```rust
// C types → Rust types
HirType::Int → i32
HirType::Float → f64
HirType::Pointer(Box<HirType>) → &T, &mut T, Box<T>, or Vec<T>
```

HIR is serializable for debugging: `cargo test -- --nocapture` shows HIR.

## File Locations

### Configuration Files

- `decy-quality.toml` - Quality standards and thresholds
- `roadmap.yaml` - PMAT roadmap with ticket states
- `Cargo.toml` - Workspace configuration (11 crates)
- `.github/workflows/quality.yml` - CI/CD pipeline

### Documentation

- `docs/specifications/decy-spec-v1.md` - Complete technical specification (1,127 lines)
- `docs/specifications/decy-unsafe-minimization-strategy.md` - Unsafe reduction strategy
- `docs/C-VALIDATION-ROADMAP.yaml` - **VALIDATION NORTH STAR** (C99/K&R reference-driven validation)
- `GETTING_STARTED.md` - Developer onboarding guide
- `INSTALL.md` - Installation troubleshooting

### Scripts

- `scripts/quality-gates.sh` - Pre-commit quality enforcement
- `scripts/verify-setup.sh` - Installation verification
- `scripts/sync-roadmap.sh` - Sync roadmap.yaml ↔ GitHub Issues

## Current Development Status

**Sprint**: 1 (Foundation & C Parser)
**Current Ticket**: DECY-001 (clang-sys integration)
**Phase**: RED (failing tests committed)

### Sprint 1 Tickets

- **DECY-001**: Setup clang-sys integration (in_progress, RED phase)
- **DECY-002**: Define HIR structure (not_started)
- **DECY-003**: Implement basic code generator (not_started)

View complete roadmap: `cat roadmap.yaml` or `make roadmap-status`

## Toyota Way Principles

Development follows Toyota Production System principles:

- **Jidoka (自働化)**: Build quality in - never merge incomplete features
- **Genchi Genbutsu (現地現物)**: Direct observation - test with real C code
- **Kaizen (改善)**: Continuous improvement - fix bugs before features
- **Hansei (反省)**: Reflection after each sprint on quality metrics

These are not just philosophy - they're enforced through:
- Quality gates (pre-commit hooks)
- Sprint retrospectives (tracked in `roadmap.yaml` quality_metrics)
- Zero defect tolerance in production code

## Special Notes

### When Working with `clang-sys`

The `decy-parser` crate requires LLVM/Clang:

```bash
# Ensure environment variables are set
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib

# Or use Makefile (sets automatically)
make build
```

### Bypassing Quality Gates

**Only bypass during RED phase**:

```bash
git commit --no-verify -m "[RED] DECY-XXX: Add failing tests"
```

All other commits MUST pass quality gates.

### Property Test Configuration

Property tests use `proptest`:

```rust
proptest! {
    #[test]
    fn property_name(input in strategy()) {
        // Test invariants
    }
}
```

- Min 100 properties per crate
- 1000 cases per property
- Total: 100K+ test cases
- Max shrink iterations: 10000

### Mutation Testing

Run mutation tests in CI (too slow for local):

```bash
# CI only (configured in quality.yml)
cargo mutants --workspace

# Target: ≥90% kill rate by Sprint 5
```

## Documentation Standards

Every public item needs documentation:

```rust
//! Module-level documentation (required)

/// Function documentation (required for public functions)
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// Description
///
/// # Examples
/// ```
/// // Doctest (required, ≥2 per function)
/// let result = function(input)?;
/// assert_eq!(result, expected);
/// # Ok::<(), Error>(())
/// ```
pub fn function(param: Type) -> Result<Output> {
    // Implementation
}
```

Missing docs are treated as errors: `#![deny(missing_docs)]` is enabled in all crates (except `decy-parser` which allows unsafe).
