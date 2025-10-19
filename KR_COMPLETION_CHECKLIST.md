# K&R C Book Completion Checklist

**Goal**: Complete all remaining K&R C (2nd Edition) language constructs to achieve 95%+ coverage
**Current Progress**: 116/150 constructs (77% complete)
**Target**: 143/150 constructs (95% complete)
**Remaining**: 34 constructs to implement/document

---

## Progress Overview

| Category | Total | Completed | Remaining | % Complete |
|----------|-------|-----------|-----------|------------|
| **Data Types** | 15 | 15 | 0 | 100% ✅ |
| **Operators** | 25 | 24 | 1 | 96% ⏳ |
| **Statements** | 12 | 12 | 0 | 100% ✅ |
| **Functions** | 10 | 10 | 0 | 100% ✅ |
| **Memory Management** | 8 | 8 | 0 | 100% ✅ |
| **String & I/O** | 12 | 11 | 1 | 92% ⏳ |
| **Preprocessor** | 20 | 6 | 14 | 30% ❌ |
| **Advanced Features** | 18 | 5 | 13 | 28% ❌ |
| **C99 Specific** | 30 | 25 | 5 | 83% ⏳ |
| **TOTAL** | **150** | **116** | **34** | **77%** |

---

## Remaining Tasks by Priority

### 🔴 HIGH PRIORITY (Complete These First)

#### 1. Preprocessor Directives (14 remaining)

**Why Critical**: Essential for real-world C code (headers, platform-specific code)

- ❌ **Complex Macro Expansion** (DECY-XXX)
  - Reference: K&R §A12.3, C99 §6.10.3
  - C: `#define MAX(a,b) ((a)>(b)?(a):(b))`
  - Rust: `macro_rules!` or inline functions
  - Priority: **CRITICAL**
  - Story Points: 13

- ❌ **Conditional Compilation** (DECY-XXX)
  - Reference: K&R §A12.4, C99 §6.10.1
  - C: `#if`, `#elif`, `#else`, `#endif`
  - Rust: `cfg!`, `#[cfg(...)]`
  - Priority: **CRITICAL**
  - Story Points: 8

- ❌ **Advanced #ifdef/#ifndef** (DECY-XXX)
  - Reference: K&R §A12.5, C99 §6.10.1
  - C: Nested conditional compilation
  - Rust: Feature flags, cfg attributes
  - Priority: **HIGH**
  - Story Points: 5

- ❌ **#pragma Directives** (DECY-XXX)
  - Reference: K&R §A12.8, C99 §6.10.6
  - C: `#pragma pack`, `#pragma once`
  - Rust: `#[repr(packed)]`, path-based module system
  - Priority: **HIGH**
  - Story Points: 8

- ❌ **Macro Stringification** (DECY-XXX)
  - Reference: K&R §A12.3.3, C99 §6.10.3.2
  - C: `#define STR(x) #x`
  - Rust: `stringify!` macro
  - Priority: **MEDIUM**
  - Story Points: 3

- ❌ **Token Pasting** (DECY-XXX)
  - Reference: K&R §A12.3.4, C99 §6.10.3.3
  - C: `#define CONCAT(a,b) a##b`
  - Rust: Complex macro expansion
  - Priority: **MEDIUM**
  - Story Points: 5

- ❌ **Predefined Macros** (DECY-XXX)
  - Reference: K&R §A12.2, C99 §6.10.8
  - C: `__FILE__`, `__LINE__`, `__DATE__`
  - Rust: `file!()`, `line!()`, compile-time constants
  - Priority: **LOW**
  - Story Points: 3

- ❌ **#undef Directive** (DECY-XXX)
  - Reference: K&R §A12.6, C99 §6.10.6
  - C: Undefine macros
  - Rust: Module scoping
  - Priority: **LOW**
  - Story Points: 2

- ❌ **#error Directive** (DECY-XXX)
  - Reference: C99 §6.10.5
  - C: Compile-time errors
  - Rust: `compile_error!` macro
  - Priority: **LOW**
  - Story Points: 2

- ❌ **#warning Directive** (DECY-XXX)
  - Reference: GCC extension
  - C: Compile-time warnings
  - Rust: Custom derive warnings
  - Priority: **LOW**
  - Story Points: 2

- ❌ **Line Control** (DECY-XXX)
  - Reference: K&R §A12.7, C99 §6.10.4
  - C: `#line` directive
  - Rust: Debug info preservation
  - Priority: **LOW**
  - Story Points: 2

- ❌ **Macro Variadic Arguments** (DECY-XXX)
  - Reference: C99 §6.10.3
  - C: `#define LOG(...) printf(__VA_ARGS__)`
  - Rust: `macro_rules!` with repetition
  - Priority: **MEDIUM**
  - Story Points: 5

- ❌ **Include Guards** (DECY-XXX)
  - Reference: K&R §A12.1
  - C: `#ifndef HEADER_H` pattern
  - Rust: Module system (no guards needed)
  - Priority: **LOW**
  - Story Points: 2

- ❌ **Recursive Macro Expansion** (DECY-XXX)
  - Reference: K&R §A12.3.5
  - C: Self-referential macros
  - Rust: Complex declarative macros
  - Priority: **MEDIUM**
  - Story Points: 8

**Preprocessor Subtotal**: 14 tasks, 68 story points

---

#### 2. File I/O Completion (1 remaining)

**Why Critical**: Chapter 7 completion

- ⏳ **scanf Family** (DECY-XXX) - 80% complete
  - Reference: K&R §7.4, C99 §7.19.6.2
  - C: `scanf`, `fscanf`, `sscanf`
  - Rust: String parsing + pattern matching
  - Priority: **HIGH**
  - Story Points: 5
  - Remaining: Complex format specifiers

**File I/O Subtotal**: 1 task, 5 story points

---

### 🟡 MEDIUM PRIORITY (Important for Coverage)

#### 3. Advanced C99 Features (5 remaining)

- ❌ **Complex Numbers** (DECY-XXX)
  - Reference: C99 §6.2.5.11, §7.3
  - C: `_Complex`, `complex.h`
  - Rust: `num-complex` crate
  - Priority: **LOW**
  - Story Points: 8

- ⏳ **Hexadecimal Float Literals** (DECY-XXX) - Documentation only
  - Reference: C99 §6.4.4.2
  - C: `0x1.fp10`
  - Rust: Manual conversion
  - Priority: **LOW**
  - Story Points: 3

- ⏳ **restrict Keyword** (DECY-XXX) - Documentation only
  - Reference: C99 §6.7.3.1
  - C: Pointer aliasing hint
  - Rust: No direct equivalent (borrow checker handles this)
  - Priority: **LOW**
  - Story Points: 2

- ⏳ **inline Keyword** (DECY-XXX) - Documentation only
  - Reference: C99 §6.7.4
  - C: Inline function hint
  - Rust: `#[inline]` attribute
  - Priority: **LOW**
  - Story Points: 2

- ❌ **Variable-Length Arrays (VLA) - Full Support** (DECY-XXX)
  - Reference: C99 §6.7.5.2
  - C: `int arr[n]` where n is runtime value
  - Rust: `Vec<T>` or heap allocation
  - Priority: **MEDIUM**
  - Story Points: 5
  - Current: Basic support exists, needs edge cases

**C99 Subtotal**: 5 tasks, 20 story points

---

#### 4. Advanced Features (13 remaining)

- ⏳ **goto Statements** (DECY-XXX) - Documentation only
  - Reference: K&R §3.8, C99 §6.8.6.1
  - C: `goto label;`
  - Rust: Loop labels with break/continue
  - Priority: **MEDIUM**
  - Story Points: 3

- ❌ **setjmp/longjmp** (DECY-XXX)
  - Reference: K&R §B9, C99 §7.13
  - C: Non-local jumps
  - Rust: `Result<T>` + early returns, or `catch_unwind`
  - Priority: **MEDIUM**
  - Story Points: 8

- ❌ **Signal Handling** (DECY-XXX)
  - Reference: K&R §B10, C99 §7.14
  - C: `signal`, `raise`
  - Rust: `signal-hook` crate or channels
  - Priority: **MEDIUM**
  - Story Points: 8

- ❌ **Bit Fields** (DECY-XXX)
  - Reference: K&R §6.9, C99 §6.7.2.1
  - C: `struct { unsigned int x:3; }`
  - Rust: Manual bit manipulation or `bitflags` crate
  - Priority: **MEDIUM**
  - Story Points: 8

- ⏳ **Flexible Array Members** (DECY-XXX) - Documentation only
  - Reference: C99 §6.7.2.1
  - C: `struct { int n; int data[]; }`
  - Rust: `Vec<T>` or unsafe with custom allocator
  - Priority: **LOW**
  - Story Points: 5

- ⏳ **Compound Literals** (DECY-XXX) - Documentation only
  - Reference: C99 §6.5.2.5
  - C: `(struct Point){.x=1, .y=2}`
  - Rust: Struct literals
  - Priority: **LOW**
  - Story Points: 3

- ⏳ **Designated Initializers** (DECY-XXX) - Documentation only
  - Reference: C99 §6.7.8
  - C: `int arr[10] = {[5]=10, [7]=20}`
  - Rust: Manual array construction
  - Priority: **LOW**
  - Story Points: 3

- ❌ **Volatile Semantics** (DECY-XXX)
  - Reference: K&R §A8.2, C99 §6.7.3
  - C: `volatile int *ptr`
  - Rust: `core::ptr::read_volatile`, `write_volatile`
  - Priority: **MEDIUM**
  - Story Points: 5

- ❌ **Register Keyword** (DECY-XXX)
  - Reference: K&R §A8.1, C99 §6.7.1
  - C: `register int x;`
  - Rust: Ignored (LLVM optimizes)
  - Priority: **LOW**
  - Story Points: 1

- ❌ **Static Assertions** (DECY-XXX)
  - Reference: C11 §6.7.10
  - C: `_Static_assert(sizeof(int)==4, "msg")`
  - Rust: `static_assertions` crate or const assertions
  - Priority: **LOW**
  - Story Points: 2

- ❌ **Atomic Operations** (DECY-XXX)
  - Reference: C11 §7.17
  - C: `_Atomic`, atomic operations
  - Rust: `std::sync::atomic`
  - Priority: **MEDIUM**
  - Story Points: 8

- ❌ **Thread-Local Storage** (DECY-XXX)
  - Reference: C11 §6.7.1
  - C: `_Thread_local`
  - Rust: `thread_local!` macro
  - Priority: **MEDIUM**
  - Story Points: 5

- ❌ **Alignment Specifiers** (DECY-XXX)
  - Reference: C11 §6.7.5
  - C: `_Alignas(16)`
  - Rust: `#[repr(align(16))]`
  - Priority: **LOW**
  - Story Points: 3

**Advanced Features Subtotal**: 13 tasks, 62 story points

---

### 🟢 LOW PRIORITY (Nice to Have)

#### 5. Cast Operators (1 remaining)

- ⏳ **Complex Type Casts** (DECY-XXX) - 80% complete
  - Reference: K&R §6.5, C99 §6.5.4
  - C: Function pointer casts, union type punning
  - Rust: `std::mem::transmute` (unsafe)
  - Priority: **LOW**
  - Story Points: 3

**Cast Operators Subtotal**: 1 task, 3 story points

---

## Implementation Roadmap

### Sprint 8: Preprocessor Foundation (2 weeks)
**Goal**: Enable basic real-world C header parsing

- [ ] DECY-XXX: Complex macro expansion (13 points)
- [ ] DECY-XXX: Conditional compilation (#if/#ifdef) (8 points)
- [ ] DECY-XXX: Macro variadic arguments (5 points)
- [ ] **Total**: 26 story points

**Acceptance Criteria**:
- Parse common C headers (stdio.h, stdlib.h snippets)
- Handle platform-specific #ifdef blocks
- Variadic printf-like macros work

---

### Sprint 9: File I/O + Advanced Preprocessor (2 weeks)
**Goal**: Complete Chapter 7, enhance preprocessor

- [ ] DECY-XXX: scanf family completion (5 points)
- [ ] DECY-XXX: #pragma directives (8 points)
- [ ] DECY-XXX: Token pasting (##) (5 points)
- [ ] DECY-XXX: Advanced #ifdef/#ifndef (5 points)
- [ ] **Total**: 23 story points

**Acceptance Criteria**:
- All Chapter 7 I/O functions supported
- Header guard patterns recognized
- Platform-specific code handled

---

### Sprint 10: Advanced Features Part 1 (2 weeks)
**Goal**: Bit fields, signals, setjmp/longjmp

- [ ] DECY-XXX: Bit fields (8 points)
- [ ] DECY-XXX: Signal handling (8 points)
- [ ] DECY-XXX: setjmp/longjmp (8 points)
- [ ] DECY-XXX: Volatile semantics (5 points)
- [ ] **Total**: 29 story points

**Acceptance Criteria**:
- Struct bit fields → Rust bit manipulation
- signal() → Rust signal handling
- Error handling with setjmp → Result<T>

---

### Sprint 11: Advanced Features Part 2 (2 weeks)
**Goal**: Atomics, threading, complex numbers

- [ ] DECY-XXX: Complex numbers (8 points)
- [ ] DECY-XXX: Atomic operations (8 points)
- [ ] DECY-XXX: Thread-local storage (5 points)
- [ ] DECY-XXX: goto statements (3 points)
- [ ] DECY-XXX: VLA full support (5 points)
- [ ] **Total**: 29 story points

**Acceptance Criteria**:
- C complex.h → num-complex crate
- C11 atomics → std::sync::atomic
- TLS → thread_local! macro

---

### Sprint 12: Cleanup & Polish (2 weeks)
**Goal**: Complete remaining low-priority items

- [ ] DECY-XXX: All remaining preprocessor tasks (16 points)
- [ ] DECY-XXX: All remaining C99/C11 tasks (10 points)
- [ ] DECY-XXX: Cast operators completion (3 points)
- [ ] **Total**: 29 story points

**Acceptance Criteria**:
- 95%+ K&R coverage achieved
- All documentation complete
- Real-world C project validation

---

## Success Metrics

### Target Coverage by Sprint

| Sprint | Constructs Complete | Coverage % | Cumulative Story Points |
|--------|---------------------|------------|-------------------------|
| Current (7) | 116/150 | 77% | 217 |
| Sprint 8 | 119/150 | 79% | 243 |
| Sprint 9 | 123/150 | 82% | 266 |
| Sprint 10 | 127/150 | 85% | 295 |
| Sprint 11 | 132/150 | 88% | 324 |
| Sprint 12 | 143/150 | **95%** ✅ | 353 |

### Quality Requirements (All Sprints)

- ✅ Coverage ≥80% (target: 90%+)
- ✅ Clippy warnings = 0
- ✅ SATD comments = 0
- ✅ Unsafe per 1000 LOC <5
- ✅ All tests passing
- ✅ Documentation complete

---

## Test Requirements Per Task

Each task must include:

1. **Unit Tests** (≥5 per feature)
   - Basic functionality
   - Edge cases
   - Error handling

2. **Property Tests** (≥3 per feature)
   - Invariants hold across random inputs
   - 256 cases per property minimum

3. **Documentation Tests** (≥2 per feature)
   - C code example
   - Expected Rust output
   - Explanation of transformation

4. **Integration Tests** (≥1 per feature)
   - End-to-end transpilation
   - Verification that generated Rust compiles
   - Unsafe block count validation

---

## Validation Strategy

### Book-Based Validation

For each construct:
1. Find example in K&R book (cite page/section)
2. Create test case with exact K&R code
3. Document expected Rust equivalent
4. Verify transpiled output matches
5. Check for STOP THE LINE conditions

### Reference Documentation

- **Primary**: K&R C (2nd Edition)
- **Secondary**: ISO C99 (ISO/IEC 9899:1999)
- **Tertiary**: ISO C11 (for modern features)
- **Validation**: GCC/Clang behavior for undefined cases

---

## Progress Tracking

### Weekly Updates

Update this checklist every Friday with:
- [ ] Constructs completed this week
- [ ] Coverage % change
- [ ] Unsafe block count change
- [ ] Blocked tasks (with reasons)
- [ ] Next week priorities

### Monthly Reviews

Monthly roadmap review (first Monday):
- [ ] Sprint completion percentage
- [ ] Velocity (story points/sprint)
- [ ] Quality metrics trends
- [ ] Adjust remaining sprints

---

## Notes

### Skipped Features (Out of Scope)

The following are intentionally NOT supported:

- **Inline Assembly**: Too platform-specific, use FFI
- **VLA in structs**: Safety issues, use Vec<T> instead
- **Nested functions**: GCC extension, not standard C
- **K&R style function declarations**: Deprecated, C89+ only

### Feature Flags

Some features may be behind feature flags:

- `unsafe-optimizations`: Allow more unsafe for performance
- `c11-support`: Enable C11-specific features
- `gcc-extensions`: Support GCC-specific extensions

---

## Summary

**Remaining Work**: 34 constructs, 158 story points
**Estimated Time**: 5 sprints (10 weeks)
**Target Completion**: Sprint 12 (95% coverage)
**Current Velocity**: ~30 story points/sprint

**Critical Path**:
1. Preprocessor (14 tasks) - Enables header file parsing
2. File I/O (1 task) - Completes Chapter 7
3. Advanced features (13 tasks) - Covers edge cases
4. C99/C11 features (5 tasks) - Modern C support

---

*Last Updated: 2025-10-19*
*Maintained by: Claude Code + PMAT*
*Methodology: EXTREME TDD + Toyota Way*
