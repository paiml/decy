# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./installation.md)
- [Quick Start](./quick-start.md)
- [Your First Transpilation](./first-transpilation.md)

# Core Concepts

- [How Decy Works](./how-it-works.md)
- [The Transpilation Pipeline](./pipeline.md)
  - [Parser (C AST)](./pipeline/parser.md)
  - [HIR (High-level IR)](./pipeline/hir.md)
  - [Ownership Inference](./pipeline/ownership.md)
  - [Code Generation](./pipeline/codegen.md)
- [Ownership & Safety](./ownership-safety.md)

# C-to-Rust Patterns

- [Pointers to References](./patterns/pointers.md)
- [Arrays and Slices](./patterns/arrays.md)
- [Structs and Enums](./patterns/structs.md)
- [Functions](./patterns/functions.md)
- [Control Flow](./patterns/control-flow.md)
- [Memory Management](./patterns/memory.md)
- [String Safety](./patterns/string-safety.md)
- [Loop + Array Safety](./patterns/loop-array-safety.md)
- [Dynamic Memory Safety](./patterns/dynamic-memory-safety.md)
- [Pointer Arithmetic Safety](./patterns/pointer-arithmetic-safety.md)
- [Type Casting Safety](./patterns/type-casting-safety.md)
- [NULL Pointer Safety](./patterns/null-pointer-safety.md)
- [Integer Overflow Safety](./patterns/integer-overflow-safety.md)
- [Buffer Overflow Safety](./patterns/buffer-overflow-safety.md)
- [Use-After-Free Safety](./patterns/use-after-free-safety.md)
- [Uninitialized Memory Safety](./patterns/uninitialized-memory-safety.md)
- [Format String Safety](./patterns/format-string-safety.md)

# Advanced Topics

- [Multi-file Projects](./advanced/multi-file.md)
- [Incremental Migration](./advanced/migration.md)
- [FFI Boundaries](./advanced/ffi.md)
- [Cache System](./advanced/cache.md)
- [Debugging](./advanced/debugging.md)

# Reference

- [CLI Reference](./reference/cli.md)
- [Configuration](./reference/config.md)
- [Known Limitations](./reference/limitations.md)
- [Troubleshooting](./reference/troubleshooting.md)

# Development

- [Contributing](./development/contributing.md)
- [Architecture](./development/architecture.md)
- [Testing](./development/testing.md)
- [Release Process](./development/releases.md)

---

[Appendix: C99 Validation](./appendix-c99.md)
[Appendix: Unsafe Minimization](./appendix-unsafe.md)
