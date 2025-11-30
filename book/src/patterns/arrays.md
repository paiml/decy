# Arrays and Slices

Decy transforms C arrays to safe Rust types with bounds checking. Array parameters become slices, and dynamic arrays become `Vec<T>`.

## The Problem

C arrays are problematic:
- No bounds checking (buffer overflows)
- Arrays decay to pointers when passed to functions
- No size information at runtime
- Manual memory management for dynamic arrays

Rust provides safe alternatives:
- **Fixed arrays**: `[T; N]` with compile-time size
- **Slices**: `&[T]` with runtime bounds checking
- **Vectors**: `Vec<T>` for dynamic sizing

## Pattern 1: Fixed-Size Arrays

### C Input

```c
int sum_array() {
    int arr[5] = {1, 2, 3, 4, 5};
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum += arr[i];
    }
    return sum;
}
```

### Rust Output

```rust
fn sum_array() -> i32 {
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let mut sum = 0;
    for i in 0..5 {
        sum += arr[i];
    }
    sum
}

#[test]
fn test_sum_array() {
    assert_eq!(sum_array(), 15);
}
```

**What Changed**:
- `int arr[5]` → `[i32; 5]` (fixed-size array)
- Loop bounds preserved for safety

## Pattern 2: Array Parameters to Slices

When C functions take array parameters, Decy transforms them to slice references.

### C Input

```c
int sum(int *arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total += arr[i];
    }
    return total;
}
```

### Rust Output

```rust
fn sum(arr: &[i32]) -> i32 {
    let mut total = 0;
    for i in 0..arr.len() {
        total += arr[i];
    }
    total
}

#[test]
fn test_sum() {
    assert_eq!(sum(&[1, 2, 3, 4, 5]), 15);
    assert_eq!(sum(&[]), 0);
}
```

**What Changed**:
- `int *arr, int len` → `&[i32]` (slice carries length)
- `len` parameter eliminated (slice knows its length)
- Bounds checking automatic

## Pattern 3: Mutable Array Parameters

### C Input

```c
void zero_array(int *arr, int len) {
    for (int i = 0; i < len; i++) {
        arr[i] = 0;
    }
}
```

### Rust Output

```rust
fn zero_array(arr: &mut [i32]) {
    for i in 0..arr.len() {
        arr[i] = 0;
    }
}

#[test]
fn test_zero_array() {
    let mut arr = [1, 2, 3, 4, 5];
    zero_array(&mut arr);
    assert_eq!(arr, [0, 0, 0, 0, 0]);
}
```

**What Changed**:
- `int *arr` (mutated) → `&mut [i32]` (mutable slice)
- Mutations tracked by borrow checker

## Pattern 4: Pointer Arithmetic to Safe Indexing

C pointer arithmetic is converted to safe slice indexing.

### C Input

```c
int get_third(int *arr) {
    return *(arr + 2);  // Pointer arithmetic
}

void iterate_ptr(int *arr, int len) {
    int *end = arr + len;
    while (arr < end) {
        *arr = 0;
        arr++;
    }
}
```

### Rust Output

```rust
fn get_third(arr: &[i32]) -> i32 {
    arr[2]  // Safe indexing
}

fn iterate_ptr(arr: &mut [i32]) {
    for item in arr.iter_mut() {
        *item = 0;
    }
}

#[test]
fn test_pointer_arithmetic() {
    let arr = [10, 20, 30, 40, 50];
    assert_eq!(get_third(&arr), 30);

    let mut arr2 = [1, 2, 3];
    iterate_ptr(&mut arr2);
    assert_eq!(arr2, [0, 0, 0]);
}
```

**What Changed**:
- `*(arr + 2)` → `arr[2]` (bounds-checked)
- Pointer iteration → iterator pattern

## Pattern 5: Dynamic Arrays (malloc)

When C uses `malloc` for arrays, Decy transforms to `Vec<T>`.

### C Input

```c
int* create_array(int size) {
    int *arr = malloc(size * sizeof(int));
    for (int i = 0; i < size; i++) {
        arr[i] = i * 2;
    }
    return arr;
}

void use_array() {
    int *arr = create_array(10);
    // use arr...
    free(arr);
}
```

### Rust Output

```rust
fn create_array(size: usize) -> Vec<i32> {
    let mut arr = Vec::with_capacity(size);
    for i in 0..size {
        arr.push((i * 2) as i32);
    }
    arr
}

fn use_array() {
    let arr = create_array(10);
    // use arr...
    // Vec automatically freed here
}

#[test]
fn test_create_array() {
    let arr = create_array(5);
    assert_eq!(arr, vec![0, 2, 4, 6, 8]);
}
```

**What Changed**:
- `malloc(size * sizeof(int))` → `Vec::with_capacity(size)`
- `free(arr)` → automatic (Drop trait)
- No memory leaks possible

## Pattern 6: Array Designated Initializers

C99 allows initializing specific array indices.

### C Input

```c
int main() {
    int arr[5] = { [0] = 10, [2] = 20, [4] = 30 };
    return arr[2];
}
```

### Rust Output

```rust
fn main() -> i32 {
    let mut arr = [0i32; 5];
    arr[0] = 10;
    arr[2] = 20;
    arr[4] = 30;
    arr[2]
}

#[test]
fn test_designated_array() {
    let mut arr = [0i32; 5];
    arr[0] = 10;
    arr[2] = 20;
    arr[4] = 30;
    assert_eq!(arr, [10, 0, 20, 0, 30]);
}
```

**What Changed**:
- `[0] = 10` → separate assignment statements
- Default initialization then specific assignments

## Pattern 7: String Iteration Parameters

C often passes strings with iteration logic. Decy transforms these to idiomatic Rust.

### C Input

```c
int count_chars(const char *str) {
    int count = 0;
    while (*str != '\0') {
        count++;
        str++;
    }
    return count;
}
```

### Rust Output

```rust
fn count_chars(s: &str) -> i32 {
    s.len() as i32
}

#[test]
fn test_count_chars() {
    assert_eq!(count_chars("hello"), 5);
    assert_eq!(count_chars(""), 0);
}
```

**What Changed**:
- `const char *str` → `&str`
- Manual iteration → `len()` method
- Null terminator handling eliminated

## How Decy Decides

### Array Type Selection

```
Is it a malloc allocation?
├─ Yes → Vec<T>
└─ No → Is it a function parameter?
   ├─ Yes → &[T] or &mut [T]
   └─ No → [T; N] (fixed array)
```

### Slice Mutability

```
Is the array modified?
├─ Yes → &mut [T]
└─ No → &[T]
```

## Index Type Conversion

Array indices in Rust must be `usize`. Decy automatically converts:

```rust
// C: arr[i] where i is int
// Rust: arr[i as usize]
```

## Summary

| C Pattern | Rust Type | Use Case |
|-----------|-----------|----------|
| `int arr[N]` | `[i32; N]` | Fixed-size array |
| `int *arr` (param) | `&[i32]` | Read-only slice parameter |
| `int *arr` (mutated) | `&mut [i32]` | Mutable slice parameter |
| `malloc(n * sizeof)` | `Vec<T>` | Dynamic array |
| `*(arr + i)` | `arr[i]` | Safe indexing |
| `const char *` | `&str` | String slice |

## Next

- [Structs and Enums](./structs.md) - Struct transformations
- [Pointer Arithmetic Safety](./pointer-arithmetic-safety.md) - Safe pointer operations

---

**Note**: All code examples in this chapter are tested in CI. If they don't compile, our release is blocked!
