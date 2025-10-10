# Property Testing

Property testing verifies **invariants** that should hold for ALL inputs, not just specific test cases.

## Philosophy

Traditional unit tests check specific examples:

```rust
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);  // Only tests ONE input
}
```

Property tests check invariants across THOUSANDS of random inputs:

```rust
proptest! {
    #[test]
    fn prop_add_commutative(a: i32, b: i32) {
        // Property: addition is commutative for ALL inputs
        prop_assert_eq!(add(a, b), add(b, a));
    }
}
```

This runs 1000+ times with random values!

## proptest Framework

DECY uses the `proptest` crate for property-based testing.

### Installation

Add to `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1.4"
```

### Basic Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_reverse_twice_is_identity(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let reversed_twice = vec.iter().rev().rev().collect::<Vec<_>>();
        prop_assert_eq!(vec, reversed_twice);
    }
}
```

## Properties in DECY

### Property 1: Deterministic Transpilation

Same C code → same Rust output (always):

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        func_name in "[a-z]{3,10}",
        param1 in "[a-z]{1,5}",
        param2 in "[a-z]{1,5}",
    ) {
        let c_code = format!(
            "int {}(int {}, int {}) {{ return {} + {}; }}",
            func_name, param1, param2, param1, param2
        );

        let output1 = transpile(&c_code).unwrap();
        let output2 = transpile(&c_code).unwrap();

        // Property: Transpilation is deterministic
        prop_assert_eq!(output1, output2);
    }
}
```

### Property 2: Function Names Preserved

```rust
proptest! {
    #[test]
    fn prop_function_names_preserved(func_name in "[a-z_][a-z0-9_]{2,20}") {
        let c_code = format!("int {}() {{ return 0; }}", func_name);
        let rust_code = transpile(&c_code).unwrap();

        // Property: Function name appears in output
        prop_assert!(
            rust_code.contains(&format!("fn {}", func_name)),
            "Expected 'fn {}' in output",
            func_name
        );
    }
}
```

### Property 3: Valid Rust Output

```rust
proptest! {
    #[test]
    fn prop_output_is_valid_rust(c_code in valid_c_function()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Output compiles as valid Rust
        prop_assert!(
            compile_rust(&rust_code).is_ok(),
            "Generated Rust must compile:\n{}",
            rust_code
        );
    }
}
```

### Property 4: Type Safety

```rust
proptest! {
    #[test]
    fn prop_int_maps_to_i32(func_name in "[a-z]{3,10}") {
        let c_code = format!("int {}() {{ return 0; }}", func_name);
        let rust_code = transpile(&c_code).unwrap();

        // Property: C 'int' → Rust 'i32'
        prop_assert!(
            rust_code.contains("i32"),
            "int functions should use i32"
        );
    }

    #[test]
    fn prop_pointers_map_to_references(ty in c_type_generator()) {
        let c_code = format!("{}* func() {{ return 0; }}", ty);
        let rust_code = transpile(&c_code).unwrap();

        // Property: Pointers become references or raw pointers
        prop_assert!(
            rust_code.contains("&") || rust_code.contains("*"),
            "Pointers should map to & or *"
        );
    }
}
```

### Property 5: No Memory Leaks

```rust
proptest! {
    #[test]
    fn prop_malloc_has_drop(c_code in c_code_with_malloc()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: malloc → Box (automatic drop)
        if c_code.contains("malloc") {
            prop_assert!(
                rust_code.contains("Box::new") || rust_code.contains("Box<"),
                "malloc should become Box for automatic cleanup"
            );
        }
    }
}
```

### Property 6: Borrow Checker Compliance

```rust
proptest! {
    #[test]
    fn prop_no_multiple_mutable_borrows(c_code in safe_c_code()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Generated code respects borrow checker
        prop_assert!(
            compile_with_borrowck(&rust_code).is_ok(),
            "Generated code must pass borrow checker"
        );
    }
}
```

## Custom Generators

Generate valid C code for testing:

```rust
use proptest::prelude::*;

/// Generate valid C function names
fn c_identifier() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]{2,20}"
}

/// Generate C type names
fn c_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("int".to_string()),
        Just("char".to_string()),
        Just("void".to_string()),
        Just("float".to_string()),
        Just("double".to_string()),
    ]
}

/// Generate C function with parameters
fn c_function() -> impl Strategy<Value = String> {
    (c_type(), c_identifier(), prop::collection::vec(c_identifier(), 0..5))
        .prop_map(|(ret_type, name, params)| {
            if params.is_empty() {
                format!("{} {}() {{ return 0; }}", ret_type, name)
            } else {
                let param_list = params
                    .iter()
                    .map(|p| format!("int {}", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {}({}) {{ return 0; }}", ret_type, name, param_list)
            }
        })
}

proptest! {
    #[test]
    fn prop_test_with_custom_generator(c_code in c_function()) {
        let result = transpile(&c_code);
        prop_assert!(result.is_ok());
    }
}
```

## Shrinking

When a property fails, proptest **shrinks** the input to find the minimal failing case:

```rust
proptest! {
    #[test]
    fn prop_example_that_fails(x in 0..1000i32) {
        // This will fail for x >= 100
        prop_assert!(x < 100);
    }
}
```

Output:

```
Test failed for input: 999
Shrinking to minimal case...
Minimal failing input: 100
```

Proptest automatically finds the SMALLEST input that causes failure!

## Stateful Testing

Test state machines with property tests:

```rust
#[derive(Debug, Clone)]
enum Action {
    Allocate(usize),
    Free(usize),
    Use(usize),
}

proptest! {
    #[test]
    fn prop_memory_state_machine(actions in prop::collection::vec(action_generator(), 0..50)) {
        let mut state = MemoryState::new();

        for action in actions {
            match action {
                Action::Allocate(id) => state.allocate(id),
                Action::Free(id) => state.free(id),
                Action::Use(id) => {
                    // Property: Can only use allocated memory
                    if !state.is_allocated(id) {
                        prop_assert!(state.use_memory(id).is_err());
                    }
                }
            }
        }

        // Property: No memory leaks at end
        prop_assert!(state.all_freed());
    }
}
```

## Regression Testing

Save failing cases to prevent regressions:

```rust
proptest! {
    #[test]
    fn prop_transpile_never_panics(c_code in "\\PC*") {
        // Should never panic, even with garbage input
        let _ = transpile(&c_code);
    }
}
```

When a failure is found, proptest saves it to `proptest-regressions/`:

```
proptest-regressions/
└── lib.proptest-regressions
    └── prop_transpile_never_panics
        └── cc 01234567 // Saved failing input
```

## Configuration

Customize proptest behavior:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,           // Run 10,000 times (default: 256)
        max_shrink_iters: 10000, // Shrink more aggressively
        timeout: 5000,          // 5 second timeout per test
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_intensive_test(input in any::<Vec<i32>>()) {
        // This will run 10,000 times
        prop_assert!(verify_invariant(&input));
    }
}
```

## DECY-Specific Properties

### Parser Properties

```rust
proptest! {
    #[test]
    fn prop_parser_never_panics(input in "\\PC*") {
        // Property: Parser should never panic
        let _ = CParser::new().unwrap().parse(&input);
    }

    #[test]
    fn prop_parse_print_roundtrip(c_code in valid_c_code()) {
        let ast = parse(&c_code).unwrap();
        let printed = ast.to_string();
        let reparsed = parse(&printed).unwrap();

        // Property: Parse → print → parse = identity
        prop_assert_eq!(ast, reparsed);
    }
}
```

### HIR Properties

```rust
proptest! {
    #[test]
    fn prop_hir_preserves_semantics(c_code in valid_c_code()) {
        let ast = parse(&c_code).unwrap();
        let hir = lower_to_hir(&ast).unwrap();

        // Property: HIR has same number of functions
        prop_assert_eq!(ast.functions().len(), hir.functions().len());

        // Property: Function names preserved
        for (ast_func, hir_func) in ast.functions().iter().zip(hir.functions()) {
            prop_assert_eq!(ast_func.name(), hir_func.name());
        }
    }
}
```

### Ownership Properties

```rust
proptest! {
    #[test]
    fn prop_malloc_becomes_owning(size in 1..1024usize) {
        let c_code = format!("int* p = malloc({});", size);
        let ownership = infer_ownership(&c_code).unwrap();

        // Property: malloc is always owning
        prop_assert_eq!(ownership.pattern, OwnershipPattern::Owning);
    }

    #[test]
    fn prop_parameter_pointers_borrowed(param_name in c_identifier()) {
        let c_code = format!("void func(int* {}) {{}}", param_name);
        let ownership = infer_ownership(&c_code).unwrap();

        // Property: Function parameters are borrowed
        prop_assert_eq!(ownership.pattern, OwnershipPattern::Borrowed);
    }
}
```

### Codegen Properties

```rust
proptest! {
    #[test]
    fn prop_generated_code_compiles(c_code in valid_c_function()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Generated Rust always compiles
        prop_assert!(
            compile_rust(&rust_code).is_ok(),
            "Generated code must compile:\n{}",
            rust_code
        );
    }

    #[test]
    fn prop_no_unsafe_for_safe_c(c_code in memory_safe_c()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Safe C → safe Rust (no unsafe blocks)
        prop_assert!(
            !rust_code.contains("unsafe"),
            "Safe C should not generate unsafe Rust"
        );
    }

    #[test]
    fn prop_clippy_clean(c_code in valid_c_function()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Generated code passes clippy
        prop_assert!(
            clippy_check(&rust_code).is_ok(),
            "Generated code must pass clippy"
        );
    }
}
```

## Integration with Coverage

Property tests contribute to coverage:

```bash
cargo llvm-cov --lcov --output-path coverage.lcov
```

Each property test execution increases coverage by testing different code paths!

## Best Practices

### 1. Test Invariants, Not Implementations

```rust
// ❌ BAD: Testing implementation details
proptest! {
    #[test]
    fn prop_uses_specific_algorithm(input in any::<Vec<i32>>()) {
        let output = sort(&input);
        // Don't test HOW it sorts
    }
}

// ✅ GOOD: Testing properties
proptest! {
    #[test]
    fn prop_sort_is_sorted(input in any::<Vec<i32>>()) {
        let output = sort(&input);
        // Test WHAT it achieves
        for i in 0..output.len() - 1 {
            prop_assert!(output[i] <= output[i + 1]);
        }
    }

    #[test]
    fn prop_sort_preserves_elements(input in any::<Vec<i32>>()) {
        let output = sort(&input);
        // Same elements, different order
        prop_assert_eq!(input.len(), output.len());
        for &x in &input {
            prop_assert!(output.contains(&x));
        }
    }
}
```

### 2. Keep Properties Simple

```rust
// ❌ BAD: Complex property hard to understand
proptest! {
    #[test]
    fn prop_complex(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
        prop_assert!(complex_condition(a, b, c) == other_complex_condition(c, b, a));
    }
}

// ✅ GOOD: Simple, obvious properties
proptest! {
    #[test]
    fn prop_add_commutative(a in any::<i32>(), b in any::<i32>()) {
        prop_assert_eq!(add(a, b), add(b, a));
    }

    #[test]
    fn prop_add_associative(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
        prop_assert_eq!(add(add(a, b), c), add(a, add(b, c)));
    }
}
```

### 3. Use Appropriate Generators

```rust
// ❌ BAD: Too broad (will generate invalid code)
proptest! {
    #[test]
    fn prop_bad_generator(input in ".*") {
        let _ = parse(&input);  // Mostly fails with garbage
    }
}

// ✅ GOOD: Constrained to valid inputs
proptest! {
    #[test]
    fn prop_good_generator(input in valid_c_function()) {
        let ast = parse(&input).unwrap();  // Always valid C
        prop_assert!(ast.functions().len() > 0);
    }
}
```

## Summary

Property testing in DECY verifies:

✅ **Determinism**: Same input → same output
✅ **Correctness**: Invariants hold for all inputs
✅ **Completeness**: Generated Rust compiles
✅ **Safety**: Borrow checker passes
✅ **Quality**: Clippy warnings zero
✅ **Robustness**: No panics on invalid input

Property tests run **thousands of times** with random inputs to ensure quality!

## Next Steps

- [Mutation Testing](./mutation-testing.md) - Verify test quality
- [Parser Verification](../components/parser.md) - Parser property tests
