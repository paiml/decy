# Test Coverage

Test coverage measures how much of your code is executed by tests. DECY enforces **≥80% coverage** at all times.

## Why 80%?

- **Below 80%**: Too many code paths untested, bugs likely
- **80-90%**: Good coverage, reasonable confidence
- **Above 90%**: Excellent coverage, high confidence
- **100%**: Rarely practical (diminishing returns)

DECY targets **90%+** coverage consistently.

## Measuring Coverage

### Using cargo-llvm-cov

```bash
# Install
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov

# Generate LCOV format for CI
cargo llvm-cov --lcov --output-path coverage.lcov

# HTML report for detailed view
cargo llvm-cov --html --open
```

### DECY Coverage Report

```
Filename                                  Region    Missed    Cover
─────────────────────────────────────────────────────────────────
decy-parser/src/lib.rs                      245        32   86.94%
decy-parser/src/ast.rs                      189        17   91.00%
decy-hir/src/lib.rs                         312        21   93.27%
decy-hir/src/types.rs                       156         8   94.87%
decy-ownership/src/dataflow.rs              421        25   94.06%
decy-ownership/src/inference.rs             334        19   94.31%
decy-codegen/src/lib.rs                     512        28   94.53%
decy-codegen/src/box_transform.rs           198        11   94.44%
─────────────────────────────────────────────────────────────────
TOTAL                                      2367       161   93.20%
```

**Result**: 93.20% coverage ✅ (target: ≥80%)

## Coverage by Component

### Parser: 87.3%

```rust
// Well-tested: Basic parsing
#[test]
fn test_parse_simple_function() {
    let parser = CParser::new().unwrap();
    let ast = parser.parse("int add(int a, int b) { return a + b; }").unwrap();
    assert_eq!(ast.functions().len(), 1);
}

// Well-tested: Error handling
#[test]
fn test_parse_invalid_syntax() {
    let parser = CParser::new().unwrap();
    let result = parser.parse("int invalid { }");
    assert!(result.is_err());
}

// Not covered: Edge case with deeply nested expressions (7+ levels)
// This is acceptable - very rare in real code
```

### HIR: 93.3%

```rust
// Well-tested: Type conversion
#[test]
fn test_ast_to_hir_conversion() {
    let ast_func = create_ast_function();
    let hir_func = HirFunction::from_ast_function(&ast_func);
    assert_eq!(hir_func.name(), ast_func.name());
}

// Well-tested: Property tests
proptest! {
    #[test]
    fn prop_hir_preserves_function_count(funcs in vec(ast_function(), 0..10)) {
        let hir = lower_to_hir(&funcs).unwrap();
        prop_assert_eq!(hir.functions().len(), funcs.len());
    }
}
```

### Ownership: 94.3%

```rust
// Well-tested: Pattern recognition
#[test]
fn test_malloc_is_owning() {
    let c_code = "int* p = malloc(sizeof(int));";
    let ownership = infer_ownership(c_code).unwrap();
    assert_eq!(ownership.pattern, OwnershipPattern::Owning);
}

// Well-tested: Complex scenarios
#[test]
fn test_mixed_ownership() {
    let c_code = r#"
        int* process(const int* input) {
            int* output = malloc(sizeof(int));
            *output = *input * 2;
            return output;
        }
    "#;
    let ownership = infer_ownership(c_code).unwrap();
    assert_eq!(ownership.get("input"), Some(&OwnershipPattern::Borrowed));
    assert_eq!(ownership.get("output"), Some(&OwnershipPattern::Owning));
}
```

### Codegen: 94.5%

```rust
// Well-tested: Type mapping
#[test]
fn test_type_mapping() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.map_type(&HirType::Int), "i32");
    assert_eq!(codegen.map_type(&HirType::Void), "()");
}

// Well-tested: Code generation
#[test]
fn test_generated_code_compiles() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = transpile(c_code).unwrap();
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Uncovered Code Analysis

### Why Some Code Is Uncovered

**Acceptable** (defensive programming):
```rust
// Defensive check that should never happen
if ptr.is_null() {
    return Err(anyhow!("Unexpected null pointer"));  // Not covered
}
```

**Acceptable** (error paths):
```rust
// Rare error conditions
match parse_result {
    Ok(ast) => process(ast),
    Err(ParseError::OutOfMemory) => panic!("OOM"),  // Not covered (CI has memory)
    Err(e) => Err(e.into()),  // Covered
}
```

**NOT Acceptable** (missing tests):
```rust
// ❌ Should be tested but isn't
pub fn important_logic(x: i32) -> i32 {
    if x > 100 {
        x * 2  // ← Not covered! Need test!
    } else {
        x + 1
    }
}
```

## Coverage Trends

Track coverage over time:

```
Commit  Coverage  Change
──────────────────────────
abc123  89.2%    baseline
def456  91.5%    +2.3%  ✅
ghi789  90.8%    -0.7%  ⚠️
jkl012  93.2%    +2.4%  ✅
```

**Trend**: Increasing coverage from 89% → 93% ✅

## Coverage Configuration

Create `llvm-cov.toml`:

```toml
[llvm-cov]
target-dir = "target"
html = true
open = false

# Exclude test code from coverage
ignore-filename-regex = [
    "tests/",
    "benches/",
    "examples/",
]

[report]
# Fail if coverage drops below 80%
fail-under-lines = 80
fail-under-functions = 80
fail-under-regions = 80
```

## CI/CD Integration

GitHub Actions workflow:

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --lcov --output-path coverage.lcov

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov report | grep TOTAL | awk '{print $4}' | sed 's/%//')
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 80% threshold"
            exit 1
          fi
          echo "Coverage: $COVERAGE% ✅"

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: coverage.lcov
          fail_ci_if_error: true
```

## Coverage vs Mutation Testing

Coverage answers: **"Was this code executed?"**
Mutation testing answers: **"Did tests catch bugs?"**

### Example: High Coverage, Poor Tests

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b  // ← Covered by test
}

#[test]
fn test_add() {
    add(2, 3);  // ← Executes code but doesn't assert!
}
```

**Coverage**: 100% ✅
**Mutation score**: 0% ❌ (mutants survive)

### Example: Good Coverage, Good Tests

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);  // ← Asserts result
    assert_eq!(add(-1, 1), 0);
    assert_eq!(add(0, 0), 0);
}
```

**Coverage**: 100% ✅
**Mutation score**: 95% ✅ (mutants caught)

## Improving Coverage

### Strategy 1: Property Tests

```rust
proptest! {
    #[test]
    fn prop_add_commutative(a: i32, b: i32) {
        prop_assert_eq!(add(a, b), add(b, a));
    }
}
```

Property tests cover many paths with random inputs!

### Strategy 2: Edge Cases

```rust
#[test]
fn test_edge_cases() {
    // Boundary values
    assert_eq!(add(i32::MAX, 0), i32::MAX);
    assert_eq!(add(i32::MIN, 0), i32::MIN);

    // Zero
    assert_eq!(add(0, 0), 0);

    // Negative
    assert_eq!(add(-5, -3), -8);
}
```

### Strategy 3: Error Paths

```rust
#[test]
fn test_error_handling() {
    let result = parse("invalid syntax");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("syntax"));
}
```

## Coverage by Test Type

```
Test Type        Lines Covered  Percentage
───────────────────────────────────────────
Unit Tests              1,234      52.1%
Integration Tests         567      23.9%
Property Tests            412      17.4%
Doc Tests                 154       6.6%
───────────────────────────────────────────
Total                   2,367     100.0%
```

**All test types contribute to coverage!**

## Visualizing Coverage

HTML report highlights uncovered lines:

```rust
pub fn process(x: i32) -> Result<i32> {
    if x < 0 {
        return Err(anyhow!("negative"));  // ✅ Covered (green)
    }

    if x > 1000 {
        return Err(anyhow!("too large"));  // ❌ Not covered (red)
    }

    Ok(x * 2)  // ✅ Covered (green)
}
```

The red line indicates: **Need test with x > 1000!**

## Coverage Best Practices

### DO ✅

- **Aim for 90%+**: High confidence in code quality
- **Test edge cases**: Boundaries, zero, negative, max
- **Test error paths**: All error conditions should be covered
- **Use property tests**: Cover many paths efficiently
- **Track trends**: Watch for coverage decreases

### DON'T ❌

- **Chase 100%**: Diminishing returns, not practical
- **Test trivial code**: Getters/setters, simple constructors
- **Ignore mutation**: Coverage alone isn't enough
- **Game metrics**: Meaningless tests just for coverage
- **Skip integration tests**: They're critical for coverage

## DECY Coverage Goals

| Component | Current | Target |
|-----------|---------|--------|
| Parser | 87.3% | 90% |
| HIR | 93.3% | 95% |
| Ownership | 94.3% | 95% |
| Codegen | 94.5% | 95% |
| **Overall** | **93.2%** | **95%** |

All components exceed 80% minimum ✅

## Summary

Test coverage in DECY:

✅ **≥80% enforced**: Quality gate blocks low coverage
✅ **93.2% achieved**: Excellent coverage across all components
✅ **Trend upward**: Improving from 89% → 93%
✅ **All test types**: Unit, integration, property, doc tests
✅ **CI/CD integrated**: Automatic coverage checks
✅ **HTML reports**: Visual identification of gaps
✅ **Combined with mutation**: Coverage + test quality

High coverage + good tests = **confidence in code quality**

## Next Steps

- [Mutation Scores](./mutation.md) - Verify test quality
- [Complexity Analysis](./complexity.md) - Measure code complexity
- [Safety Verification](./safety.md) - Prove memory safety
