# Format String Safety

## Overview

Format string vulnerabilities (CWE-134) are among the **most dangerous** class of software bugs. According to OWASP, format string bugs can lead to **arbitrary code execution**, **information disclosure**, and **denial of service**. These vulnerabilities occur when user-controlled input is used as a format string in functions like `printf`, `sprintf`, and `scanf`.

Decy's transpiler transforms dangerous C format string patterns into safe Rust code with compile-time format validation and type-safe formatting.

**EXTREME TDD Goal**: ≤30 unsafe blocks per 1000 LOC for format string operations.

## The Format String Problem in C

### CWE-134: Use of Externally-Controlled Format String

According to **CWE-134**:

> The software uses a function that accepts a format string as an argument, but the format string originates from an external source.

### Common Format String Vulnerabilities

```c
// Pattern 1: User input as format string (CRITICAL VULNERABILITY!)
char* user_input = get_user_input();
printf(user_input);  // DANGEROUS! Allows format string injection

// Pattern 2: Unbounded sprintf (buffer overflow)
char buffer[10];
sprintf(buffer, "Very long string: %s", some_string);  // Buffer overflow!

// Pattern 3: Mismatched format specifiers
printf("%d %d\n", 42);  // Missing argument - undefined behavior!

// Pattern 4: scanf without width specifier
char buffer[10];
scanf("%s", buffer);  // No bounds checking - overflow possible!
```

**Real-world impact**:
- **Arbitrary code execution** (format string injection, stack manipulation)
- **Information disclosure** (reading stack/memory with %x, %s)
- **Denial of service** (crashes from invalid format specifiers)
- **Privilege escalation** (modifying memory with %n)

**Notable incidents**:
- **CVE-2000-0844**: Wu-ftpd format string vulnerability → remote root
- **CVE-2012-0809**: Sudo format string vulnerability → privilege escalation
- **CVE-2015-1838**: Wireshark format string → code execution

## Decy's Format String Safety Transformations

### Pattern 1: Safe printf with Format String

**C Code**:
```c
#include <stdio.h>

int main() {
    int value = 42;
    printf("Value: %d\n", value);
    return 0;
}
```

**Decy-Generated Rust**:
```rust
fn main() {
    let value: i32 = 42;
    println!("Value: {}", value);
    std::process::exit(0);
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let value = 42;
    println!("Value: {}", value);  // Compile-time format validation
}
```

**Safety improvements**:
- Format string is compile-time constant
- Type-safe format arguments (`{}` checks types)
- No format string injection possible

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 2: printf with Multiple Arguments

**C Code**:
```c
#include <stdio.h>

int main() {
    int a = 10;
    int b = 20;
    printf("a=%d, b=%d, sum=%d\n", a, b, a + b);
    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let a = 10;
    let b = 20;
    println!("a={}, b={}, sum={}", a, b, a + b);
}
```

**Safety improvements**:
- Compiler validates argument count matches format string
- Type checking for each argument
- No undefined behavior from missing arguments

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 3: sprintf (Unbounded) → Bounded String Formatting

**C Code** (dangerous):
```c
#include <stdio.h>

int main() {
    char buffer[100];
    int value = 42;
    sprintf(buffer, "Value: %d", value);  // No bounds checking!
    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let value = 42;
    let buffer = format!("Value: {}", value);  // Heap-allocated, grows as needed
    // Or with fixed capacity:
    let mut buffer = String::with_capacity(100);
    use std::fmt::Write;
    write!(&mut buffer, "Value: {}", value).unwrap();
}
```

**Safety improvements**:
- `format!()` allocates exactly the right size
- No buffer overflow possible
- Compile-time format validation

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 4: snprintf (Bounded) → Safe String Formatting

**C Code**:
```c
#include <stdio.h>

int main() {
    char buffer[50];
    int value = 42;
    snprintf(buffer, sizeof(buffer), "Value: %d", value);
    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let value = 42;
    let buffer = format!("Value: {}", value);

    // If you need fixed-size buffer:
    let mut buffer = [0u8; 50];
    use std::io::Write;
    let bytes = format!("Value: {}", value).as_bytes();
    let len = bytes.len().min(buffer.len());
    buffer[..len].copy_from_slice(&bytes[..len]);
}
```

**Safety improvements**:
- Explicit size limit respected
- No buffer overflow
- Clear truncation semantics

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 5: Format Specifiers (%d, %s, %f, etc.)

**C Code**:
```c
#include <stdio.h>

int main() {
    int i = 42;
    double d = 3.14;
    char* s = "test";

    printf("int=%d, double=%f, string=%s\n", i, d, s);
    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let i = 42;
    let d = 3.14;
    let s = "test";

    println!("int={}, double={}, string={}", i, d, s);
    // Or with formatting:
    println!("int={:}, double={:.2}, string={}", i, d, s);
}
```

**Rust format specifiers**:
- `{}` - Display (default formatting)
- `{:?}` - Debug (programmer-facing)
- `{:.2}` - Precision for floats
- `{:10}` - Width
- `{:x}` - Lowercase hex
- `{:X}` - Uppercase hex
- `{:b}` - Binary

**Safety improvements**:
- Type-safe format specifiers
- Compile-time validation
- No undefined behavior from mismatched types

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 6: Width and Precision Specifiers

**C Code**:
```c
#include <stdio.h>

int main() {
    int value = 42;
    double pi = 3.14159;

    printf("%10d\n", value);      // Width 10
    printf("%.2f\n", pi);          // 2 decimal places
    printf("%10.2f\n", pi);        // Width 10, 2 decimals
    return 0;
}
```

**Idiomatic Rust**:
```rust
fn main() {
    let value = 42;
    let pi = 3.14159;

    println!("{:10}", value);     // Width 10
    println!("{:.2}", pi);         // 2 decimal places
    println!("{:10.2}", pi);       // Width 10, 2 decimals
}
```

**Safety improvements**:
- Width and precision preserved
- No buffer overflow from width specifiers
- Type-safe formatting

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 7: scanf with Width Specifier

**C Code** (with width - safer):
```c
#include <stdio.h>

int main() {
    char buffer[10];
    scanf("%9s", buffer);  // Prevents overflow (leaves room for \0)
    return 0;
}
```

**C Code** (without width - dangerous):
```c
char buffer[10];
scanf("%s", buffer);  // DANGEROUS! No bounds checking
```

**Idiomatic Rust**:
```rust
use std::io::{self, BufRead};

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut buffer).unwrap();

    // Or for specific types:
    let value: i32 = buffer.trim().parse().unwrap_or(0);
}
```

**Safety improvements**:
- `String` grows automatically (no buffer overflow)
- Type-safe parsing with `parse()`
- Error handling with `Result<T, E>`

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

### Pattern 8: Format String Injection Prevention

**C Code** (CRITICAL VULNERABILITY):
```c
#include <stdio.h>

int main() {
    char* user_input = get_user_input();
    printf(user_input);  // DANGEROUS! Format string injection!
    return 0;
}
```

**Correct C Code**:
```c
printf("%s", user_input);  // Safe: user input is data, not format
```

**Idiomatic Rust**:
```rust
fn main() {
    let user_input = get_user_input();
    println!("{}", user_input);  // Safe by design

    // Format string MUST be compile-time constant:
    // println!(user_input);  // Compile error!
}
```

**Safety improvements**:
- Format strings must be string literals (compile-time constants)
- Impossible to use user input as format string
- No format string injection possible

**Metrics**: 0 unsafe blocks per 1000 LOC ✅

---

## EXTREME TDD Validation

### Integration Tests (19 tests)

**File**: `crates/decy-core/tests/format_string_safety_integration_test.rs`

**Coverage**:
1. Safe printf with format string
2. printf with multiple arguments
3. printf with string format
4. sprintf with bounds
5. snprintf bounded
6. scanf with width specifier
7. scanf integer input
8. printf integer formats (%d, %u, %x, %o)
9. printf float formats (%f, %e, %g)
10. printf char format (%c)
11. printf width specifier
12. printf precision specifier
13. printf complex format (multiple types)
14. sprintf to buffer
15. printf escape sequences (\n, \t)
16. printf percent escape (%%)
17. Unsafe density target
18. Transpiled code compiles
19. Safety documentation

**All 19 tests passed on first run** ✅

---

### Property Tests (12 properties, 3,072+ executions)

**File**: `crates/decy-core/tests/format_string_property_tests.rs`

**Properties validated**:
1. **printf with integer transpiles** (256 values from -1000 to 1000)
2. **printf with multiple integers transpiles** (256 a/b pairs)
3. **printf with float transpiles** (256 floats)
4. **sprintf to buffer transpiles** (256 buffer sizes and values)
5. **snprintf bounded transpiles** (256 buffer sizes and values)
6. **printf with width transpiles** (256 width/value combinations)
7. **printf with precision transpiles** (256 precision/value combinations)
8. **scanf with width transpiles** (256 buffer sizes)
9. **printf with hex format transpiles** (256 hex values)
10. **Unsafe density below target** (≤30 per 1000 LOC) (256 cases)
11. **Generated code balanced braces** (256 cases)
12. **Transpilation is deterministic** (256 cases)

**All 12 property tests passed** (3,072+ total test cases) ✅

---

### Executable Example

**File**: `crates/decy-core/examples/format_string_safety_demo.rs`

**Run with**:
```bash
cargo run -p decy-core --example format_string_safety_demo
```

**Output** (verified):
```
=== Decy Format String Safety Demonstration ===

## Example 1: Safe printf with Format String
✓ Unsafe blocks: 0 (0.0 per 1000 LOC)
✓ Format string is compile-time constant
✓ Type-safe formatting

[... 5 more examples ...]

**EXTREME TDD Goal**: ≤30 unsafe blocks per 1000 LOC
**Status**: ACHIEVED ✅
```

---

## Safety Metrics Summary

| Pattern | C Danger | Rust Safety | Unsafe/1000 LOC | Status |
|---------|----------|-------------|-----------------|--------|
| printf | Format string injection | Compile-time literals | 0 | ✅ |
| sprintf | Buffer overflow | format!() allocates | 0 | ✅ |
| snprintf | Still unbounded in C | Bounded String | 0 | ✅ |
| scanf | No width → overflow | String grows automatically | 0 | ✅ |
| Format specifiers | Type mismatches | Type-safe formatting | 0 | ✅ |
| Width/precision | Buffer overflow | Safe formatting | 0 | ✅ |
| User input | Injection attack | Literals only | 0 | ✅ |

**Overall target**: ≤30 unsafe blocks per 1000 LOC ✅ **ACHIEVED (0 unsafe)**

---

## Best Practices

### 1. Use println!/format! Instead of Raw Formatting

```rust
// ✅ GOOD: Type-safe formatting
println!("Value: {}", value);

// ❌ BAD: Raw C-style (not available in safe Rust)
// printf("Value: %d\n", value);
```

### 2. Use format! for String Building

```rust
// ✅ GOOD: format! allocates correctly
let message = format!("Hello, {}!", name);

// ❌ BAD: Manual buffer management
let mut buffer = [0u8; 100];
// ... complex sprintf-like code
```

### 3. Use parse() for Type-Safe Input

```rust
// ✅ GOOD: Type-safe parsing
let value: i32 = input.trim().parse().unwrap_or(0);

// ❌ BAD: Unsafe scanf-like parsing
// scanf("%d", &value);
```

### 4. Use Debug/Display Traits

```rust
// ✅ GOOD: Implement Display for custom types
impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

let p = Point { x: 10, y: 20 };
println!("{}", p);  // Calls Display::fmt
```

### 5. Handle Formatting Errors

```rust
// ✅ GOOD: Handle write! errors
use std::fmt::Write;
let mut s = String::new();
write!(&mut s, "Value: {}", value)?;

// Or use format! which can't fail:
let s = format!("Value: {}", value);
```

---

## Edge Cases Validated

### Multiple Format Arguments
```rust
println!("{} {} {} {}", a, b, c, d);  // All type-checked
```

### Nested Format Strings
```rust
let inner = format!("Inner: {}", x);
println!("Outer: {}", inner);  // Safe nesting
```

### Custom Format Specifiers
```rust
println!("{:?}", value);      // Debug
println!("{:#?}", value);     // Pretty debug
println!("{:x}", value);      // Hex
println!("{:b}", value);      // Binary
```

### Width and Alignment
```rust
println!("{:<10}", value);    // Left-align, width 10
println!("{:>10}", value);    // Right-align, width 10
println!("{:^10}", value);    // Center, width 10
```

---

## CWE-134 References

### CWE-134: Use of Externally-Controlled Format String

> The software uses a function that accepts a format string as an argument, but the format string originates from an external source.

**Decy Implementation**: Rust's `println!()` and `format!()` macros require format strings to be compile-time constants, making format string injection impossible.

### Common Consequences

- **Confidentiality**: Read memory via %x, %s
- **Integrity**: Modify memory via %n
- **Availability**: Crash via invalid format strings

**Decy Implementation**: All format arguments are type-checked at compile time, preventing all of these attacks.

---

## Summary

Decy's format string safety transformations provide:

1. **Compile-Time Format Validation**: Format strings must be literals
2. **Type-Safe Formatting**: Arguments type-checked against format string
3. **No Format String Injection**: User input cannot be format string
4. **Automatic Buffer Management**: `format!()` allocates correctly
5. **Minimal Unsafe**: 0 unsafe blocks per 1000 LOC

**EXTREME TDD Validation**:
- 19 integration tests ✅
- 12 property tests (3,072+ executions) ✅
- Executable demo with metrics ✅

**CWE-134 Compliance**: Complete mitigation ✅

**Safety Goal**: ACHIEVED ✅ (0 unsafe blocks)

**Next Steps**: Explore [Concurrency Safety](./concurrency-safety.md) for data race prevention patterns (future chapter).
