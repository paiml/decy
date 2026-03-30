# SQLite-Style Testing Specification v1.0

**Document ID**: DECY-SPEC-TEST-001
**Version**: 1.0
**Status**: ACTIVE
**Last Updated**: October 15, 2025
**Authors**: Decy Development Team

## Executive Summary

This specification defines comprehensive testing standards for the Decy C-to-Rust transpiler based on SQLite's legendary testing methodology (100% branch coverage, 248.5M tests before release) and best practices from Rust, LLVM, GCC, and other mature compiler ecosystems. The goal is to achieve aerospace-grade reliability with **<5 unsafe blocks per 1000 LOC** while maintaining comprehensive validation of every C99 construct transformation.

## Research Foundation

### Industry Analysis: Compiler Testing Best Practices

| Project | Primary Framework | Key Innovation | Adoption Insight |
|---------|-------------------|----------------|------------------|
| **SQLite** | TH3 + TCL + SLT | 100% branch coverage, 248.5M test instances | Gold standard: 614 tests per line of code |
| **Rust** | Built-in test + Cargo | Specification testing (2025), UI tests for diagnostics | Integration tests in `tests/` directory |
| **LLVM** | lit + FileCheck | Differential testing, IR validation | 100K+ regression tests |
| **GCC** | DejaGnu + TCL | Torture tests, optimization verification | 40+ years of test accumulation |
| **CompCert** | Coq proofs + tests | Formally verified correctness | Mathematical proof of soundness |
| **Clang** | lit + AST matchers | Static analysis testing, diagnostic verification | AST-based validation |
| **Zig** | Built-in test runner | Compile-time execution, behavior tests | Zero external dependencies |

### Key Universal Patterns Identified

1. **Multi-Harness Testing**: All mature compilers use multiple complementary test approaches
2. **Differential Testing**: Compare output against reference implementations (GCC, Clang)
3. **Torture Tests**: Extreme edge cases and stress testing (GCC torture suite)
4. **Regression Testing**: Every bug becomes a permanent test (LLVM model)
5. **Coverage-Driven**: 100% branch coverage for mission-critical code
6. **Property-Based Testing**: Automated invariant validation
7. **Documentation Testing**: All examples must compile and run
8. **Unsafe Auditing**: Track and minimize unsafe blocks systematically

## SQLite Testing Philosophy Applied to Decy

### The 614:1 Test Ratio

SQLite achieves **614 lines of test code per line of application code**. For Decy:

- **Target Ratio**: 100:1 minimum (more achievable for a transpiler)
- **Current LOC**: ~5,000 lines of Rust code
- **Target Tests**: 500,000 test instances minimum
- **Method**: Combination of unit, integration, property, documentation, and torture tests

### 100% Branch Coverage

SQLite achieves 100% branch coverage. For Decy:

- **Coverage Target**: 80% overall, 90% for `decy-ownership` (critical path)
- **Mutation Testing**: ≥90% mutation kill rate by Sprint 10
- **Quality Gates**: Coverage cannot decrease between commits

## Decy Multi-Harness Testing Framework

### Core Testing Categories

#### 1. **Unit Tests** - Component-Level Validation
**Scope**: Individual functions and modules
**Coverage Target**: 85% line coverage
**Location**: `#[cfg(test)] mod tests` in each crate
**Examples**:
- Parser: Individual C construct parsing (expressions, statements, declarations)
- HIR: AST node creation and manipulation
- Analyzer: Control flow, data flow, type inference
- Ownership: Pointer classification, lifetime inference
- Codegen: Rust code generation for specific patterns

**Test Pattern**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function_declaration() {
        let c_code = "int add(int a, int b);";
        let result = parse_declaration(c_code);

        assert!(result.is_ok());
        let decl = result.unwrap();
        assert_eq!(decl.name, "add");
        assert_eq!(decl.return_type, Type::Int);
        assert_eq!(decl.params.len(), 2);
    }
}
```

#### 2. **Documentation Tests** - C99 Feature Validation
**Scope**: Every C99 construct documented with executable tests
**Coverage Target**: 150 C99 features (from C-VALIDATION-ROADMAP.yaml)
**Location**: `crates/decy-codegen/tests/*_documentation_test.rs`
**Examples**:
- `long_long_type_documentation_test.rs` (16 tests)
- `hexadecimal_float_literals_documentation_test.rs` (15 tests)
- `inline_functions_documentation_test.rs` (16 tests)

**Test Pattern**:
```rust
/// Document transformation of C construct to Rust
///
/// C99 Reference: ISO C99 §X.Y.Z
#[test]
fn test_c99_feature_transformation() {
    let _c_code = r#"
    /* C code example */
    int x = 42;
    "#;

    let _rust_equivalent = r#"
    // Rust equivalent
    let x: i32 = 42;
    "#;

    // Executable Rust code demonstrating transformation
    let x: i32 = 42;
    assert_eq!(x, 42);
}
```

**Current Status**: 57/150 features documented (38%)

#### 3. **Integration Tests** - End-to-End Transpilation
**Scope**: Complete C programs transpiled to Rust and compiled
**Coverage Target**: All major C99 language features in combination
**Location**: `tests/integration/*.rs`
**Examples**:
- `pointer_arithmetic_test.rs` - Pointer transformations
- `control_flow_test.rs` - If/for/while/switch integration
- `struct_test.rs` - Struct definitions and usage
- `malloc_free_test.rs` - Memory management patterns

**Test Pattern**:
```rust
#[test]
fn test_complete_c_program_transpilation() {
    let c_program = r#"
    #include <stdio.h>

    int main() {
        int x = 42;
        printf("Value: %d\n", x);
        return 0;
    }
    "#;

    // Parse → HIR → Analyze → Ownership → Codegen
    let rust_code = transpile(c_program).expect("transpilation failed");

    // Verify it compiles
    assert!(compile_rust(&rust_code).is_ok());

    // Verify output matches
    let c_output = run_c_program(c_program);
    let rust_output = run_rust_program(&rust_code);
    assert_eq!(c_output, rust_output);
}
```

#### 4. **Property-Based Tests** - Invariant Validation
**Scope**: Mathematical properties and invariants
**Coverage Target**: All critical invariants in parser, analyzer, ownership
**Location**: `tests/properties/*.rs`, `proptest-regressions/`
**Examples**:
- Parser: Parse → Format → Parse roundtrip
- Ownership: Unique owner per allocation
- Lifetimes: Borrows don't outlive owners
- Type safety: C types map to valid Rust types

**Test Pattern**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_parse_format_roundtrip(c_code in valid_c_expression()) {
        let ast = parse_expression(&c_code)?;
        let formatted = format_expression(&ast);
        let re_parsed = parse_expression(&formatted)?;

        prop_assert_eq!(ast, re_parsed);
    }

    #[test]
    fn property_ownership_unique(c_code in c_malloc_free_pattern()) {
        let ownership = infer_ownership(&c_code)?;

        // Every allocation has exactly one owner
        for alloc in ownership.allocations() {
            prop_assert_eq!(ownership.owners(alloc).count(), 1);
        }
    }
}
```

#### 5. **Torture Tests** - Extreme Edge Cases
**Scope**: Pathological inputs, stress testing, compiler limits
**Coverage Target**: Edge cases that break typical implementations
**Location**: `tests/torture/*.rs`
**Examples**:
- Deeply nested expressions (1000+ levels)
- Extremely long identifiers (10K characters)
- Complex pointer arithmetic chains
- Massive struct hierarchies
- Edge-case numeric literals (LLONG_MAX, denormals)

**Test Pattern**:
```rust
#[test]
fn torture_deeply_nested_expressions() {
    // Generate 1000-level nested expression: (((((...x)))))
    let mut c_code = String::from("x");
    for _ in 0..1000 {
        c_code = format!("({})", c_code);
    }

    // Should parse without stack overflow
    let result = parse_expression(&c_code);
    assert!(result.is_ok());
}

#[test]
fn torture_pointer_arithmetic_chain() {
    // Test complex pointer arithmetic: p[i][j] + *(p + i) + (p + i)->field
    let c_code = r#"
    struct Node { int* data; };
    int compute(struct Node** p, int i, int j) {
        return p[i][j] + *(p[i]->data + j) + (p + i)[0]->data[j];
    }
    "#;

    let rust_code = transpile(c_code).expect("torture test failed");
    assert!(compile_rust(&rust_code).is_ok());
}
```

#### 6. **Regression Tests** - Bug Prevention
**Scope**: Every GitHub issue, every discovered bug
**Coverage Target**: 100% of reported issues
**Location**: `tests/regression/github_issue_*.rs`
**Examples**:
- `github_issue_001_main_function.rs` - DECY-AUDIT-001 fix
- `github_issue_002_cli_guidance.rs` - DECY-AUDIT-002 fix
- Future issues from validation roadmap

**Test Pattern**:
```rust
/// Regression test for GitHub Issue #42: Parser crash on empty struct
///
/// Bug: Parser panicked on `struct Empty {};`
/// Fix: Handle empty struct member list
/// Ticket: DECY-042
#[test]
fn github_issue_042_empty_struct_crash() {
    let c_code = "struct Empty {};";

    // Should not panic
    let result = parse_declaration(c_code);

    // Should succeed
    assert!(result.is_ok());
    let decl = result.unwrap();
    assert_eq!(decl.members.len(), 0);
}
```

#### 7. **Differential Tests** - Reference Comparison
**Scope**: Compare Decy output against GCC, Clang, rustc
**Coverage Target**: Semantic equivalence for all valid C99
**Location**: `tests/differential/*.rs`
**Examples**:
- Compare GCC compilation output vs Decy transpilation output
- Verify rustc accepts Decy-generated code
- Cross-validate with Clang AST

**Test Pattern**:
```rust
#[test]
fn differential_numeric_operations() {
    let c_code = r#"
    int main() {
        int a = 2147483647;  // INT_MAX
        int b = a + 1;        // Overflow
        return b;
    }
    "#;

    // Compile with GCC
    let gcc_output = compile_and_run_gcc(c_code).expect("GCC failed");

    // Transpile with Decy
    let rust_code = transpile(c_code).expect("Decy failed");
    let decy_output = compile_and_run_rust(&rust_code).expect("rustc failed");

    // Note: C undefined behavior vs Rust defined behavior
    // Document the difference, don't just assert equality
    println!("GCC output: {}", gcc_output);
    println!("Decy output: {}", decy_output);
    assert!(decy_output.contains("panic") || decy_output == gcc_output);
}
```

#### 8. **Mutation Tests** - Test Quality Validation
**Scope**: Validate test suite effectiveness
**Coverage Target**: ≥90% mutation kill rate
**Location**: `cargo mutants --workspace`
**Run Schedule**: CI only (too slow for local)

**Quality Metrics**:
- **Mutation Score**: Percentage of mutants killed by tests
- **Target**: 90% by Sprint 10
- **Current**: Baseline being established
- **Tool**: `cargo-mutants`

#### 9. **Unsafe Auditing Tests** - Safety Validation
**Scope**: Track, minimize, and audit every unsafe block
**Coverage Target**: <5 unsafe blocks per 1000 LOC
**Location**: `tests/unsafe_audit/*.rs`
**Examples**:
- Count unsafe blocks in generated code
- Verify SAFETY comments on all unsafe
- Test unsafe elimination patterns

**Test Pattern**:
```rust
#[test]
fn audit_unsafe_block_count() {
    let c_code = include_str!("../fixtures/typical_program.c");
    let rust_code = transpile(c_code).expect("transpilation failed");

    let unsafe_count = count_unsafe_blocks(&rust_code);
    let loc = count_lines(&rust_code);
    let unsafe_per_1000 = (unsafe_count as f64 / loc as f64) * 1000.0;

    assert!(
        unsafe_per_1000 < 5.0,
        "Unsafe block count exceeded: {} per 1000 LOC (target: <5)",
        unsafe_per_1000
    );
}

#[test]
fn audit_all_unsafe_have_safety_comments() {
    let c_code = include_str!("../fixtures/typical_program.c");
    let rust_code = transpile(c_code).expect("transpilation failed");

    let unsafe_blocks = extract_unsafe_blocks(&rust_code);

    for block in unsafe_blocks {
        assert!(
            block.has_safety_comment(),
            "Unsafe block at line {} missing SAFETY comment",
            block.line
        );
    }
}
```

## Test Organization Structure

```
tests/
├── unit/                           # Unit tests (if separate from src)
│   ├── parser_unit_tests.rs
│   ├── hir_unit_tests.rs
│   └── ownership_unit_tests.rs
│
├── integration/                    # End-to-end integration tests
│   ├── pointer_arithmetic_test.rs
│   ├── control_flow_test.rs
│   ├── struct_test.rs
│   └── malloc_free_test.rs
│
├── properties/                     # Property-based tests
│   ├── parser_properties.rs
│   ├── ownership_properties.rs
│   └── type_safety_properties.rs
│
├── torture/                        # Extreme edge case tests
│   ├── deeply_nested_test.rs
│   ├── pointer_chains_test.rs
│   └── compiler_limits_test.rs
│
├── regression/                     # Bug prevention tests
│   ├── github_issue_001.rs
│   ├── github_issue_002.rs
│   └── audit_fixes.rs
│
├── differential/                   # Reference comparison tests
│   ├── gcc_comparison_test.rs
│   ├── clang_comparison_test.rs
│   └── semantic_equivalence_test.rs
│
├── unsafe_audit/                   # Unsafe block tracking
│   ├── unsafe_count_test.rs
│   ├── safety_comment_test.rs
│   └── elimination_patterns_test.rs
│
└── fixtures/                       # Test data
    ├── c_programs/                 # Sample C programs
    │   ├── typical_program.c
    │   ├── complex_pointers.c
    │   └── edge_cases.c
    └── expected_rust/              # Expected transpilation output
        ├── typical_program.rs
        └── complex_pointers.rs

crates/
└── decy-codegen/
    └── tests/                      # Documentation tests (150 features)
        ├── long_long_type_documentation_test.rs
        ├── hexadecimal_float_literals_documentation_test.rs
        ├── inline_functions_documentation_test.rs
        └── ... (147 more to implement)
```

## Quality Gates and CI Integration

### Mandatory Quality Gates

#### 1. **Documentation Test Gate**
```bash
# All C99 feature documentation tests must pass
cargo test --test '*_documentation_test' -- --nocapture
# Fail if ANY C99 feature test fails
```

#### 2. **Integration Test Gate**
```bash
# End-to-end transpilation tests must pass
cargo test --test 'integration/*' -- --nocapture
# Fail if transpilation produces invalid Rust
```

#### 3. **Coverage Gate**
```bash
# Coverage must meet minimum thresholds
cargo tarpaulin --workspace --out Html --output-dir coverage/
# Fail if coverage < 80% (or < 90% for decy-ownership)
```

#### 4. **Unsafe Audit Gate**
```bash
# Unsafe block count must be under limit
cargo test --test unsafe_audit -- --nocapture
# Fail if unsafe blocks per 1000 LOC > 5
```

#### 5. **Property Test Gate**
```bash
# Property-based tests must pass
cargo test properties/ --release
# Run for minimum 100K test cases
```

#### 6. **Clippy Gate**
```bash
# Zero clippy warnings allowed
cargo clippy --workspace -- -D warnings
# Fail on ANY clippy warning
```

#### 7. **SATD Comment Gate**
```bash
# Zero technical debt comments allowed
./scripts/check-satd.sh
# Fail on TODO, FIXME, HACK, XXX, TEMP, WIP, BROKEN
```

### CI Pipeline Integration

```yaml
# .github/workflows/sqlite-style-testing.yml
name: SQLite-Style Comprehensive Testing
on: [push, pull_request]

jobs:
  documentation-tests:
    name: C99 Feature Documentation Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run documentation tests
        run: |
          cargo test --package decy-codegen \
            --test '*_documentation_test' \
            -- --nocapture --test-threads=1

      - name: Generate documentation coverage report
        run: |
          echo "Documentation Test Coverage: $(cargo test --list | grep documentation_test | wc -l) / 150 features"

  integration-tests:
    name: End-to-End Integration Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install LLVM/Clang
        run: |
          sudo apt-get update
          sudo apt-get install -y llvm-14 libclang-14-dev clang-14

      - name: Run integration tests
        run: cargo test --test 'integration/*' -- --nocapture

  property-tests:
    name: Property-Based Robustness Tests
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run property tests (100K cases)
        run: |
          cargo test properties/ --release -- \
            --nocapture \
            --test-threads=1
        env:
          PROPTEST_CASES: 100000

  torture-tests:
    name: Torture Tests (Extreme Edge Cases)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run torture tests
        run: cargo test torture/ --release -- --nocapture

  unsafe-audit:
    name: Unsafe Block Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Audit unsafe block count
        run: |
          cargo test --test unsafe_audit -- --nocapture

      - name: Generate unsafe audit report
        run: |
          ./scripts/audit-unsafe.sh > unsafe-audit-report.txt
          cat unsafe-audit-report.txt

      - name: Upload unsafe audit report
        uses: actions/upload-artifact@v4
        with:
          name: unsafe-audit-${{ github.sha }}
          path: unsafe-audit-report.txt

  coverage:
    name: Code Coverage Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage report
        run: |
          cargo tarpaulin \
            --workspace \
            --out Html \
            --out Xml \
            --output-dir coverage/ \
            --timeout 600 \
            --exclude-files 'tests/*'

      - name: Check coverage thresholds
        run: |
          python3 scripts/check-coverage-thresholds.py \
            --overall 80 \
            --ownership 90 \
            --report coverage/cobertura.xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage/cobertura.xml

  mutation-tests:
    name: Mutation Testing (Weekly)
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'  # Run weekly, not on every commit
    timeout-minutes: 180
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation tests
        run: |
          cargo mutants --workspace \
            --timeout 300 \
            --output mutants.json

      - name: Check mutation score
        run: |
          python3 scripts/check-mutation-score.py \
            --target 90 \
            --report mutants.json

  differential-tests:
    name: Differential Testing (GCC/Clang)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install GCC and Clang
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc clang

      - name: Run differential tests
        run: cargo test differential/ -- --nocapture

  quality-gates:
    name: Quality Gates (Clippy, SATD, Format)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy (zero warnings)
        run: cargo clippy --workspace -- -D warnings

      - name: Check for SATD comments
        run: ./scripts/check-satd.sh

      - name: Check complexity
        run: cargo install cargo-geiger && cargo geiger

  comprehensive-report:
    name: Comprehensive Testing Report
    needs: [documentation-tests, integration-tests, property-tests, torture-tests, unsafe-audit, coverage]
    runs-on: ubuntu-latest
    if: always()
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Generate comprehensive report
        run: |
          python3 scripts/generate-test-report.py \
            --output comprehensive-test-report.html

      - name: Upload comprehensive report
        uses: actions/upload-artifact@v4
        with:
          name: test-report-${{ github.sha }}
          path: comprehensive-test-report.html
```

## Metrics and Success Criteria

### Testing Metrics Dashboard

| Metric | Target | Current | Status |
|--------|---------|---------|--------|
| **Documentation Tests** | 150/150 (100%) | 57/150 (38%) | 🔄 In Progress |
| **Line Coverage** | ≥80% | TBD | 🎯 Pending |
| **Branch Coverage** | ≥75% | TBD | 🎯 Pending |
| **Ownership Coverage** | ≥90% | TBD | 🎯 Critical |
| **Mutation Score** | ≥90% | TBD | 🎯 Pending |
| **Unsafe per 1000 LOC** | <5 | 0 | ✅ Excellent |
| **Integration Tests** | ≥50 | TBD | 🎯 Pending |
| **Property Tests** | 100K cases | TBD | 🎯 Pending |
| **Torture Tests** | ≥20 | 0 | 🎯 Pending |
| **Regression Tests** | 100% issues | 2/2 (100%) | ✅ Good |
| **Differential Tests** | ≥10 | 0 | 🎯 Pending |

### Test-to-Code Ratio

| Crate | LOC | Test LOC | Ratio | Target |
|-------|-----|----------|-------|--------|
| **decy-parser** | ~800 | TBD | TBD | 50:1 |
| **decy-hir** | ~400 | TBD | TBD | 50:1 |
| **decy-analyzer** | ~600 | TBD | TBD | 50:1 |
| **decy-ownership** | ~500 | TBD | TBD | 100:1 (critical) |
| **decy-codegen** | ~1000 | TBD | TBD | 50:1 |
| **decy-verify** | ~300 | TBD | TBD | 50:1 |
| **Overall** | ~5000 | TBD | TBD | **100:1 minimum** |

**Target**: 500,000 total test instances (inspired by SQLite's 614:1 ratio, adapted for transpiler complexity)

### Quality Trends Tracking

```rust
// Track quality metrics over time
struct QualityMetrics {
    sprint: u32,
    date: String,

    // Coverage metrics
    line_coverage: f64,
    branch_coverage: f64,
    ownership_coverage: f64,

    // Test counts
    documentation_tests: usize,
    integration_tests: usize,
    property_test_cases: usize,

    // Quality indicators
    unsafe_per_1000_loc: f64,
    mutation_score: f64,
    clippy_warnings: usize,
    satd_comments: usize,

    // Performance
    avg_test_duration_ms: f64,
    total_test_count: usize,
}
```

## Implementation Roadmap

### Phase 1: Foundation (Sprint 6-7)
- ✅ Documentation test infrastructure (57/150 features)
- 🎯 Create test organization structure
- 🎯 Implement integration test framework
- 🎯 Set up property test harness
- 🎯 Configure CI pipeline

### Phase 2: Coverage (Sprint 8-9)
- 🎯 Complete documentation tests (150/150 features)
- 🎯 Achieve 80% line coverage
- 🎯 Achieve 90% ownership coverage
- 🎯 Implement torture test suite
- 🎯 Add regression test framework

### Phase 3: Advanced (Sprint 10-11)
- 🎯 Differential testing against GCC/Clang
- 🎯 Mutation testing (≥90% score)
- 🎯 Unsafe auditing automation
- 🎯 Performance benchmarking
- 🎯 Fuzz testing integration

### Phase 4: SQLite-Level (Sprint 12+)
- 🎯 100,000+ test cases via property testing
- 🎯 Comprehensive torture test coverage
- 🎯 100% regression test coverage
- 🎯 Automated test generation from C99 spec
- 🎯 Formal verification exploration

## Best Practices and Guidelines

### Test Writing Guidelines

1. **Every test must have a clear purpose**
   ```rust
   /// Tests that C99 long long type maps to Rust i64
   ///
   /// Reference: ISO C99 §6.2.5
   /// Ticket: DECY-040
   #[test]
   fn test_long_long_to_i64_mapping() { ... }
   ```

2. **Every test must reference the C99 specification**
   - Include ISO C99 section number
   - Note if feature is NEW in C99 vs C89/K&R
   - Link to validation roadmap task ID

3. **Every test must be deterministic**
   - No random values without seeds
   - No timing-dependent behavior
   - Reproducible failures

4. **Every test must be fast**
   - Unit tests: <10ms
   - Integration tests: <100ms
   - Property tests: budget wisely (use `PROPTEST_CASES`)

5. **Every unsafe block must be tested**
   - Test the safety preconditions
   - Test edge cases that could violate safety
   - Document why unsafe is necessary

### Maintenance Schedule

- **Daily**: Review test failures and fix immediately
- **Per Commit**: All quality gates must pass
- **Weekly**: Review coverage reports and add missing tests
- **Per Sprint**: Comprehensive test metrics review
- **Per Release**: Full test suite execution including mutation tests
- **Quarterly**: Benchmark against SQLite testing standards

### Test Ownership

| Test Category | Owner | Priority |
|---------------|-------|----------|
| Documentation Tests | All team | Critical |
| Integration Tests | Core team | High |
| Property Tests | Ownership team | Critical |
| Torture Tests | QA team | Medium |
| Regression Tests | Developer who fixed | Critical |
| Differential Tests | Validation team | High |
| Mutation Tests | QA team | Medium |
| Unsafe Audit | Security team | Critical |

## Conclusion

This SQLite-style testing specification establishes Decy as a reliability-first transpiler, following the proven methodology of one of the most tested software projects in history. By adapting SQLite's 614:1 test-to-code ratio, 100% branch coverage philosophy, and multi-harness testing approach to the unique challenges of C-to-Rust transpilation, we create a foundation for aerospace-grade reliability.

The implementation of this specification will:
- **Prevent regressions** through comprehensive test coverage
- **Minimize unsafe code** through systematic auditing
- **Ensure correctness** through property-based and differential testing
- **Build confidence** through Toyota Way quality-first development
- **Enable fearless refactoring** through exhaustive test suites

This is not just testing—this is building quality into every line of code from the start.

---

**Status**: APPROVED
**Implementation Start**: Sprint 6 (Current)
**Full Implementation**: Sprint 12 (Target)
**Review Cycle**: Per Sprint
**Next Review**: Sprint 7
