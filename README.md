<div align="center">

<img src=".github/decy-hero.svg" alt="decy" width="800">

<p align="center">
  <a href="https://crates.io/crates/decy"><img src="https://img.shields.io/crates/v/decy.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/decy"><img src="https://docs.rs/decy/badge.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License"></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/tests-99.9%25%20passing-brightgreen.svg" alt="Tests">
  <img src="https://img.shields.io/badge/coverage-95%25-purple.svg" alt="Coverage">
  <img src="https://img.shields.io/badge/PMAT-A+-blue.svg" alt="PMAT Score">
  <img src="https://img.shields.io/badge/repo%20health-84%2F100-green.svg" alt="Repo Health">
</p>

</div>

---

## What's New in 2.0

**Release Date:** January 2025

- **99.9% Test Pass Rate** - 1,391 tests passing
- **95% Code Coverage** - Comprehensive test suite
- **DECY-220**: Fixed malloc cast expression handling (`(int*)malloc(n)` → `Vec<T>`)
- **Portable Tests**: All test paths use `CARGO_MANIFEST_DIR`
- **System Include Discovery**: Automatic detection of `stdlib.h`, `string.h`

```bash
cargo install decy
```

---

## Quick Start

```bash
# Transpile a C file to Rust
decy transpile input.c -o output.rs

# Transpile an entire project
decy transpile-project src/ -o rust_output/

# Audit unsafe code
decy audit output.rs
```

**Example:**

```c
// input.c
int add(int a, int b) {
    return a + b;
}
```

```bash
decy transpile input.c
```

```rust
// Generated Rust (no unsafe!)
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## Quality Metrics (PMAT)

| Metric | Score | Target |
|--------|-------|--------|
| **Rust Project Score** | 92.9% (A+) | 90%+ |
| **Repository Health** | 84.5/100 (B+) | 80+ |
| **Test Coverage** | 95.1% | 80%+ |
| **Test Pass Rate** | 99.9% | 100% |
| **Clippy Warnings** | 0 | 0 |

Run quality analysis:

```bash
pmat rust-project-score
pmat repo-score
pmat analyze complexity
```

---

## Installation

### From crates.io (Recommended)

```bash
cargo install decy
```

### From Source

```bash
git clone https://github.com/paiml/decy.git
cd decy
make install   # Installs Rust + LLVM/Clang
cargo install --path crates/decy
```

### Requirements

- **Rust**: 1.70+ (stable)
- **LLVM/Clang**: 14+ (for C parsing)
- **Platform**: Linux, macOS, Windows (WSL2)

---

## Features

### Core Transpilation

```bash
# Single file
decy transpile input.c -o output.rs

# Project with caching (10-20x faster on unchanged files)
decy transpile-project src/ -o rust_output/
decy cache-stats src/
```

### Debug & Visualization

```bash
# Visualize C AST
decy debug --visualize-ast input.c

# Visualize ownership inference
decy debug --visualize-ownership input.c

# Step-through debugging
decy debug --step-through input.c
```

### Safety Analysis

```bash
# Audit unsafe blocks
decy audit output.rs --verbose

# Generate verification book
decy verify --book-output ./book
```

### MCP Integration

```bash
# Start MCP server for Claude Code
decy mcp-server --port 3000
```

---

## Architecture

```
C Source → Parser → HIR → Analyzer → Ownership → Codegen → Rust
             │         │       │          │          │
           clang    Rust-IR  Types   &T/&mut T    Safe code
```

### Crates

| Crate | Description |
|-------|-------------|
| `decy-parser` | C AST parsing (clang-sys) |
| `decy-hir` | High-level IR (Rust-oriented) |
| `decy-analyzer` | Static analysis, type inference |
| `decy-ownership` | Ownership inference (pointers → references) |
| `decy-codegen` | Rust code generation |
| `decy-verify` | Safety verification |
| `decy-debugger` | AST/HIR visualization |
| `decy` | CLI binary |

---

## Unsafe Minimization

Decy uses a 4-phase approach to minimize unsafe code:

| Phase | Reduction | Technique |
|-------|-----------|-----------|
| 1. Pattern-Based | 100% → 50% | `malloc/free` → `Box`, arrays → `Vec` |
| 2. Ownership | 50% → 20% | Infer `&T`, `&mut T` from usage |
| 3. Lifetime | 20% → 10% | Infer `<'a, 'b>` annotations |
| 4. Safe Wrappers | 10% → <5% | Generate safe abstractions |

**Target:** <5 unsafe blocks per 1000 LOC

---

## Development

### EXTREME TDD Workflow

```bash
# RED: Write failing tests
git commit -m "[RED] DECY-XXX: Add failing tests"

# GREEN: Minimal implementation
git commit -m "[GREEN] DECY-XXX: Implement feature"

# REFACTOR: Meet quality gates
git commit -m "[REFACTOR] DECY-XXX: Clean up"
```

### Quality Gates

```bash
make quality-gates   # Run all checks
make test            # Run tests
make coverage        # Generate coverage report
```

### Running Tests

```bash
cargo test --workspace          # All tests
cargo test -p decy-ownership    # Single crate
cargo llvm-cov --workspace      # Coverage
```

---

## Documentation

- **[Getting Started](GETTING_STARTED.md)** - Developer guide
- **[Specification](docs/specifications/decy-spec-v1.md)** - Technical spec
- **[Unsafe Strategy](docs/specifications/decy-unsafe-minimization-strategy.md)** - How we reduce unsafe
- **[Roadmap](roadmap.yaml)** - Development plan

---

## License

MIT OR Apache-2.0

---

## Acknowledgments

- **[C2Rust](https://github.com/immunant/c2rust)** - Mozilla's C-to-Rust transpiler
- **[PMAT](https://github.com/paiml/pmat)** - Quality metrics toolkit
- **Toyota Production System** - Quality principles

---

<div align="center">
<b>Built with EXTREME quality standards</b>
</div>
