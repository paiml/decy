# Decy: C-to-Rust Transpiler with EXTREME Quality Standards

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

Decy is a production-grade C-to-Rust transpiler that generates idiomatic, safe Rust code with minimal `unsafe` blocks (<5 per 1000 LOC). Built using EXTREME TDD methodology, Toyota Way principles, and PMAT qualification.

## 🎯 Project Goals

- **Transpile legacy C projects** (CPython, Git, NumPy, SQLite) to safe, idiomatic Rust
- **Minimize unsafe code** through advanced ownership and lifetime inference
- **Maintain EXTREME quality standards**: 80%+ coverage, 90%+ mutation score, 0 warnings
- **Verify correctness** using book-based testing (mdBook compilation + lint checks)

## 🚀 Quick Start

### One-Command Installation

Everything is automated for reproducibility:

```bash
# Clone repository
git clone https://github.com/your-org/decy.git
cd decy

# Install EVERYTHING (Rust, LLVM/Clang, tools)
make install

# Verify installation
./scripts/verify-setup.sh
```

The `make install` command installs:
- ✅ Rust toolchain (latest stable)
- ✅ LLVM 14 + Clang development libraries
- ✅ rustfmt, clippy, llvm-tools-preview
- ✅ cargo-llvm-cov, cargo-mutants, cargo-watch
- ✅ All required system dependencies

### Build and Test

```bash
# Build workspace
make build

# Run tests
make test

# Run quality checks
make quality-gates

# See all commands
make help
```

### Basic Usage

```bash
# Transpile a single C file
decy transpile input.c -o output.rs

# Transpile an entire GitHub repository
decy transpile-repo https://github.com/python/cpython --output ./cpython-rust

# Start the MCP server (for Claude Code integration)
decy mcp-server

# Generate verification book
decy verify --book-output ./book
```

## 📚 Documentation

- **[Getting Started](GETTING_STARTED.md)** - Comprehensive guide for new developers
- **[Specification](docs/specifications/decy-spec-v1.md)** - Complete technical specification
- **[Unsafe Minimization Strategy](docs/specifications/decy-unsafe-minimization-strategy.md)** - How we reduce unsafe code
- **[Roadmap](roadmap.yaml)** - 20-sprint development plan with detailed tickets
- **[Quality Configuration](decy-quality.toml)** - Quality gates and enforcement rules

## 🏗️ Architecture

Decy uses a multi-stage transpilation pipeline:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   C Parser  │────▶│     HIR     │────▶│  Analyzer   │────▶│  Ownership  │
│ (clang-sys) │     │ (Rust-IR)   │     │  (Types)    │     │  Inference  │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                     │
                                                                     ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    Book     │◀────│   Codegen   │◀────│   Verify    │◀────│  Lifetime   │
│ Verification│     │ (Rust code) │     │  (Safety)   │     │  Inference  │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### Key Components

- **decy-parser**: C AST parsing using LLVM/Clang bindings
- **decy-hir**: High-level Intermediate Representation (Rust-oriented)
- **decy-analyzer**: Static analysis and type inference
- **decy-ownership**: Ownership inference (CRITICAL - converts pointers to &T/&mut T/Box/Vec)
- **decy-verify**: Safety property verification
- **decy-codegen**: Rust code generation with minimal unsafe
- **decy-book**: Book-based verification (mdBook + compile + lint)
- **decy-agent**: Background daemon for incremental transpilation
- **decy-mcp**: MCP server for Claude Code integration
- **decy-repo**: GitHub repository transpilation with parallel processing
- **decy**: CLI tool

## 🧪 Testing Philosophy

Decy follows a comprehensive 4-tier testing approach:

1. **Unit Tests** (85% coverage target) - Per-function testing
2. **Property Tests** (100+ properties × 1000 cases) - Randomized edge case discovery
3. **Mutation Tests** (90%+ kill rate) - Test quality verification
4. **Integration Tests** - End-to-end pipeline validation

### Running Tests

```bash
# All tests
cargo test --workspace

# Unit tests only
cargo test --lib

# Property tests
cargo test --features proptest-tests

# Mutation tests (requires cargo-mutants)
cargo mutants --workspace

# Coverage report
cargo llvm-cov --workspace --html
```

## 🔒 Quality Standards

Decy enforces EXTREME quality standards at all times:

| Metric | Requirement | Enforced By |
|--------|-------------|-------------|
| Test Coverage | ≥80% | Pre-commit hook + CI |
| Mutation Kill Rate | ≥90% | CI (by Sprint 5) |
| Clippy Warnings | 0 | Pre-commit hook + CI |
| SATD Comments | 0 (TODO/FIXME/HACK) | Pre-commit hook + CI |
| Cyclomatic Complexity | ≤10 per function | Code review |
| Unsafe Blocks | <5 per 1000 LOC | Metrics tracking |

### Pre-Commit Quality Gates

All commits must pass quality gates:

```bash
# Automatically runs on 'git commit'
# Or run manually:
./scripts/quality-gates.sh
```

Quality gates check:
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy -- -D warnings`)
- ✅ Tests (`cargo test --workspace`)
- ✅ Coverage (≥80%)
- ✅ SATD comments (zero tolerance)
- ✅ Documentation builds

## 🎓 Development Methodology

### EXTREME TDD Workflow

Every ticket follows the RED-GREEN-REFACTOR cycle:

```bash
# RED Phase: Write failing tests
git commit -m "[RED] DECY-XXX: Add failing tests"

# GREEN Phase: Minimal implementation
git commit -m "[GREEN] DECY-XXX: Implement feature"

# REFACTOR Phase: Meet quality gates
git commit -m "[REFACTOR] DECY-XXX: Clean up and optimize"

# Final: Squash into atomic commit
git rebase -i HEAD~3
git commit -m "DECY-XXX: Feature description

- Coverage: 85% ✅
- Mutation score: 92% ✅
- Quality grade: A ✅

Closes #XXX"
```

### Toyota Way Principles

- **Jidoka (自働化)**: Build quality in - never merge incomplete features
- **Genchi Genbutsu (現地現物)**: Direct observation - test with real C code
- **Kaizen (改善)**: Continuous improvement - fix bugs before features
- **Hansei (反省)**: Reflection after each sprint on quality metrics

## 📊 Current Status

**Sprint**: 0 (Initialization) - COMPLETED ✅
**Version**: 0.1.0
**Coverage**: 95.68% ✅ (Target: ≥80%)
**Total Tests**: 191 unit + 6 integration + 400+ property = 597+ test cases
**Mutation Score**: N/A (Will implement in Sprint 5)
**Next Milestone**: Sprint 1 - Foundation & C Parser

### Recent Achievements

✅ **DECY-009: Malloc-to-Box Transformation Pipeline** (5 phases completed)
- Phase 1: Function call support in HIR
- Phase 2: Assignment statement support in HIR
- Phase 3: Pattern detection for malloc/free
- Phase 4: Box::new() code generation
- Phase 5: Box<T> type generation

**Impact**:
- Successfully transforms unsafe malloc/free patterns → safe Box<T>
- Eliminates manual memory management
- Provides memory safety through RAII
- Generated code is idiomatic and safe

**Documentation**:
- [Malloc-to-Box Transformation Guide](docs/malloc-to-box-transformation.md)
- [Interactive Example](crates/decy-codegen/examples/malloc_to_box.rs)
- Comprehensive integration tests

### Sprint 1 Goals (Weeks 1-2)

- [x] DECY-002: Define HIR structure ✅
- [x] DECY-003: Implement basic code generator ✅
- [x] DECY-009: Malloc-to-Box transformation ✅
- [ ] DECY-001: Setup clang-sys integration (In Progress)

See [roadmap.yaml](roadmap.yaml) for the complete 20-sprint plan.

## 🎯 Target Projects

Decy aims to successfully transpile these real-world C projects:

| Project | LOC | Priority | Target Sprint |
|---------|-----|----------|---------------|
| CPython | 500K | High | Sprint 20 |
| Git | 200K | High | Sprint 15 |
| NumPy | 100K | Medium | Sprint 12 |
| SQLite | 150K | Medium | Sprint 18 |

## 🛠️ MCP Integration

Decy provides an MCP server for Claude Code integration:

```bash
# Start MCP server
decy mcp-server --port 3000

# Available tools:
# - transpile_file: Transpile a single C file
# - transpile_function: Transpile a C function
# - analyze_ownership: Analyze pointer ownership
# - suggest_refactoring: Suggest safe Rust patterns
# - verify_safety: Verify safety properties
# - generate_book: Generate verification book
```

## 📈 Unsafe Code Reduction Strategy

Decy minimizes unsafe code through a 4-phase approach:

1. **Phase 1: Pattern-Based** (100% → 50%) - Detect malloc/free → Box, arrays → Vec
2. **Phase 2: Ownership Inference** (50% → 20%) - Infer &T, &mut T from pointer usage
3. **Phase 3: Lifetime Inference** (20% → 10%) - Infer lifetime annotations
4. **Phase 4: Safe Wrappers** (10% → <5%) - Generate safe abstractions

See [docs/specifications/decy-unsafe-minimization-strategy.md](docs/specifications/decy-unsafe-minimization-strategy.md) for details.

## 🤝 Contributing

We welcome contributions! Please see:

- [GETTING_STARTED.md](GETTING_STARTED.md) - Development guide

### Development Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/decy.git
cd decy

# Install pre-commit hooks
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
# (Already done if you cloned - hook is committed)

# Create feature branch
git checkout -b feature/DECY-XXX-description

# Make changes following EXTREME TDD
# Run quality gates before committing
./scripts/quality-gates.sh

# Commit and push
git commit -m "DECY-XXX: Description"
git push origin feature/DECY-XXX-description
```

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

Decy is inspired by:

- **Depyler**: Python-to-Rust transpiler with book verification
- **bashrs**: Rust-to-Shell transpiler with EXTREME quality gates
- **paiml-mcp-agent-toolkit**: PMAT qualification framework
- **C2Rust**: Mozilla's C-to-Rust transpiler (we aim to improve on unsafe code generation)
- **Toyota Production System**: Principles of quality and continuous improvement

## 📞 Contact & Support

- **Documentation**: [docs/](docs/)
- **Roadmap**: [roadmap.yaml](roadmap.yaml)

---

**Built with EXTREME quality standards. Zero compromises. 🚀**
