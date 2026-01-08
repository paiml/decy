# Introduction

**Decy** is a C-to-Rust transpiler that generates safe, idiomatic Rust code with minimal `unsafe` blocks. Built with **EXTREME TDD** and **Toyota Way** principles, Decy aims to make C-to-Rust migration practical and maintainable.

## Key Features

✅ **Safety First**: Target <5 `unsafe` blocks per 1000 lines of code
✅ **Ownership Inference**: Automatically converts C pointers to Rust references
✅ **TDD Verified**: 95%+ test coverage, 100K+ property tests
✅ **Production Validated**: Tested against real-world C projects
✅ **Incremental Migration**: Work with mixed C/Rust codebases
✅ **System Include Support**: Automatic stdlib.h, string.h discovery

## Current Status

**Version**: 2.0.0
**Test Pass Rate**: 99.9%
**Test Coverage**: 95.1%
**Passing Tests**: 1,391

## What Works Well

- ✅ Single-file C programs with system includes
- ✅ Basic C constructs (functions, variables, control flow)
- ✅ Pointer-to-reference inference
- ✅ malloc/free → Vec/Box pattern detection (DECY-220)
- ✅ Fast incremental transpilation (10-20x with cache)
- ✅ System include discovery (stdlib.h, string.h, etc.)

## Current Limitations

- ⚠️ Some complex macros not supported
- ⚠️ Multi-file projects require explicit handling
- ⚠️ C++ not supported (use c2rust for C++)

**Honest Assessment**: Decy 2.0 handles most single-file C programs including those with system includes. Complex production codebases with heavy macro usage may require preprocessing.

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
