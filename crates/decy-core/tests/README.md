# Decy Test Suite: SQLite-Style Comprehensive Testing

This directory contains the comprehensive test suite for Decy, following SQLite's legendary testing methodology adapted for C-to-Rust transpilation.

## Overview

**Goal**: Achieve aerospace-grade reliability with <5 unsafe blocks per 1000 LOC through comprehensive, multi-harness testing.

**Inspiration**: SQLite (614 tests per line of code, 100% branch coverage, 248.5M tests before release)

**Target**: 100:1 test-to-code ratio minimum (500,000+ test instances)

## Test Categories

### 1. Integration Tests (`integration/`)
**Purpose**: End-to-end transpilation validation

**Test Count**: 8 foundation tests (target: 50+)

**Coverage**: Complete C source → Rust output pipeline

**Examples**:
```bash
cargo test --test basic_transpilation_test
```

**Status**: ✅ Foundation tests complete (16% of target)

---

### 2. Property-Based Tests (`properties/`)
**Purpose**: Automated invariant validation and edge case discovery

**Test Count**: 7 properties (target: 20+)

**Tool**: `proptest` (Rust QuickCheck equivalent)

**Test Cases**: 100+ per property (configurable to 100K for release)

**Examples**:
```bash
# Development (100 cases per property)
cargo test properties/

# Release validation (100K cases per property)
PROPTEST_CASES=100000 cargo test properties/ --release
```

**Properties Validated**:
- Parser never panics on any input
- Valid C99 identifiers accepted
- Numeric literal handling
- Nested parentheses support (depth 1-20)
- All C99 basic types recognized
- All C99 binary operators handled
- String literals with special characters

**Status**: ✅ Infrastructure complete (35% of target properties)

---

### 3. Torture Tests (`torture/`)
**Purpose**: Extreme edge cases and compiler limit testing

**Test Count**: 13 torture tests (target: 50+)

**Inspiration**: GCC torture test suite

**Examples**:
```bash
cargo test torture/
```

**Edge Cases Tested**:
- Deeply nested parentheses (100 levels)
- Extremely long identifiers (1000 chars)
- Integer literal edge cases (INT_MAX, LLONG_MAX)
- Float literal edge cases (DBL_MAX, hex floats, INF, NAN)
- Deeply nested structs (50 levels)
- Complex pointer arithmetic chains
- Extremely long strings (10K chars)
- Many function parameters (200 params)
- Deeply nested function calls (50 levels)
- Complex expressions (all operators)
- Multidimensional arrays (15 dimensions)

**Requirement**: ⚠️ **NO PANICS ALLOWED** - All tests must pass or fail gracefully

**Status**: ✅ Foundation tests complete (26% of target)

---

### 4. Unsafe Audit Tests (`unsafe_audit/`)
**Purpose**: Track, minimize, and audit unsafe blocks in generated code

**Test Count**: 10 audit tests (target: 15+)

**Critical Goal**: <5 unsafe blocks per 1000 lines of code

**Examples**:
```bash
# Quick audit
cargo test unsafe_audit/

# Comprehensive audit (includes file scans)
cargo test unsafe_audit/ --ignored
```

**Audit Checks**:
- Count unsafe blocks in generated code
- Calculate unsafe per 1000 LOC ratio
- Verify <5 per 1000 LOC threshold
- Check for SAFETY comments on all unsafe blocks
- Audit decy-parser (FFI exception allowed)
- Track unsafe reduction over sprints

**Status**: ✅ Infrastructure complete (67% of target)

---

### 5. Regression Tests (`regression/`)
**Purpose**: Prevent historical bugs from returning

**Test Count**: 2 regression tests (DECY-AUDIT-001, DECY-AUDIT-002)

**Coverage**: 100% of reported GitHub issues

**Convention**: `github_issue_NNN.rs` for each resolved issue

**Examples**:
```bash
cargo test regression/
```

**Requirement**: Every bug fix MUST add a regression test

**Status**: ✅ 100% of current issues covered

---

### 6. Differential Tests (`differential/`)
**Purpose**: Compare Decy output against reference implementations

**Test Count**: 0 (target: 10+)

**References**: GCC, Clang, rustc

**Examples**:
```bash
cargo test differential/
```

**Validation**:
- Semantic equivalence with GCC/Clang
- Rustc accepts Decy-generated code
- Cross-validate with Clang AST

**Status**: 🎯 Not yet implemented

---

### 7. Documentation Tests (`crates/decy-codegen/tests/`)
**Purpose**: Document all 150 C99 features with executable tests

**Test Count**: 57 features documented (target: 150)

**Reference**: `C-VALIDATION-ROADMAP.yaml`

**Examples**:
```bash
cargo test --package decy-codegen --test '*_documentation_test'
```

**Features Documented** (38% complete):
- ✅ long long type (16 tests)
- ✅ hexadecimal float literals (15 tests)
- ✅ inline functions (16 tests)
- ✅ restrict keyword (16 tests)
- ✅ for loop declarations (16 tests)
- ✅ mixed declarations (15 tests)
- ✅ ... and 51 more features

**Status**: 🔄 In Progress (Sprint 5)

---

### 8. Test Fixtures (`fixtures/`)
**Purpose**: Reusable test data for multiple test categories

**Structure**:
```
fixtures/
├── c_programs/          # Sample C programs
│   ├── typical_program.c
│   ├── complex_pointers.c
│   └── edge_cases.c
└── expected_rust/       # Expected transpilation output
    ├── typical_program.rs
    └── complex_pointers.rs
```

**Usage**:
```rust
let c_code = include_str!("../fixtures/c_programs/typical_program.c");
let expected = include_str!("../fixtures/expected_rust/typical_program.rs");
```

---

## Running Tests

### Quick Validation (Development)
```bash
# All unit tests
cargo test

# Specific category
cargo test --test basic_transpilation_test
cargo test properties/
cargo test torture/
cargo test unsafe_audit/
```

### Comprehensive Validation (CI / Release)
```bash
# Documentation tests (C99 features)
cargo test --package decy-codegen --test '*_documentation_test'

# Integration tests
cargo test --test integration/*

# Property tests (100K cases)
PROPTEST_CASES=100000 cargo test properties/ --release

# Torture tests
cargo test torture/ --release

# Unsafe audit (including file scans)
cargo test unsafe_audit/ --ignored

# All tests
cargo test --workspace --all-features
```

### Coverage Analysis
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Open report
open coverage/index.html
```

### Mutation Testing (Weekly)
```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation tests (slow!)
cargo mutants --workspace --timeout 300

# Target: ≥90% mutation kill rate
```

---

## Quality Gates

### Pre-Commit Gates (MUST PASS)

1. **Documentation Tests**: All C99 feature tests pass
   ```bash
   cargo test --package decy-codegen --test '*_documentation_test'
   ```

2. **Integration Tests**: End-to-end transpilation succeeds
   ```bash
   cargo test --test integration/*
   ```

3. **Unsafe Audit**: <5 unsafe per 1000 LOC
   ```bash
   cargo test --test unsafe_audit
   ```

4. **Property Tests**: All invariants hold
   ```bash
   cargo test properties/
   ```

5. **Coverage**: ≥80% overall, ≥90% for decy-ownership
   ```bash
   cargo tarpaulin --workspace
   ```

6. **Clippy**: Zero warnings
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

7. **SATD Comments**: Zero technical debt
   ```bash
   ./scripts/check-satd.sh
   ```

---

## Test Metrics Dashboard

| Metric | Target | Current | Status |
|--------|---------|---------|--------|
| **Documentation Tests** | 150 | 57 (38%) | 🔄 In Progress |
| **Integration Tests** | 50 | 8 (16%) | 🔄 In Progress |
| **Property Tests** | 20 | 7 (35%) | 🔄 In Progress |
| **Torture Tests** | 50 | 13 (26%) | 🔄 In Progress |
| **Unsafe Audit Tests** | 15 | 10 (67%) | 🔄 In Progress |
| **Regression Tests** | 100% issues | 2/2 (100%) | ✅ Complete |
| **Differential Tests** | 10 | 0 (0%) | 🎯 Pending |
| **Unsafe per 1000 LOC** | <5 | 0 | ✅ Excellent |
| **Line Coverage** | ≥80% | TBD | 🎯 Pending |
| **Mutation Score** | ≥90% | TBD | 🎯 Pending |

---

## Test-to-Code Ratio

**Target**: 100:1 minimum (500,000 test instances for ~5,000 LOC codebase)

**Current Progress**:
- Unit tests: TBD
- Documentation tests: 57 features × ~15 tests = ~855 tests
- Integration tests: 8 tests
- Property tests: 7 properties × 100 cases = 700 test cases (dev mode)
- Property tests: 7 properties × 100K cases = 700,000 test cases (release mode)
- Torture tests: 13 tests
- Unsafe audit: 10 tests
- Regression tests: 2 tests

**Total Test Instances** (dev mode): ~1,588
**Total Test Instances** (release mode): ~701,588

**Ratio** (release mode): **140:1** (exceeds target!) 🎉

---

## Implementation Roadmap

### ✅ Phase 1: Foundation (Sprint 5-6)
- ✅ Test directory structure created
- ✅ Integration test framework (8 tests)
- ✅ Property test harness (7 properties)
- ✅ Torture test suite (13 tests)
- ✅ Unsafe audit framework (10 tests)
- ✅ Documentation tests (57/150 features)

### 🔄 Phase 2: Coverage (Sprint 7-8)
- 🎯 Complete documentation tests (150/150 features)
- 🎯 Add 42 more integration tests (to reach 50)
- 🎯 Add 13 more property tests (to reach 20)
- 🎯 Add 37 more torture tests (to reach 50)
- 🎯 Achieve 80% line coverage
- 🎯 Achieve 90% ownership coverage

### 🎯 Phase 3: Advanced (Sprint 9-10)
- 🎯 Implement differential testing (10 tests)
- 🎯 Mutation testing (≥90% score)
- 🎯 Fuzz testing integration
- 🎯 Performance benchmarking
- 🎯 CI/CD automation

### 🎯 Phase 4: SQLite-Level (Sprint 11-12)
- 🎯 100K+ property test cases per property
- 🎯 Comprehensive torture test coverage
- 🎯 Automated test generation from C99 spec
- 🎯 Formal verification exploration

---

## Best Practices

### Writing Tests

1. **Every test must have a clear purpose**
   - Document the C99 feature being tested
   - Reference ISO C99 section
   - Link to roadmap task ID

2. **Every test must be deterministic**
   - No random values without seeds
   - No timing-dependent behavior
   - Reproducible failures

3. **Every test must be fast**
   - Unit tests: <10ms
   - Integration tests: <100ms
   - Property tests: budget wisely

4. **Every unsafe block must be tested**
   - Test safety preconditions
   - Test edge cases that could violate safety
   - Document why unsafe is necessary

### Test Naming Convention

```rust
// Pattern: test_{category}_{feature}_{scenario}
#[test]
fn test_integration_pointer_arithmetic_basic() { }

#[test]
fn test_property_parser_never_panics() { }

#[test]
fn test_torture_deeply_nested_expressions() { }

#[test]
fn test_audit_unsafe_block_count() { }
```

---

## Continuous Improvement

### Daily
- Review test failures and fix immediately
- Add tests for new features

### Per Commit
- All quality gates must pass
- Coverage cannot decrease

### Weekly
- Review coverage reports
- Add missing tests
- Update metrics dashboard

### Per Sprint
- Comprehensive test metrics review
- Plan next phase of test implementation
- Benchmark against SQLite standards

---

## References

- **Specification**: `docs/specifications/testing-sqlite-style.md`
- **Roadmap**: `docs/C-VALIDATION-ROADMAP.yaml`
- **CI**: `.github/workflows/quality.yml`
- **SQLite Testing**: https://www.sqlite.org/testing.html

---

**Remember**: This is not just testing—this is building quality into every line of code from the start.

*"Quality is not an act, it is a habit." - Aristotle*
