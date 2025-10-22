# Introduction

**Decy** is a C-to-Rust transpiler that generates safe, idiomatic Rust code with minimal `unsafe` blocks. Built with **EXTREME TDD** and **Toyota Way** principles, Decy aims to make C-to-Rust migration practical and maintainable.

## Key Features

✅ **Safety First**: Target <5 `unsafe` blocks per 1000 lines of code  
✅ **Ownership Inference**: Automatically converts C pointers to Rust references  
✅ **TDD Verified**: 90%+ test coverage, 100K+ property tests  
✅ **Production Validated**: Tested against real-world C projects  
✅ **Incremental Migration**: Work with mixed C/Rust codebases  

## Current Status

**Version**: 0.2.0  
**Real-World Readiness**: 40% (Sprint 17)  
**Test Coverage**: 89.83%  
**Passing Tests**: 613  

## What Works Well

- ✅ Single-file C programs
- ✅ Basic C constructs (functions, variables, control flow)
- ✅ Pointer-to-reference inference
- ✅ malloc/free → Box pattern detection
- ✅ Fast incremental transpilation (10-20x with cache)

## Current Limitations

- ⚠️ `#include` directives (P0 blocker - Sprint 18)
- ⚠️ `extern "C"` guards (P1 - Sprint 18)
- ⚠️ Multi-file projects require workarounds
- ⚠️ Some complex macros not supported

**Honest Assessment**: Decy excels at transpiling simple-to-moderate C code. Complex production codebases may require preprocessing. We're actively working on closing these gaps.

## Who Should Use Decy

**Good Fit**:
- Learning Rust by seeing C patterns mapped to Rust
- Migrating small C utilities to Rust
- Prototyping C-to-Rust conversions
- Single-file C programs

**Not Ready Yet**:
- Large multi-file C projects (without preprocessing)
- C++ codebases
- Production migration without manual review

## Philosophy

Decy follows **EXTREME TDD** and the **Toyota Way**:

- **Quality First**: Zero defects, high coverage, continuous testing
- **Honesty**: Transparent about capabilities and limitations
- **Kaizen**: Continuous improvement through measured progress
- **Jidoka**: Build quality in at each stage

## How This Book Works

**TDD-Enforced Examples**: Every code example in this book is tested! All Rust code blocks are compiled and tested as part of our CI pipeline. This ensures:

1. **Accuracy**: Examples actually work
2. **Maintenance**: Examples stay up-to-date with the codebase
3. **Confidence**: You can trust the code you see

When you see code like this:

```rust
// This compiles and runs!
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 2), 4);
}
```

It's guaranteed to compile and pass tests, or our CI fails and prevents release.

## Next Steps

- [Installation](./installation.md) - Get Decy installed
- [Quick Start](./quick-start.md) - Transpile your first C program
- [First Transpilation](./first-transpilation.md) - Detailed walkthrough

Ready? Let's begin! →
