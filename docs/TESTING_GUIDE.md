# Decy Testing Guide

**Three-Tiered Testing Workflow (Certeza Methodology)**

This guide explains Decy's testing strategy based on the Certeza framework for achieving asymptotic test effectiveness.

**Reference**: `docs/specifications/improve-testing-quality-using-certeza-concepts.md`

---

## Table of Contents

1. [Overview](#overview)
2. [Tier 1: ON-SAVE (Sub-Second)](#tier-1-on-save)
3. [Tier 2: ON-COMMIT (Minutes)](#tier-2-on-commit)
4. [Tier 3: ON-MERGE (Hours)](#tier-3-on-merge)
5. [Writing Tests](#writing-tests)
6. [Quality Metrics](#quality-metrics)
7. [Troubleshooting](#troubleshooting)

---

## Overview

Decy uses **three tiers** of testing to balance rapid feedback with comprehensive verification:

| Tier | Timing | Frequency | Target Time | Purpose |
|------|--------|-----------|-------------|---------|
| **Tier 1** | ON-SAVE | Every save | <1 second | Fast feedback loop |
| **Tier 2** | ON-COMMIT | Pre-commit | 1-5 minutes | Quality gates |
| **Tier 3** | ON-MERGE | PR/nightly | 1-4 hours | Exhaustive verification |

**Key Principle**: Never run slow checks (mutation testing, formal verification) in fast feedback loops. This preserves developer flow state.

---

## Tier 1: ON-SAVE

**Goal**: Sub-second feedback to maintain flow state during TDD

### Running Tier 1

```bash
# Manual execution
./scripts/on-save.sh

# Editor integration (recommended)
# Add to your editor's on-save hook
```

### What It Checks

1. **Fast unit tests** - Library tests only (`cargo test --lib`)
2. **Quick clippy** - Basic linting (`cargo clippy --quiet`)
3. **Format check** - Code formatting (`cargo fmt --check`)
4. **SATD detection** - No TODO/FIXME/HACK comments

### Target Time

**<1 second** - If it takes longer, tests need optimization

### Why It Matters

Research shows context-switching costs **23 minutes of productivity** [Mark et al., 2008]. Sub-second feedback prevents mental interruption during TDD RED-GREEN-REFACTOR cycles.

### Editor Integration

#### VS Code (.vscode/settings.json)

```json
{
  "runOnSave.commands": [
    {
      "match": "\\.rs$",
      "command": "./scripts/on-save.sh",
      "runIn": "terminal"
    }
  ]
}
```

#### Vim/Neovim

```vim
autocmd BufWritePost *.rs !./scripts/on-save.sh
```

---

## Tier 2: ON-COMMIT

**Goal**: Comprehensive verification before code integration

### Running Tier 2

```bash
# Pre-commit hook (automatic)
git commit -m "Your message"

# Manual execution
make quality-gates

# Bypass (RED phase only!)
git commit --no-verify -m "[RED] DECY-XXX: Add failing tests"
```

### What It Checks

1. **Full test suite** - All tests (`cargo test --all-targets`)
2. **Property tests** - 1000 cases per property
3. **Coverage analysis** - Must be ≥80% (90% for ownership)
4. **Integration tests** - Cross-crate integration
5. **Documentation** - Builds without errors
6. **Clippy (full)** - All lints, zero warnings

### Quality Gates

**ENFORCED** - Commit blocked if any fail:

- ✅ Coverage ≥80% (85% target, 90% for `decy-ownership`)
- ✅ All tests pass
- ✅ Clippy warnings = 0
- ✅ SATD comments = 0
- ✅ Cyclomatic complexity ≤10

### Target Time

**1-5 minutes** - Acceptable delay for quality assurance

### When to Bypass

**ONLY during TDD RED phase** when tests should fail:

```bash
# RED: Write failing test
git commit --no-verify -m "[RED] DECY-XXX: Add failing test for feature"

# GREEN: Implement feature
git commit -m "[GREEN] DECY-XXX: Implement feature"

# REFACTOR: Clean up
git commit -m "[REFACTOR] DECY-XXX: Meet quality gates"
```

---

## Tier 3: ON-MERGE

**Goal**: Exhaustive verification for merge readiness

### Running Tier 3

```bash
# Automatic: Runs on every Pull Request
# See: .github/workflows/tier3-verification.yml

# Manual: Run mutation testing on a crate
./scripts/run-mutation-test.sh decy-ownership

# Manual: Run full workspace mutation testing (SLOW!)
./scripts/run-mutation-test.sh --all
```

### What It Checks

1. **Mutation Testing** - Verify test suite logic (85%+ score)
2. **Formal Verification** - Mathematical proofs (Kani)
3. **Miri** - Undefined behavior detection (zero violations)
4. **Fuzz Testing** - 1-hour campaigns (zero crashes)
5. **Benchmarks** - Performance regression detection

### Quality Gates

**PR BLOCKED** if any fail:

- ✅ Mutation score ≥85% overall (≥90% for ownership)
- ✅ All Kani proofs pass
- ✅ Zero Miri violations
- ✅ Zero fuzz crashes

### Target Time

**1-4 hours** - Run in CI, not locally

### Mutation Testing Deep Dive

**What is mutation testing?**

Traditional coverage measures *which lines executed*, not *whether tests detect bugs*. Mutation testing deliberately introduces bugs (mutants) and verifies tests fail.

**Example**:

```rust
// Original code
fn is_owning_pointer(&self, ptr: PointerId) -> bool {
    self.allocations.contains_key(&ptr)  // Original
}

// Mutant: Replace contains_key with is_empty
fn is_owning_pointer(&self, ptr: PointerId) -> bool {
    self.allocations.is_empty()  // MUTANT
}
```

If tests **still pass** with the mutant, your test suite has a **logic gap**.

**Running mutation tests**:

```bash
# Single crate (10-30 minutes)
./scripts/run-mutation-test.sh decy-ownership

# Results
Mutation Score: 87.5% (35/40 mutants caught)
  ✅ PASS (threshold: 90.0%)

Surviving Mutants: 5
  1. src/analyzer.rs:42 in classify_pointer
     Status: missed
```

**Fixing surviving mutants**:

1. Identify the mutant location
2. Add a test that would fail with that mutation
3. Re-run mutation testing
4. Repeat until target score achieved

---

## Writing Tests

### Unit Tests

**Location**: `#[cfg(test)] mod tests` or `*_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_owner_invariant() {
        let mut analyzer = OwnershipAnalyzer::new();
        let alloc = AllocationId(1);
        let owner = PointerId(1);

        analyzer.register_allocation(alloc, owner);

        let owners = analyzer.owners_of(alloc);
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0], owner);
    }
}
```

**Requirements**:
- ≥5 unit tests per module
- Target: 85% coverage

### Property-Based Tests

**Location**: `tests/*_property_tests.rs`

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_unique_owner_per_allocation(
        alloc_id in 0u32..100,
        owner in 0u32..100,
    ) {
        let mut analyzer = OwnershipAnalyzer::new();
        analyzer.register_allocation(AllocationId(alloc_id), PointerId(owner));

        let owners = analyzer.owners_of(AllocationId(alloc_id));
        prop_assert_eq!(owners.len(), 1);
    }
}
```

**Requirements**:
- ≥3 property tests per module
- 1000 cases per property
- Target: 100+ properties × 1000 cases = 100K+ total tests

### Doctests

**Location**: In `///` doc comments

```rust
/// Classifies a pointer as owning, borrowing, or unknown.
///
/// # Examples
///
/// ```
/// use decy_ownership::{OwnershipAnalyzer, AllocationId, PointerId};
///
/// let mut analyzer = OwnershipAnalyzer::new();
/// let alloc = AllocationId(1);
/// let ptr = PointerId(1);
///
/// analyzer.register_allocation(alloc, ptr);
/// assert_eq!(analyzer.classify_pointer(ptr), PointerClassification::Owning);
/// ```
pub fn classify_pointer(&self, ptr: PointerId) -> PointerClassification {
    // ...
}
```

**Requirements**:
- ≥2 doctests per public function
- Must compile and pass

### Integration Tests

**Location**: `tests/*_integration_tests.rs`

```rust
// tests/transpilation_integration_tests.rs
use decy_core::Transpiler;
use std::fs;

#[test]
fn test_end_to_end_transpilation() {
    let c_code = "int main() { return 0; }";
    let transpiler = Transpiler::new();

    let rust_code = transpiler.transpile_str(c_code).unwrap();

    // Verify generated Rust compiles
    assert!(rust_code.contains("fn main()"));
    assert!(rust_code.contains("0"));
}
```

---

## Quality Metrics

### Current Targets

| Metric | Minimum | Target | Measurement |
|--------|---------|--------|-------------|
| **Line Coverage** | 80% | 95% | `cargo llvm-cov` |
| **Branch Coverage** | - | 90% | `cargo llvm-cov --branch` |
| **Mutation Score** | 85% | 90% (ownership) | `cargo mutants` |
| **Property Tests** | 100 | 40+ critical | Count in tests/ |
| **Cyclomatic Complexity** | ≤10 | ≤10 | `cargo-complexity` |
| **Unsafe Blocks** | <5 per 1K LOC | Minimize | `grep -r "unsafe"` |
| **SATD Comments** | 0 | 0 | `grep -r "TODO\|FIXME"` |

### Checking Metrics

```bash
# Coverage
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Mutation score
./scripts/run-mutation-test.sh decy-ownership

# Complexity
cargo install cargo-complexity
cargo complexity --all

# Unsafe blocks
grep -r "unsafe" crates/ --include="*.rs" | wc -l
```

---

## Troubleshooting

### Tier 1 Takes Too Long (>1s)

**Problem**: ON-SAVE feedback is slow, breaks flow state

**Solutions**:
1. Use `cargo test --lib` instead of `cargo test --all`
2. Split slow tests into integration tests (Tier 2)
3. Use `#[ignore]` for slow unit tests
4. Check for expensive property tests in unit test module

### Pre-Commit Hook Blocks Commit

**Problem**: `make quality-gates` fails

**Solutions**:

1. **Coverage too low**:
   ```bash
   cargo llvm-cov --html
   # Identify uncovered code, add tests
   ```

2. **Clippy warnings**:
   ```bash
   cargo clippy --fix
   ```

3. **SATD comments**:
   ```bash
   grep -rn "TODO\|FIXME" crates/ --include="*.rs"
   # Remove or replace with GitHub issues
   ```

4. **Tests failing**:
   ```bash
   cargo test --all-targets
   # Fix failing tests
   ```

5. **During RED phase** (tests should fail):
   ```bash
   git commit --no-verify -m "[RED] DECY-XXX: Add failing test"
   ```

### Mutation Testing Never Completes

**Problem**: `cargo mutants` runs for hours

**Solutions**:
1. Test single crate: `./scripts/run-mutation-test.sh decy-ownership`
2. Use `--timeout` flag to limit mutant runtime
3. Run in CI, not locally
4. Exclude slow tests: Add to `mutants.toml` exclude patterns

### Kani Verification Fails

**Problem**: Formal verification harnesses don't pass

**Solutions**:
1. Check Kani is installed: `cargo kani --version`
2. Review proof harness bounds (`#[kani::unwind(N)]`)
3. Simplify invariants for bounded model checking
4. Check for non-deterministic code (e.g., HashMap iteration)

---

## Quick Reference

```bash
# Tier 1: Sub-second feedback
./scripts/on-save.sh

# Tier 2: Pre-commit (automatic)
git commit -m "Message"

# Tier 2: Manual
make quality-gates

# Tier 3: Mutation testing
./scripts/run-mutation-test.sh decy-ownership

# Coverage report
cargo llvm-cov --html

# Run specific test
cargo test -p decy-ownership test_unique_owner

# Run property tests only
cargo test --test ownership_invariants_property_tests
```

---

## Further Reading

- **Specification**: `docs/specifications/improve-testing-quality-using-certeza-concepts.md`
- **Quality Config**: `decy-quality.toml`
- **CI Workflow**: `.github/workflows/tier3-verification.yml`
- **Certeza Framework**: https://github.com/paiml/certeza

---

**Last Updated**: 2025-11-18
**Maintainer**: Decy Team
