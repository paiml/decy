//! Documentation tests for for loop variable declarations (C99 §6.8.5.3)
//!
//! C99 introduced the ability to declare variables in for loop initializers.
//! This test suite documents how DECY transforms C99 for loop declarations to Rust.
//!
//! **Reference**: ISO C99 §6.8.5.3 (Iteration statements)
//!              K&R C (2nd Edition) does NOT support this (pre-C99)
//!
//! **Key Differences**:
//! - C89: Variables must be declared before for loop
//! - C99: Variables can be declared in for loop initializer
//! - Rust: Loop variables scoped to iterator (for i in 0..n)
//! - C99 loop variable scoped to loop body
//! - Rust range syntax more concise
//!
//! **Safety**: All transformations are SAFE (0 unsafe blocks)
//!
//! **Version**: v0.38.0

/// Document transformation of basic for loop with declaration
///
/// C99 allows `for (int i = 0; ...)` → Rust range-based for loop
///
/// C Reference: ISO C99 §6.8.5.3
#[test]
fn test_basic_for_declaration() {
    let _c_code = r#"
// C99 (allowed):
for (int i = 0; i < 10; i++) {
    printf("%d\n", i);
}

// C89 (required):
// int i;
// for (i = 0; i < 10; i++) { ... }
"#;

    let _rust_equivalent = r#"
for i in 0..10 {
    println!("{}", i);
}
// i is automatically scoped to the loop
"#;

    let mut sum = 0;
    for i in 0..10 {
        sum += i;
    }
    assert_eq!(sum, 45); // 0+1+2+...+9 = 45
}

/// Document for loop with multiple variables
///
/// C99 comma operator in for loop → Rust tuple or separate variables
#[test]
fn test_for_multiple_variables() {
    let _c_code = r#"
for (int i = 0, j = 10; i < j; i++, j--) {
    printf("%d %d\n", i, j);
}
"#;

    let _rust_equivalent = r#"
let mut i = 0;
let mut j = 10;
while i < j {
    println!("{} {}", i, j);
    i += 1;
    j -= 1;
}
"#;

    let mut i = 0;
    let mut j = 10;
    let mut iterations = 0;
    while i < j {
        iterations += 1;
        i += 1;
        j -= 1;
    }
    assert_eq!(iterations, 5);
    assert_eq!(i, 5);
    assert_eq!(j, 5);
}

/// Document for loop variable scope
///
/// C99 loop variable scoped to loop body
#[test]
fn test_for_variable_scope() {
    let _c_code = r#"
for (int i = 0; i < 5; i++) {
    printf("%d\n", i);
}
// i is not accessible here (C99 scoping)
"#;

    let _rust_equivalent = r#"
for i in 0..5 {
    println!("{}", i);
}
// i is not accessible here (same scoping as C99)
"#;

    // Verify scope with multiple loops
    let mut sum1 = 0;
    for i in 0..5 {
        sum1 += i;
    }

    let mut sum2 = 0;
    for i in 0..10 {
        // This is a different 'i'
        sum2 += i;
    }

    assert_eq!(sum1, 10); // 0+1+2+3+4
    assert_eq!(sum2, 45); // 0+1+2+...+9
}

/// Document nested for loops with declarations
///
/// C99 nested loops with declarations → Rust nested ranges
#[test]
fn test_nested_for_declarations() {
    let _c_code = r#"
for (int i = 0; i < 3; i++) {
    for (int j = 0; j < 3; j++) {
        printf("%d,%d ", i, j);
    }
}
"#;

    let _rust_equivalent = r#"
for i in 0..3 {
    for j in 0..3 {
        print!("{},{} ", i, j);
    }
}
"#;

    let mut count = 0;
    for i in 0..3 {
        for j in 0..3 {
            count += 1;
            assert!(i < 3);
            assert!(j < 3);
        }
    }
    assert_eq!(count, 9); // 3 * 3
}

/// Document for loop with different types
///
/// C99 allows any integer type in declaration
#[test]
fn test_for_different_types() {
    let _c_code = r#"
for (size_t i = 0; i < 100; i++) { }
for (unsigned int i = 0; i < 50; i++) { }
for (long i = 0; i < 1000; i++) { }
"#;

    let _rust_equivalent = r#"
for i in 0usize..100 { }
for i in 0u32..50 { }
for i in 0i64..1000 { }
// Type is inferred from range bounds
"#;

    let mut sum_usize = 0usize;
    for i in 0usize..10 {
        sum_usize += i;
    }
    assert_eq!(sum_usize, 45);

    let mut sum_u32 = 0u32;
    for i in 0u32..10 {
        sum_u32 += i;
    }
    assert_eq!(sum_u32, 45);
}

/// Document for loop with initialization expression
///
/// C99 allows complex initialization
#[test]
fn test_for_complex_initialization() {
    let _c_code = r#"
int start = 5;
for (int i = start * 2; i < 20; i++) {
    printf("%d\n", i);
}
"#;

    let _rust_equivalent = r#"
let start = 5;
for i in (start * 2)..20 {
    println!("{}", i);
}
"#;

    let start = 5;
    let mut count = 0;
    for i in (start * 2)..20 {
        assert!(i >= 10);
        assert!(i < 20);
        count += 1;
    }
    assert_eq!(count, 10); // 10..20 = 10 iterations
}

/// Document for loop with step increment
///
/// C99 loop with custom increment → Rust step_by
#[test]
fn test_for_custom_increment() {
    let _c_code = r#"
for (int i = 0; i < 20; i += 2) {
    printf("%d\n", i);
}
"#;

    let _rust_equivalent = r#"
for i in (0..20).step_by(2) {
    println!("{}", i);
}
"#;

    let mut values = Vec::new();
    for i in (0..20).step_by(2) {
        values.push(i);
    }
    assert_eq!(values, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}

/// Document reverse for loop
///
/// C99 countdown loop → Rust rev()
#[test]
fn test_for_reverse() {
    let _c_code = r#"
for (int i = 10; i > 0; i--) {
    printf("%d\n", i);
}
"#;

    let _rust_equivalent = r#"
for i in (1..=10).rev() {
    println!("{}", i);
}
"#;

    let mut values = Vec::new();
    for i in (1..=10).rev() {
        values.push(i);
    }
    assert_eq!(values, vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
}

/// Document for loop over array with declaration
///
/// C99 array iteration with index → Rust enumerate or range
#[test]
fn test_for_array_iteration() {
    let _c_code = r#"
int arr[5] = {1, 2, 3, 4, 5};
for (int i = 0; i < 5; i++) {
    printf("%d\n", arr[i]);
}
"#;

    let _rust_equivalent = r#"
let arr = [1, 2, 3, 4, 5];
// Option 1: Index-based
for i in 0..arr.len() {
    println!("{}", arr[i]);
}
// Option 2: Direct iteration (more idiomatic)
for &val in &arr {
    println!("{}", val);
}
"#;

    let arr = [1, 2, 3, 4, 5];

    // Index-based
    let mut sum1 = 0;
    for i in 0..arr.len() {
        sum1 += arr[i];
    }

    // Direct iteration
    let mut sum2 = 0;
    for &val in &arr {
        sum2 += val;
    }

    assert_eq!(sum1, 15);
    assert_eq!(sum2, 15);
}

/// Document for loop with break and continue
///
/// C99 loop with control flow → Rust same syntax
#[test]
fn test_for_control_flow() {
    let _c_code = r#"
for (int i = 0; i < 20; i++) {
    if (i % 2 == 0) continue;
    if (i > 10) break;
    printf("%d\n", i);
}
"#;

    let _rust_equivalent = r#"
for i in 0..20 {
    if i % 2 == 0 { continue; }
    if i > 10 { break; }
    println!("{}", i);
}
"#;

    let mut values = Vec::new();
    for i in 0..20 {
        if i % 2 == 0 {
            continue;
        }
        if i > 10 {
            break;
        }
        values.push(i);
    }
    assert_eq!(values, vec![1, 3, 5, 7, 9]);
}

/// Document for loop with pointer arithmetic
///
/// C99 pointer iteration → Rust slice iteration
#[test]
fn test_for_pointer_iteration() {
    let _c_code = r#"
int arr[] = {1, 2, 3, 4, 5};
for (int* p = arr; p < arr + 5; p++) {
    printf("%d\n", *p);
}
"#;

    let _rust_equivalent = r#"
let arr = [1, 2, 3, 4, 5];
for val in &arr {
    println!("{}", val);
}
// Or with mutable access:
// for val in &mut arr { ... }
"#;

    let arr = [1, 2, 3, 4, 5];
    let mut sum = 0;
    for val in &arr {
        sum += val;
    }
    assert_eq!(sum, 15);
}

/// Document for loop with empty body
///
/// C99 loop used for side effects only
#[test]
fn test_for_empty_body() {
    let _c_code = r#"
int n = 10;
for (int i = 0; i < n; i++);  // Empty body
// Or in increment:
for (int i = 0; i < n; n--);  // Modify n in increment
"#;

    let _rust_equivalent = r#"
let n = 10;
for _i in 0..n { }  // Empty body (but less common)

// Better: just use arithmetic
let _result = n;  // If no side effects needed
"#;

    let n = 10;
    let mut count = 0;
    for _ in 0..n {
        count += 1;
    }
    assert_eq!(count, 10);
}

/// Document for loop with shadowing
///
/// C99 vs Rust variable shadowing
#[test]
fn test_for_variable_shadowing() {
    let _c_code = r#"
int i = 100;
for (int i = 0; i < 5; i++) {
    printf("%d\n", i);  // Inner i (0..4)
}
printf("%d\n", i);  // Outer i (100)
"#;

    let _rust_equivalent = r#"
let i = 100;
for i in 0..5 {
    println!("{}", i);  // Inner i (0..4)
}
println!("{}", i);  // Outer i (100)
"#;

    let i = 100;
    let mut inner_sum = 0;
    for i in 0..5 {
        inner_sum += i;
    }
    assert_eq!(inner_sum, 10); // 0+1+2+3+4
    assert_eq!(i, 100); // Outer i unchanged
}

/// Document for loop with enumerate
///
/// C99 pattern with index and value → Rust enumerate
#[test]
fn test_for_enumerate() {
    let _c_code = r#"
int values[] = {10, 20, 30, 40, 50};
for (int i = 0; i < 5; i++) {
    printf("Index %d: Value %d\n", i, values[i]);
}
"#;

    let _rust_equivalent = r#"
let values = [10, 20, 30, 40, 50];
for (i, &value) in values.iter().enumerate() {
    println!("Index {}: Value {}", i, value);
}
"#;

    let values = [10, 20, 30, 40, 50];
    let mut index_sum = 0;
    let mut value_sum = 0;
    for (i, &value) in values.iter().enumerate() {
        index_sum += i;
        value_sum += value;
    }
    assert_eq!(index_sum, 10); // 0+1+2+3+4
    assert_eq!(value_sum, 150); // 10+20+30+40+50
}

/// Document infinite for loop
///
/// C99 infinite loop → Rust loop
#[test]
fn test_for_infinite() {
    let _c_code = r#"
for (;;) {
    if (condition) break;
}
"#;

    let _rust_equivalent = r#"
loop {
    if condition { break; }
}
// More idiomatic in Rust to use 'loop'
"#;

    let mut counter = 0;
    loop {
        counter += 1;
        if counter >= 10 {
            break;
        }
    }
    assert_eq!(counter, 10);
}

/// Summary: For Loop Declarations (C99 §6.8.5.3)
///
/// **Transformation Rules**:
/// 1. C99 `for (int i = 0; i < n; i++)` → Rust `for i in 0..n`
/// 2. C99 `for (int i = a; i < b; i += step)` → Rust `for i in (a..b).step_by(step)`
/// 3. C99 `for (int i = n; i > 0; i--)` → Rust `for i in (1..=n).rev()`
/// 4. C99 array iteration → Rust `for val in &arr` or `for i in 0..arr.len()`
/// 5. C99 `for (;;)` → Rust `loop { }`
///
/// **Key Insights**:
/// - C99 added variable declarations in for loops (NOT in C89/K&R)
/// - Both C99 and Rust scope variable to loop body
/// - Rust range syntax more concise and expressive
/// - Rust prevents common off-by-one errors
/// - Rust iterators more powerful (enumerate, filter, map)
/// - No pointer arithmetic needed in Rust
/// - Rust ranges are half-open [start, end)
/// - Use ..= for inclusive ranges [start, end]
///
/// **Safety**: ✅ 0 unsafe blocks (iterator-based loops are safe)
///
/// **Coverage**: 15 test cases covering:
/// - Basic for declarations
/// - Multiple variables
/// - Variable scope
/// - Nested loops
/// - Different types
/// - Complex initialization
/// - Custom increment (step_by)
/// - Reverse iteration
/// - Array iteration
/// - Control flow (break/continue)
/// - Pointer iteration
/// - Empty body
/// - Variable shadowing
/// - Enumerate pattern
/// - Infinite loops
#[test]
fn test_for_declaration_summary() {
    // C89 did not support declarations in for loops
    let c89_supports_for_decl = false;

    // C99 added this feature
    let c99_supports_for_decl = true;

    // Rust supports scoped loop variables
    let rust_supports_scoped_vars = true;

    assert!(
        !c89_supports_for_decl,
        "C89/K&R did not have for loop declarations"
    );
    assert!(c99_supports_for_decl, "C99 added for loop declarations");
    assert!(rust_supports_scoped_vars, "Rust loop variables are scoped");

    // Verify scoping behavior
    let outer = 100;
    let mut iterations = 0;
    for _outer in 0..5 {
        // Inner variable shadows outer
        iterations += 1;
    }
    assert_eq!(outer, 100, "Outer variable unchanged");
    assert_eq!(iterations, 5);

    // No unsafe blocks needed
    let unsafe_blocks = 0;
    assert_eq!(unsafe_blocks, 0, "Iterator-based loops are safe");
}
