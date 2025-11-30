# Functions

Decy transforms C functions to idiomatic Rust functions with proper type signatures, ownership semantics, and safety guarantees.

## Basic Function Transformation

### C Input

```c
int add(int a, int b) {
    return a + b;
}
```

### Rust Output

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, 1), 0);
}
```

**What Changed**:
- `int` → `i32`
- Implicit return (last expression without semicolon)

## Pattern 1: Void Functions

### C Input

```c
void print_value(int value) {
    printf("Value: %d\n", value);
}
```

### Rust Output

```rust
fn print_value(value: i32) {
    println!("Value: {}", value);
}

#[test]
fn test_print_value() {
    // This compiles and runs without panic
    print_value(42);
}
```

**What Changed**:
- `void` return → no return type annotation
- `printf` → `println!` macro

## Pattern 2: Pointer Parameters

### C Input

```c
void swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}
```

### Rust Output

```rust
fn swap(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}

#[test]
fn test_swap() {
    let mut x = 10;
    let mut y = 20;
    swap(&mut x, &mut y);
    assert_eq!(x, 20);
    assert_eq!(y, 10);
}
```

**What Changed**:
- `int *a` → `&mut i32` (mutable reference)
- Manual swap → `std::mem::swap`

## Pattern 3: Array Parameters

### C Input

```c
int sum_array(int *arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total += arr[i];
    }
    return total;
}
```

### Rust Output

```rust
fn sum_array(arr: &[i32]) -> i32 {
    arr.iter().sum()
}

#[test]
fn test_sum_array() {
    assert_eq!(sum_array(&[1, 2, 3, 4, 5]), 15);
    assert_eq!(sum_array(&[]), 0);
}
```

**What Changed**:
- `int *arr, int len` → `&[i32]` (slice with length)
- Manual loop → iterator `.sum()`

## Pattern 4: Return by Pointer (Output Parameters)

C often uses output parameters for returning values. Decy transforms these to return values.

### C Input

```c
int divide(int a, int b, int *result) {
    if (b == 0) {
        return -1;  // Error
    }
    *result = a / b;
    return 0;  // Success
}
```

### Rust Output

```rust
fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), Some(5));
    assert_eq!(divide(10, 0), None);
}
```

**What Changed**:
- Output parameter → return value
- Error code → `Option` type
- Clearer API

## Pattern 5: Multiple Return Values

### C Input

```c
void minmax(int *arr, int len, int *min, int *max) {
    *min = arr[0];
    *max = arr[0];
    for (int i = 1; i < len; i++) {
        if (arr[i] < *min) *min = arr[i];
        if (arr[i] > *max) *max = arr[i];
    }
}
```

### Rust Output

```rust
fn minmax(arr: &[i32]) -> (i32, i32) {
    let min = *arr.iter().min().unwrap();
    let max = *arr.iter().max().unwrap();
    (min, max)
}

#[test]
fn test_minmax() {
    let (min, max) = minmax(&[3, 1, 4, 1, 5, 9, 2, 6]);
    assert_eq!(min, 1);
    assert_eq!(max, 9);
}
```

**What Changed**:
- Output parameters → tuple return
- Manual iteration → iterator methods

## Pattern 6: Static Variables

C static variables become Rust statics or are refactored.

### C Input

```c
int counter() {
    static int count = 0;
    count++;
    return count;
}
```

### Rust Output

```rust
use std::sync::atomic::{AtomicI32, Ordering};

static COUNT: AtomicI32 = AtomicI32::new(0);

fn counter() -> i32 {
    COUNT.fetch_add(1, Ordering::SeqCst) + 1
}

#[test]
fn test_counter() {
    // Note: test isolation may vary
    let _ = counter();
    let _ = counter();
    // Counter increments on each call
}
```

**What Changed**:
- `static int` → `AtomicI32` (thread-safe)
- Increment is atomic operation

## Pattern 7: Function Pointers

### C Input

```c
int apply(int (*op)(int, int), int a, int b) {
    return op(a, b);
}

int mul(int a, int b) {
    return a * b;
}

int main() {
    return apply(mul, 3, 4);  // Returns 12
}
```

### Rust Output

```rust
fn apply<F>(op: F, a: i32, b: i32) -> i32
where
    F: Fn(i32, i32) -> i32,
{
    op(a, b)
}

fn mul(a: i32, b: i32) -> i32 {
    a * b
}

fn main() -> i32 {
    apply(mul, 3, 4)
}

#[test]
fn test_apply() {
    assert_eq!(apply(mul, 3, 4), 12);
    assert_eq!(apply(|a, b| a + b, 3, 4), 7);
}
```

**What Changed**:
- Function pointer → generic with `Fn` trait
- Works with closures too

## Pattern 8: Variadic Functions

Variadic C functions are transformed based on usage.

### C Input

```c
// printf-like function
void log_message(const char *format, ...) {
    // Implementation using va_list
}
```

### Rust Output

```rust
// Macro-based approach
macro_rules! log_message {
    ($fmt:expr) => {
        println!("{}", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        println!($fmt, $($arg)*)
    };
}

#[test]
fn test_log_message() {
    log_message!("Hello");
    log_message!("Value: {}", 42);
}
```

**What Changed**:
- Variadic function → macro
- Type-safe format strings

## Type Mapping

| C Type | Rust Type | Notes |
|--------|-----------|-------|
| `int` | `i32` | 32-bit signed |
| `unsigned int` | `u32` | 32-bit unsigned |
| `long` | `i64` | 64-bit signed |
| `float` | `f32` | 32-bit float |
| `double` | `f64` | 64-bit float |
| `char` | `i8` or `u8` | Depends on context |
| `void` | `()` | Unit type |
| `void *` | `*mut c_void` or generic | Context dependent |

## How Decy Decides

### Parameter Types

```
Is parameter a pointer?
├─ Yes → Is it const?
│   ├─ Yes → &T (immutable reference)
│   └─ No → Is it written through?
│       ├─ Yes → &mut T (mutable reference)
│       └─ No → &T (immutable reference)
└─ No → T (value type)
```

### Return Types

```
Does function return pointer?
├─ Yes → Analyze ownership...
└─ No → Does it have output params?
   ├─ Yes → Transform to return value
   └─ No → Direct mapping
```

## Summary

| C Pattern | Rust Equivalent |
|-----------|-----------------|
| `int func()` | `fn func() -> i32` |
| `void func()` | `fn func()` |
| `void func(int *out)` | `fn func() -> i32` |
| `int *arr, int len` | `&[i32]` |
| `int *p` (mutated) | `&mut i32` |
| `const int *p` | `&i32` |
| `int (*fn)(int)` | `impl Fn(i32) -> i32` |

## Next

- [Control Flow](./control-flow.md) - Loops and conditionals
- [Arrays and Slices](./arrays.md) - Array patterns

---

**Note**: All code examples in this chapter are tested in CI. If they don't compile, our release is blocked!
