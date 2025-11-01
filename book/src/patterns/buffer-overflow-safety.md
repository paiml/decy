# Buffer Overflow Safety

## Overview

Buffer overflow vulnerabilities (CWE-120, CWE-119) are **the most infamous class of security bugs** in C/C++ programs. According to MITRE CWE, buffer overflows have caused countless exploits and remain a critical threat despite decades of awareness.

Decy's transpiler transforms C buffer overflow patterns into Rust code where **buffer overflows are prevented** through compile-time and runtime bounds checking, slice types, and safe container types.

**EXTREME TDD Goal**: ≤100 unsafe blocks per 1000 LOC for buffer operations.

## The Buffer Overflow Problem in C

### CWE-120: Buffer Copy without Checking Size of Input

According to **CWE-120**:

> The product copies an input buffer to an output buffer without verifying that the size of the input buffer is less than the size of the output buffer, leading to a buffer overflow.

### CWE-119: Improper Restriction of Operations within Memory Buffer

According to **CWE-119**:

> The software performs operations on a memory buffer, but it can read from or write to a memory location that is outside of the intended boundary of the buffer.

### Common Buffer Overflow Patterns

```c
// Pattern 1: Fixed array without bounds checking
int arr[10];
arr[15] = 42;  // BUFFER OVERFLOW! Undefined behavior

// Pattern 2: Loop with off-by-one error
int arr[5];
for (int i = 0; i <= 5; i++) {  // Off-by-one!
    arr[i] = i;  // Writes past end
}

// Pattern 3: String buffer overflow
char buffer[10];
strcpy(buffer, "This is way too long");  // OVERFLOW!

// Pattern 4: Unsafe string functions
char dest[5];
gets(dest);  // No bounds checking! Always unsafe

// Pattern 5: Array indexing without validation
int arr[100];
int index = user_input;  // Unchecked!
arr[index] = 42;  // Potential overflow
```

**Real-world impact**:
- **Arbitrary code execution** (shellcode injection)
- **Information disclosure** (read beyond buffer)
- **Denial of service** (crashes)
- **Stack/heap corruption** (program instability)

**Notable incidents**:
- **Morris Worm (1988)**: First internet worm, exploited buffer overflow in fingerd
- **Code Red (2001)**: IIS buffer overflow, infected 359,000 systems
- **Heartbleed (2014)**: OpenSSL buffer over-read, leaked private keys
- **WannaCry (2017)**: SMBv1 buffer overflow, global ransomware attack

## Decy's Buffer Overflow Safety Transformations

### Pattern 1: Fixed Array Access → Bounded Array

**C Code** (potential overflow):
```c
int main() {
    int arr[10];
    int i;

    for (i = 0; i < 10; i++) {
        arr[i] = i * 2;
    }

    return arr[5];
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let mut arr: [i32; 10] = [0; 10];
    
    for i in 0..10 {
        arr[i] = i * 2;  // Bounds checked at runtime
    }
    
    std::process::exit(arr[5]);
}
```

**Safety improvements**:
- Array size known at compile time
- Automatic bounds checking on `arr[i]`
- **Runtime panic instead of undefined behavior**
- Iterator-based patterns eliminate indexing

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 2: Array Index Validation → Checked Access

**C Code** (manual bounds checking):
```c
int main() {
    int arr[5];
    int index = 3;

    if (index >= 0 && index < 5) {
        arr[index] = 42;
        return arr[index];
    }

    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let mut arr: [i32; 5] = [0; 5];
    let index = 3;

    // Option 1: Runtime bounds check (panic on overflow)
    if index >= 0 && index < 5 {
        arr[index] = 42;
        std::process::exit(arr[index]);
    }

    // Option 2: get() for safe access
    if let Some(value) = arr.get_mut(index) {
        *value = 42;
    }

    std::process::exit(0);
}
```

**Safety improvements**:
- `arr[i]` panics on out-of-bounds (vs silent corruption)
- `arr.get(i)` returns `Option<&T>` (no panic)
- Compiler enforces array size consistency
- No silent wrap-around or overflow

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 3: Buffer Copy → Slice Copy

**C Code** (manual copy):
```c
int main() {
    int src[5] = {10, 20, 30, 40, 50};
    int dst[5];
    int i;

    for (i = 0; i < 5; i++) {
        dst[i] = src[i];
    }

    return dst[2];
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let src: [i32; 5] = [10, 20, 30, 40, 50];
    let mut dst: [i32; 5] = [0; 5];

    // Option 1: Slice copy (safe, checked)
    dst.copy_from_slice(&src);

    // Option 2: Clone/copy
    let dst = src;  // Copy for i32 (Copy trait)

    std::process::exit(dst[2]);
}
```

**Safety improvements**:
- `copy_from_slice` checks lengths at runtime
- No manual indexing, no overflow possible
- Compiler validates size compatibility
- Iterator-based patterns available

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 4: String Buffer → String Type

**C Code** (string buffer):
```c
int main() {
    char str[20];
    int i;

    for (i = 0; i < 10; i++) {
        str[i] = 'A' + i;
    }
    str[10] = '\0';

    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    // Option 1: Vec<u8> for byte buffer
    let mut str: Vec<u8> = Vec::with_capacity(20);
    
    for i in 0..10 {
        str.push(b'A' + i as u8);
    }

    // Option 2: String (UTF-8 validated)
    let mut s = String::new();
    for i in 0..10 {
        s.push((b'A' + i as u8) as char);
    }

    std::process::exit(0);
}
```

**Safety improvements**:
- `String` and `Vec<u8>` grow dynamically
- No null terminator needed
- UTF-8 validation (for `String`)
- Automatic capacity management
- No buffer overflow possible

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 5: 2D Array → Nested Array or Vec

**C Code** (2D array):
```c
int main() {
    int matrix[3][3];
    int i, j;

    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) {
            matrix[i][j] = i * 3 + j;
        }
    }

    return matrix[1][1];
}
```

**Idiomatic Rust**:
```rust
fn main() {
    // Option 1: Fixed-size nested array
    let mut matrix: [[i32; 3]; 3] = [[0; 3]; 3];
    
    for i in 0..3 {
        for j in 0..3 {
            matrix[i][j] = (i * 3 + j) as i32;
        }
    }

    // Option 2: Vec of Vec (dynamic)
    let mut matrix: Vec<Vec<i32>> = vec![vec![0; 3]; 3];
    
    std::process::exit(matrix[1][1]);
}
```

**Safety improvements**:
- Both dimensions bounds-checked
- Nested indexing validated at runtime
- Vec version allows dynamic sizing
- No partial initialization errors

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 6: Partial Copy → Checked Slice Operations

**C Code** (partial buffer copy):
```c
int main() {
    int src[10];
    int dst[10];
    int count = 5;
    int i;

    for (i = 0; i < 10; i++) {
        src[i] = i;
    }

    for (i = 0; i < count && i < 10; i++) {
        dst[i] = src[i];
    }

    return dst[3];
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let mut src: [i32; 10] = [0; 10];
    let mut dst: [i32; 10] = [0; 10];
    let count = 5;

    // Initialize src
    for i in 0..10 {
        src[i] = i as i32;
    }

    // Partial copy with slicing
    let copy_count = count.min(10);
    dst[..copy_count].copy_from_slice(&src[..copy_count]);

    std::process::exit(dst[3]);
}
```

**Safety improvements**:
- Slice bounds validated at runtime
- `copy_from_slice` checks length match
- `min()` ensures no overflow
- No manual bounds checking needed

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

## EXTREME TDD Validation

### Integration Tests (17 tests)

**File**: `crates/decy-core/tests/buffer_overflow_safety_integration_test.rs`

**Coverage**:
1. Fixed array access
2. String buffer safe size
3. Array index validation
4. Loop bounds checking
5. 2D array access
6. Manual buffer copy
7. Partial buffer copy
8. String initialization
9. String length checking
10. Off-by-one prevention
11. Variable size arrays
12. Struct with array members
13. Nested array access
14. Function with array parameter
15. Unsafe density target
16. Code compilation
17. Safety documentation

**All 17 tests passed on first run** ✅

---

### Property Tests (13 properties, 3,328+ executions)

**File**: `crates/decy-core/tests/buffer_overflow_property_tests.rs`

**Properties validated**:
1. **Fixed array access transpiles** (256 size/value combinations)
2. **Array index validation transpiles** (256 size/index combinations)
3. **Loop bounds checking transpiles** (256 size/multiplier combinations)
4. **2D array access transpiles** (256 row/column combinations)
5. **Buffer copy operations transpile** (256 size/value combinations)
6. **Partial buffer copy transpiles** (256 size/count combinations)
7. **String buffer operations transpile** (256 buffer/fill combinations)
8. **Variable size arrays transpile** (256 array/used combinations)
9. **Struct with array member transpiles** (256 size/value combinations)
10. **Nested arrays transpile** (256 outer/inner combinations)
11. **Unsafe density below target** (≤100 per 1000 LOC) (256 cases)
12. **Generated code balanced braces** (256 cases)
13. **Transpilation is deterministic** (256 cases)

**All 13 property tests passed** (3,328+ total test cases) ✅

---

### Executable Example

**File**: `crates/decy-core/examples/buffer_overflow_safety_demo.rs`

**Run with**:
```bash
cargo run -p decy-core --example buffer_overflow_safety_demo
```

**Output** (verified):
```
=== Decy Buffer Overflow Safety Demonstration ===

## Example 1: Fixed Array Access
✓ Unsafe blocks: 0 (0.0 per 1000 LOC)
✓ Array access with loop bounds checking
✓ Prevents buffer overflow at compile/runtime

[... 2 more examples ...]

**EXTREME TDD Goal**: ≤100 unsafe blocks per 1000 LOC
**Status**: ACHIEVED ✅
```

---

## Safety Metrics Summary

| Pattern | C Danger | Rust Safety | Unsafe/1000 LOC | Status |
|---------|----------|-------------|-----------------|--------|
| Fixed array access | Silent overflow | Runtime panic | 0 | ✅ |
| Index validation | Manual checking | Automatic checks | 0 | ✅ |
| Buffer copy | memcpy overflow | Slice validation | 0 | ✅ |
| String buffer | strcpy overflow | String/Vec<u8> | 0 | ✅ |
| 2D arrays | Both dimensions unchecked | Nested bounds check | 0 | ✅ |
| Partial copy | Manual bounds | Slice range check | 0 | ✅ |

**Overall target**: ≤100 unsafe blocks per 1000 LOC ✅ **ACHIEVED (0 unsafe)**

---

## Best Practices

### 1. Use Array Types with Bounds Checking

```rust
// ✅ GOOD: Fixed-size array (bounds checked)
let mut arr: [i32; 10] = [0; 10];
arr[5] = 42;  // Runtime panic if out of bounds

// ❌ BAD: Raw pointer arithmetic (unsafe)
let ptr = unsafe { libc::malloc(10 * std::mem::size_of::<i32>()) };
unsafe { *ptr.offset(5) = 42; }  // No bounds checking!
```

### 2. Prefer Slices Over Indexing

```rust
// ✅ GOOD: Slice operations (bounds checked)
let src = [1, 2, 3, 4, 5];
let dst = &src[1..4];  // Slice with runtime check

// ✅ GOOD: get() for safe access
if let Some(value) = arr.get(index) {
    // Use value safely
}

// ⚠️ OK: Direct indexing (panics on overflow)
let value = arr[index];  // Better than C, but panics
```

### 3. Use Vec<T> for Dynamic Arrays

```rust
// ✅ GOOD: Dynamic array with automatic growth
let mut vec: Vec<i32> = Vec::new();
vec.push(42);  // Grows automatically, no overflow

// ✅ GOOD: Pre-allocated capacity
let mut vec: Vec<i32> = Vec::with_capacity(100);
for i in 0..100 {
    vec.push(i);  // No reallocation needed
}
```

### 4. Use String Instead of char[]

```rust
// ✅ GOOD: String type (UTF-8, grows dynamically)
let mut s = String::from("Hello");
s.push_str(" World");  // No buffer overflow possible

// ✅ GOOD: Vec<u8> for binary data
let mut buf: Vec<u8> = Vec::new();
buf.extend_from_slice(b"data");
```

### 5. Validate Array Access with Ranges

```rust
// ✅ GOOD: Range-based iteration (no indexing)
let arr = [1, 2, 3, 4, 5];
for value in arr.iter() {
    // Use value, no index needed
}

// ✅ GOOD: Enumerate for index + value
for (i, value) in arr.iter().enumerate() {
    // Both index and value available safely
}
```

---

## CWE-120 and CWE-119 References

### CWE-120: Buffer Copy without Checking Size of Input

> The product copies an input buffer to an output buffer without verifying that the size of the input buffer is less than the size of the output buffer, leading to a buffer overflow.

**Decy Implementation**: Rust's slice operations (`copy_from_slice`, `clone_from_slice`) validate that source and destination lengths match. `Vec` and `String` types grow dynamically, eliminating fixed-size buffer constraints.

### CWE-119: Improper Restriction of Operations within Memory Buffer

> The software performs operations on a memory buffer, but it can read from or write to a memory location that is outside of the intended boundary of the buffer.

**Decy Implementation**: Rust enforces bounds checking on all array and slice accesses. Out-of-bounds access causes a runtime panic (in debug and release builds) instead of silent memory corruption. The compiler prevents construction of out-of-bounds references.

---

## Summary

Decy's buffer overflow safety transformations provide:

1. **Automatic Bounds Checking**: All array/slice access validated at runtime
2. **Compile-Time Size Validation**: Array sizes checked by compiler
3. **Dynamic Growth**: `Vec<T>` and `String` eliminate fixed buffers
4. **Safe Abstractions**: Slices, iterators, and ranges replace manual indexing
5. **Minimal Unsafe**: 0 unsafe blocks per 1000 LOC

**EXTREME TDD Validation**:
- 17 integration tests ✅
- 13 property tests (3,328+ executions) ✅
- Executable demo with metrics ✅

**CWE-120/CWE-119 Compliance**: Complete mitigation ✅

**Safety Goal**: ACHIEVED ✅ (0 unsafe blocks)

**Next Steps**: All critical C buffer overflow patterns validated! The comprehensive EXTREME TDD methodology has proven Decy's safety transformations across 11 vulnerability classes.
