# Mutation Testing

Mutation testing answers the question: **"How good are my tests?"**

## The Problem

You can have 100% test coverage but still have BAD tests:

```rust
pub fn is_positive(x: i32) -> bool {
    x > 0  // ← Covered by tests
}

#[test]
fn test_is_positive() {
    is_positive(5);  // ← Test runs but doesn't assert anything!
}
```

Coverage: 100% ✅
Quality: 0% ❌

## The Solution: Mutation Testing

Mutation testing **introduces bugs** into your code and verifies tests catch them.

### Original Code

```rust
pub fn is_positive(x: i32) -> bool {
    x > 0
}
```

### Mutant 1: Change > to <

```rust
pub fn is_positive(x: i32) -> bool {
    x < 0  // ← Mutated!
}
```

**Good tests will FAIL** with this mutant.
**Bad tests will PASS** (they didn't check the result!).

### Mutant 2: Change > to >=

```rust
pub fn is_positive(x: i32) -> bool {
    x >= 0  // ← Mutated!
}
```

### Mutant 3: Return false

```rust
pub fn is_positive(x: i32) -> bool {
    false  // ← Mutated!
}
```

## Good vs Bad Tests

### Bad Test (doesn't catch mutants)

```rust
#[test]
fn test_is_positive_bad() {
    is_positive(5);  // ❌ No assertion!
}
```

**Result**: ALL mutants survive ❌

### Good Test (catches mutants)

```rust
#[test]
fn test_is_positive_good() {
    assert!(is_positive(5));   // ✅ Catches mutant 2 and 3
    assert!(!is_positive(-5)); // ✅ Catches mutant 1
    assert!(!is_positive(0));  // ✅ Catches mutant 2
}
```

**Result**: ALL mutants killed ✅

## cargo-mutants

DECY uses `cargo-mutants` for mutation testing.

### Installation

```bash
cargo install cargo-mutants
```

### Basic Usage

```bash
# Test all mutants in the project
cargo mutants

# Test only changed code
cargo mutants --in-diff origin/main

# With timeout (kill slow tests)
cargo mutants --timeout 120
```

### Configuration

Create `mutants.toml`:

```toml
# Minimum mutation score to pass
minimum_test_timeout = 120

# Files to exclude
exclude_globs = [
    "tests/**",
    "benches/**",
    "**/test_*.rs",
]

# Specific mutations to exclude
exclude_mutants = [
    # Don't mutate panic messages
    "panic!",
    "unreachable!",
]
```

## Types of Mutants

### 1. Arithmetic Operators

```rust
// Original
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Mutants
a - b  // Replace + with -
a * b  // Replace + with *
a / b  // Replace + with /
a % b  // Replace + with %
```

### 2. Comparison Operators

```rust
// Original
fn is_greater(a: i32, b: i32) -> bool {
    a > b
}

// Mutants
a < b   // Replace > with <
a >= b  // Replace > with >=
a == b  // Replace > with ==
a != b  // Replace > with !=
```

### 3. Logical Operators

```rust
// Original
fn both_true(a: bool, b: bool) -> bool {
    a && b
}

// Mutants
a || b  // Replace && with ||
!a && b // Negate first operand
a && !b // Negate second operand
```

### 4. Return Values

```rust
// Original
fn get_value() -> i32 {
    42
}

// Mutants
return 0;   // Replace with 0
return 1;   // Replace with 1
return -1;  // Replace with -1
```

### 5. Conditionals

```rust
// Original
fn absolute(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}

// Mutants
if x > 0 { ... }    // Replace < with >
if x <= 0 { ... }   // Replace < with <=
if true { ... }     // Replace condition with true
if false { ... }    // Replace condition with false
```

### 6. Function Calls

```rust
// Original
fn process(x: i32) -> i32 {
    helper(x)
}

// Mutants
helper(0)  // Replace argument with 0
helper(1)  // Replace argument with 1
0          // Remove function call
```

## Mutation Testing Output

### Example Report

```
Testing 248 mutants
========================

[1/248] CAUGHT: decy_parser/src/lib.rs:45:12: replaced + with -
[2/248] CAUGHT: decy_parser/src/lib.rs:67:8: replaced && with ||
[3/248] CAUGHT: decy_hir/src/types.rs:23:16: replaced > with >=
...
[245/248] CAUGHT: decy_codegen/src/generate.rs:123:20: replaced return with 0
[246/248] MISSED: decy_codegen/src/generate.rs:145:12: replaced Some with None
[247/248] TIMEOUT: decy_ownership/src/infer.rs:89:16: infinite loop
[248/248] CAUGHT: decy_borrow/src/checker.rs:234:8: negated condition

========================
Results:
  Caught:    234 mutants (94.4%) ✅
  Missed:     14 mutants (5.6%)  ⚠️
  Timeout:     0 mutants
  Unviable:    0 mutants
========================
Kill rate: 94.4% (target: ≥90%) ✅
```

## Understanding Results

### CAUGHT (✅ Good!)

```
CAUGHT: src/lib.rs:45:12: replaced + with -
```

Your tests detected this mutant and failed. **Good tests!**

### MISSED (⚠️ Bad!)

```
MISSED: src/lib.rs:67:8: replaced && with ||
```

Your tests **didn't detect** this mutant. You need more tests!

### TIMEOUT

```
TIMEOUT: src/lib.rs:89:16: infinite loop
```

The mutant caused an infinite loop. This is caught (killed by timeout).

### UNVIABLE

```
UNVIABLE: src/lib.rs:123:4: type error
```

The mutant doesn't compile. Doesn't count toward score.

## Example: Improving Test Quality

### Original Code

```rust
pub fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

### Bad Test (low mutation score)

```rust
#[test]
fn test_factorial() {
    assert_eq!(factorial(0), 1);  // Only tests base case!
}
```

**Mutation Results**:

```
CAUGHT: replaced <= with <        ✅
MISSED: replaced 1 with 0          ❌
MISSED: replaced * with +          ❌
MISSED: replaced n - 1 with n      ❌
MISSED: replaced n - 1 with n - 2  ❌
──────────────────────────────────
Kill rate: 20% (1/5 mutants) ❌
```

### Good Test (high mutation score)

```rust
#[test]
fn test_factorial() {
    assert_eq!(factorial(0), 1);   // Base case: n = 0
    assert_eq!(factorial(1), 1);   // Base case: n = 1
    assert_eq!(factorial(2), 2);   // Recursive case
    assert_eq!(factorial(3), 6);   // Catches * mutants
    assert_eq!(factorial(4), 24);  // Catches n-1 mutants
    assert_eq!(factorial(5), 120); // More coverage
}
```

**Mutation Results**:

```
CAUGHT: replaced <= with <        ✅
CAUGHT: replaced 1 with 0          ✅
CAUGHT: replaced * with +          ✅
CAUGHT: replaced n - 1 with n      ✅
CAUGHT: replaced n - 1 with n - 2  ✅
──────────────────────────────────
Kill rate: 100% (5/5 mutants) ✅
```

## DECY Mutation Testing

### Parser Mutations

```rust
// Original
pub fn parse_type(&mut self) -> Result<HirType> {
    if self.peek() == "int" {
        self.advance();
        Ok(HirType::Int)
    } else {
        Err(anyhow!("Expected type"))
    }
}
```

**Mutants**:
- Replace `==` with `!=`
- Replace `Ok(HirType::Int)` with `Ok(HirType::Void)`
- Replace `Err(anyhow!(...))` with `Ok(HirType::Int)`

**Tests to catch them**:

```rust
#[test]
fn test_parse_int_type() {
    let mut parser = Parser::new("int");
    assert_eq!(parser.parse_type().unwrap(), HirType::Int);
}

#[test]
fn test_parse_invalid_type() {
    let mut parser = Parser::new("invalid");
    assert!(parser.parse_type().is_err());
}

#[test]
fn test_parse_not_void() {
    let mut parser = Parser::new("int");
    assert_ne!(parser.parse_type().unwrap(), HirType::Void);
}
```

### Ownership Mutations

```rust
// Original
pub fn is_owning(&self, var: &str) -> bool {
    self.ownership.get(var) == Some(&OwnershipPattern::Owning)
}
```

**Mutants**:
- Replace `==` with `!=`
- Replace `Some(&OwnershipPattern::Owning)` with `None`
- Replace `Some(&OwnershipPattern::Owning)` with `Some(&OwnershipPattern::Borrowed)`

**Tests to catch them**:

```rust
#[test]
fn test_is_owning_detects_owned() {
    let mut analysis = OwnershipAnalysis::new();
    analysis.set_ownership("p", OwnershipPattern::Owning);
    assert!(analysis.is_owning("p"));
}

#[test]
fn test_is_owning_rejects_borrowed() {
    let mut analysis = OwnershipAnalysis::new();
    analysis.set_ownership("p", OwnershipPattern::Borrowed);
    assert!(!analysis.is_owning("p"));
}

#[test]
fn test_is_owning_rejects_unknown() {
    let analysis = OwnershipAnalysis::new();
    assert!(!analysis.is_owning("unknown"));
}
```

### Codegen Mutations

```rust
// Original
pub fn generate_type(&self, ty: &HirType) -> String {
    match ty {
        HirType::Int => "i32".to_string(),
        HirType::Void => "()".to_string(),
        HirType::Pointer(inner) => format!("*mut {}", self.generate_type(inner)),
    }
}
```

**Mutants**:
- Replace `"i32"` with `"i64"`
- Replace `"*mut"` with `"*const"`
- Replace `"()"` with `""`

**Tests to catch them**:

```rust
#[test]
fn test_generate_int_type() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.generate_type(&HirType::Int), "i32");
}

#[test]
fn test_generate_void_type() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.generate_type(&HirType::Void), "()");
}

#[test]
fn test_generate_pointer_is_mutable() {
    let codegen = CodeGenerator::new();
    let ptr = HirType::Pointer(Box::new(HirType::Int));
    let result = codegen.generate_type(&ptr);
    assert!(result.contains("*mut"));
    assert!(!result.contains("*const"));
}
```

## Mutation Testing in CI/CD

Add to `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on:
  pull_request:
    branches: [main]

jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for diff

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation testing
        run: |
          cargo mutants --in-diff origin/main \
                        --timeout 120 \
                        --output mutants.json

      - name: Check mutation score
        run: |
          # Extract kill rate from output
          KILL_RATE=$(jq '.summary.caught / .summary.total * 100' mutants.json)
          if (( $(echo "$KILL_RATE < 90" | bc -l) )); then
            echo "Mutation score too low: $KILL_RATE% (target: ≥90%)"
            exit 1
          fi
          echo "Mutation score: $KILL_RATE% ✅"

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: mutation-results
          path: mutants.json
```

## Best Practices

### 1. Run on Diffs Only

```bash
# ❌ Slow: test entire codebase
cargo mutants

# ✅ Fast: test only changes
cargo mutants --in-diff origin/main
```

### 2. Use Timeouts

```bash
# ❌ Infinite loops hang forever
cargo mutants

# ✅ Kill slow tests
cargo mutants --timeout 120
```

### 3. Focus on Critical Code

```toml
# mutants.toml
[test]
# Only mutate core logic
include_globs = [
    "src/parser/*.rs",
    "src/ownership/*.rs",
    "src/codegen/*.rs",
]

# Skip less critical code
exclude_globs = [
    "src/cli/*.rs",
    "src/utils/*.rs",
]
```

### 4. Integrate with Coverage

```bash
# 1. Check coverage
cargo llvm-cov

# 2. If coverage ≥80%, check mutation score
cargo mutants --in-diff origin/main
```

Both must pass!

## Common Pitfalls

### Pitfall 1: Testing Implementation, Not Behavior

```rust
// ❌ BAD: Tests internal state
#[test]
fn test_uses_hashmap() {
    let analyzer = Analyzer::new();
    assert!(analyzer.data.is_empty());  // Testing internals!
}

// ✅ GOOD: Tests behavior
#[test]
fn test_analyzer_finds_variables() {
    let analyzer = Analyzer::new();
    analyzer.analyze("int x;");
    assert!(analyzer.has_variable("x"));  // Testing behavior
}
```

### Pitfall 2: Not Testing Edge Cases

```rust
// ❌ BAD: Only happy path
#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), 5);
}

// ✅ GOOD: Tests edge cases
#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), 5);
    assert_eq!(divide(0, 5), 0);
    assert_eq!(divide(7, 3), 2);  // Integer division
}

#[test]
#[should_panic]
fn test_divide_by_zero() {
    divide(10, 0);
}
```

### Pitfall 3: Weak Assertions

```rust
// ❌ BAD: Weak assertion
#[test]
fn test_parse() {
    let result = parse("int x;");
    assert!(result.is_ok());  // Doesn't check contents!
}

// ✅ GOOD: Strong assertions
#[test]
fn test_parse() {
    let ast = parse("int x;").unwrap();
    assert_eq!(ast.declarations().len(), 1);
    assert_eq!(ast.declarations()[0].name(), "x");
    assert_eq!(ast.declarations()[0].ty(), &HirType::Int);
}
```

## Summary

Mutation testing ensures test quality:

✅ **Measures effectiveness**: Are tests actually checking behavior?
✅ **Finds weak tests**: Tests that don't assert enough
✅ **Improves confidence**: High mutation score = good tests
✅ **Prevents regressions**: Good tests catch bugs early
✅ **Complements coverage**: 100% coverage + 90% mutation score = excellent quality

Target: **≥90% mutation kill rate**

## Next Steps

- [Parser Verification](../components/parser.md) - Apply mutation testing to parser
- [Test Coverage](../metrics/coverage.md) - Combining coverage and mutation metrics
