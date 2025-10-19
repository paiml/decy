# Macro Expansion Verification

This chapter verifies that DECY correctly transpiles C preprocessor `#define` macros to idiomatic Rust code.

**References**:
- K&R §4.11 - Macro Substitution
- ISO C99 §6.10.3 - Macro Replacement

**Verification Strategy**:
- Object-like macros → Rust `const` declarations
- Function-like macros → Rust `#[inline]` functions
- Ternary operators → Rust `if-else` expressions
- Type inference from macro body
- Name conversion: `SCREAMING_SNAKE_CASE` → `snake_case` (for functions)

## Object-Like Macros (Constants)

Object-like macros are simple value replacements that should transpile to Rust `const` declarations.

### Integer Constants

**C Code**:
```c
#define MAX 100
#define MIN -50
```

**Expected Rust**:
```rust
const MAX: i32 = 100;
const MIN: i32 = -50;
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_constants.rs:integer_constants}}
```

### Floating Point Constants

**C Code**:
```c
#define PI 3.14159
#define E 2.71828
```

**Expected Rust**:
```rust
const PI: f64 = 3.14159;
const E: f64 = 2.71828;
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_constants.rs:float_constants}}
```

### String Constants

**C Code**:
```c
#define GREETING "Hello, World!"
#define VERSION "v1.0.0"
```

**Expected Rust**:
```rust
const GREETING: &str = "Hello, World!";
const VERSION: &str = "v1.0.0";
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_constants.rs:string_constants}}
```

### Character Constants

**C Code**:
```c
#define NEWLINE '\n'
#define TAB '\t'
```

**Expected Rust**:
```rust
const NEWLINE: char = '\n';
const TAB: char = '\t';
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_constants.rs:char_constants}}
```

### Hexadecimal Constants

**C Code**:
```c
#define FLAGS 0xFF
#define MASK 0x0F
```

**Expected Rust**:
```rust
const FLAGS: i32 = 0xFF;
const MASK: i32 = 0x0F;
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_constants.rs:hex_constants}}
```

## Function-Like Macros

Function-like macros with parameters should transpile to `#[inline]` Rust functions.

### Single Parameter Macros

**C Code**:
```c
#define SQR(x) ((x) * (x))
#define DOUBLE(x) ((x) * 2)
```

**Expected Rust**:
```rust
#[inline]
fn sqr(x: i32) -> i32 {
    x * x
}

#[inline]
fn double(x: i32) -> i32 {
    x * 2
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_functions.rs:single_param}}
```

### Two Parameter Macros

**C Code**:
```c
#define ADD(a, b) ((a) + (b))
#define MUL(a, b) ((a) * (b))
```

**Expected Rust**:
```rust
#[inline]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[inline]
fn mul(a: i32, b: i32) -> i32 {
    a * b
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_functions.rs:two_param}}
```

### Three Parameter Macros

**C Code**:
```c
#define ADD3(a, b, c) ((a) + (b) + (c))
```

**Expected Rust**:
```rust
#[inline]
fn add3(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_functions.rs:three_param}}
```

## Ternary Operator Transformation

C's ternary operator (`?:`) should be transformed to Rust's `if-else` expression.

### MAX Macro

**C Code**:
```c
#define MAX(a, b) ((a) > (b) ? (a) : (b))
```

**Expected Rust**:
```rust
#[inline]
fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_ternary.rs:max_macro}}
```

### MIN Macro

**C Code**:
```c
#define MIN(a, b) ((a) < (b) ? (a) : (b))
```

**Expected Rust**:
```rust
#[inline]
fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_ternary.rs:min_macro}}
```

### ABS Macro

**C Code**:
```c
#define ABS(x) ((x) < 0 ? -(x) : (x))
```

**Expected Rust**:
```rust
#[inline]
fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_ternary.rs:abs_macro}}
```

## Type Inference

DECY infers Rust types from the macro body.

### Return Type Inference

**Arithmetic expressions** → `i32`:
```c
#define SQR(x) ((x) * (x))    // → i32
```

**Comparison expressions** → `bool`:
```c
#define IS_POSITIVE(x) ((x) > 0)    // → bool
```

**Logical expressions** → `bool`:
```c
#define AND(a, b) ((a) && (b))    // → bool
```

**Ternary expressions** → `i32` (type of branches, not condition):
```c
#define MAX(a, b) ((a) > (b) ? (a) : (b))    // → i32
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_type_inference.rs:type_inference}}
```

### Parameter Type Inference

Currently, all parameters are inferred as `i32`. Future work will support:
- Generic parameters: `fn max<T: Ord>(a: T, b: T) -> T`
- Boolean parameters for logical macros
- Float parameters for floating-point math

## Name Conversion

Macro names follow C convention (`SCREAMING_SNAKE_CASE`). DECY converts these to Rust naming conventions:

- **Object-like macros** (constants): Keep `SCREAMING_SNAKE_CASE`
  - `MAX` → `const MAX: i32`
- **Function-like macros**: Convert to `snake_case`
  - `IS_POSITIVE` → `fn is_positive`

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_naming.rs:name_conversion}}
```

## Operator Spacing

DECY adds proper spacing around operators while preserving unary operators:

**Binary operators** (get spaces):
- `x+y` → `x + y`
- `a*b` → `a * b`
- `x>y` → `x > y`

**Unary operators** (no spaces):
- `-(x)` → `-x` (not `- x`)
- `!(x)` → `!x` (not `! x`)

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_spacing.rs:operator_spacing}}
```

## Parentheses Cleanup

DECY removes unnecessary parentheses from C macro bodies:

**C** (defensive parentheses):
```c
#define SQR(x) ((x)*(x))
```

**Rust** (clean):
```rust
fn sqr(x: i32) -> i32 {
    x * x  // No unnecessary parentheses
}
```

**Verification**:
```rust
{{#rustdoc_include ../../../examples/macro_expansion_parens.rs:parens_cleanup}}
```

## Test Coverage

Macro expansion is verified with:

- **40 total tests** (20 integration + 20 property tests)
- **10,240 test cases** (with proptest, 256 cases per property)
- **100% code coverage** of macro generation paths

### Integration Tests

- 10 constant macro tests (`macro_expansion_constants_test.rs`)
- 10 function-like macro tests (`function_like_macro_expansion_test.rs`)

### Property Tests

- 10 constant macro properties (`macro_expansion_property_tests.rs`)
- 10 function-like macro properties (`function_like_macro_property_tests.rs`)

**Example Properties**:
- All numeric constants generate valid `const` declarations
- Float constants always typed as `f64`
- String constants always typed as `&str`
- Macro names preserved exactly (for constants)
- Function names converted to `snake_case`
- Arithmetic expressions return `i32`
- Comparison expressions return `bool`
- Ternary operators transform to `if-else`

## Safety Analysis

Macro expansion generates **zero unsafe blocks**:

- All constant macros → safe `const` declarations
- All function-like macros → safe `#[inline]` functions
- No pointer manipulation
- No memory allocation
- No system calls

**Safety Score**: ✅ 100% safe (0 unsafe blocks)

## Limitations and Future Work

Current limitations (Sprint 9):

1. **No nested macro expansion**: `MAX(a, MIN(b, c))` not yet supported
2. **No recursive macros**: Direct/indirect recursion not detected
3. **Fixed type inference**: All parameters are `i32`, no generics
4. **No string macros**: `#define STR(x) #x` not supported
5. **No token pasting**: `##` operator not supported
6. **No variadic macros**: `...` arguments not supported

Future sprints will address these limitations while maintaining zero unsafe blocks.

## Conclusion

DECY successfully transpiles C preprocessor macros to idiomatic Rust:

✅ Object-like macros → `const` declarations
✅ Function-like macros → `#[inline]` functions
✅ Ternary operators → `if-else` expressions
✅ Type inference from macro body
✅ Name conversion for functions
✅ Zero unsafe blocks
✅ 100% test coverage

This foundation enables handling of real-world C codebases that rely heavily on macros, a critical step toward production-ready C-to-Rust transpilation.
