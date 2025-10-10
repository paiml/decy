# Introduction

Welcome to the **DECY Verification Book** - a comprehensive testing and verification guide for the DECY C-to-Rust transpiler.

## What is DECY?

DECY is a production-grade C-to-Rust transpiler that automatically converts legacy C code into safe, idiomatic, fully-tested Rust code. Unlike traditional transpilers, DECY follows **EXTREME Test-Driven Development (TDD)** methodology with:

- **≥80% test coverage** maintained at ALL times
- **≥90% mutation testing score** as target
- **100% linting passing** continuously
- **Zero tolerance for technical debt**
- **Property-based testing** for correctness guarantees
- **Book-based verification** (this book!)

## Philosophy

> "If it's not tested in the book, it doesn't work."

This book is not just documentation - it's **executable verification**. Every code example in this book:

1. ✅ **Compiles** with `cargo build`
2. ✅ **Runs** with `cargo test`
3. ✅ **Lints clean** with `cargo clippy`
4. ✅ **Passes property tests** with randomized inputs
5. ✅ **Survives mutation testing** with ≥90% kill rate

## Inspired By

DECY's book-based verification is inspired by:

- **The Rust Book**: Runnable code examples in documentation
- **mdBook best practices**: Executable verification through documentation

## How to Use This Book

Each chapter demonstrates a component of the transpiler with:

1. **Explanation**: What the component does
2. **Examples**: Real C code → Rust transpilation
3. **Tests**: Unit, property, and mutation tests
4. **Verification**: Proof that it works correctly

All code blocks are tested automatically when you run:

```bash
mdbook test
```

## Quality Standards

This book enforces the same quality standards as the transpiler:

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Test Coverage | ≥80% | CI/CD blocks if <80% |
| Mutation Score | ≥90% | Reported in metrics |
| Clippy Warnings | 0 | Build fails on warnings |
| Property Tests | 100+ | Required for core logic |
| Doc Tests | 100% | All public APIs documented |

## Structure

The book is organized into sections:

### Methodology
Learn about EXTREME TDD, quality gates, property testing, and mutation testing.

### Core Components
Verify each component of the transpiler pipeline:
- Parser (C AST extraction)
- HIR (High-level IR)
- Dataflow analysis
- Ownership inference
- Borrow generation
- Lifetime analysis
- Code generation

### End-to-End Verification
Complete transpilation examples from C to Rust:
- Simple functions
- Pointer handling
- Ownership patterns
- Lifetime annotations
- Box transformations

### Real-World Examples
Transpilation of actual C codebases:
- Python source code (cpython)
- Git source code
- NumPy arrays
- SQLite B-tree implementation

### Quality Metrics
Measure and track quality:
- Test coverage reports
- Mutation testing scores
- Complexity analysis
- Safety verification

## Running the Examples

All examples in this book can be run locally:

```bash
# Clone the repository
git clone https://github.com/noahgift/decy
cd decy

# Build the book
mdbook build

# Test all code examples
mdbook test

# Serve locally with hot reload
mdbook serve
```

## Contributing

Found an issue? Code doesn't work? **That's a bug!**

This book is living documentation. If an example fails:

1. File an issue: https://github.com/noahgift/decy/issues
2. Fix the code (not the test!)
3. Submit a PR with the fix
4. Verify `mdbook test` passes

## Let's Begin

Ready to explore how DECY transpiles C to Rust with extreme quality?

→ [Start with EXTREME TDD Methodology](./methodology/extreme-tdd.md)
