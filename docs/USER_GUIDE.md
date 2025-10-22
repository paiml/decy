# Decy User Guide

**Version**: 0.2.0
**Date**: 2025-10-22
**Target Audience**: C developers with basic Rust knowledge

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
   - [Prerequisites](#prerequisites)
   - [Installing Decy](#installing-decy)
   - [Verification](#verification)
3. [Quick Start](#quick-start)
   - [Your First Transpilation](#your-first-transpilation)
   - [Transpiling a Project](#transpiling-a-project)
   - [Checking Build Order](#checking-build-order)
4. [Common C-to-Rust Patterns](#common-c-to-rust-patterns)
   - [Pointers](#pointers)
   - [Arrays](#arrays)
   - [Structs](#structs)
   - [Functions](#functions)
   - [Control Flow](#control-flow)
   - [Macros](#macros)
5. [Troubleshooting](#troubleshooting)
   - [Parse Errors](#parse-errors)
   - [Compilation Errors](#compilation-errors)
   - [Performance Issues](#performance-issues)
   - [Cache Problems](#cache-problems)
6. [Performance Optimization](#performance-optimization)
   - [Understanding the Cache](#understanding-the-cache)
   - [Parallel Transpilation](#parallel-transpilation)
   - [Incremental Workflows](#incremental-workflows)
7. [Advanced Topics](#advanced-topics)
   - [Incremental Migration Strategy](#incremental-migration-strategy)
   - [FFI Boundaries](#ffi-boundaries)
   - [Manual Code Cleanup](#manual-code-cleanup)
   - [Cargo Integration](#cargo-integration)
8. [FAQ](#faq)
9. [Known Limitations](#known-limitations)
10. [Getting Help](#getting-help)

---

## Introduction

**Decy** is a C-to-Rust transpiler that generates safe, idiomatic Rust code with minimal `unsafe` blocks. It's designed for developers who want to:

- **Migrate legacy C codebases** to Rust incrementally
- **Learn Rust** by seeing how C patterns map to Rust idioms
- **Improve code safety** by leveraging Rust's ownership system
- **Maintain C/Rust interoperability** during migration

### What Decy Does Well

✅ **Basic C transpilation** - Functions, variables, control flow, structs
✅ **Pointer inference** - Automatically converts C pointers to `&T`, `&mut T`, or `Box<T>`
✅ **Single-file projects** - Perfect for learning and small utilities
✅ **Safe code generation** - Minimal unsafe blocks (<5 per 1000 LOC target)
✅ **Incremental transpilation** - Caching for 10-20x speedup on unchanged files

### Current Limitations (v0.2.0)

⚠️ **Multi-file projects** - `#include` directive support is limited (see [Known Limitations](#known-limitations))
⚠️ **C++ compatibility** - `extern "C"` guards not yet supported
⚠️ **Header-only libraries** - `.h` files not processed by default
⚠️ **Real-world readiness** - 40% (validated against production C projects)

**Honest Assessment**: Decy excels at transpiling simple-to-moderate C code. Complex production codebases may require preprocessing or workarounds. We're actively working on closing these gaps in Sprint 18+.

### Philosophy

Decy follows **EXTREME TDD** and **Toyota Way** principles:
- **Quality first** - 90%+ test coverage, zero warnings
- **Safety first** - Minimize unsafe code through ownership inference
- **Transparency** - Honest about capabilities and limitations
- **Continuous improvement** - Kaizen mindset for incremental progress

## Installation

### Prerequisites

Decy requires:
- **Rust** 1.75.0 or later
- **LLVM 14** and Clang development libraries
- **Git** (for cloning the repository)

### Platform-Specific Setup

#### Ubuntu/Debian Linux

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 14 and Clang
sudo apt-get update
sudo apt-get install -y llvm-14 llvm-14-dev libclang-14-dev clang-14

# Set environment variables
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
```

#### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install LLVM via Homebrew
brew install llvm@14

# Set environment variables
export LLVM_CONFIG_PATH="$(brew --prefix llvm@14)/bin/llvm-config"
export LIBCLANG_PATH="$(brew --prefix llvm@14)/lib"
```

#### Windows (WSL Recommended)

We recommend using Windows Subsystem for Linux (WSL) and following the Ubuntu instructions above.

### Installing Decy

#### Option 1: From Source (Recommended for v0.2.0)

```bash
# Clone the repository
git clone https://github.com/paiml/decy.git
cd decy

# One-command installation (installs everything!)
make install

# Verify installation
./scripts/verify-setup.sh
```

The `make install` command will:
- Install Rust toolchain components (rustfmt, clippy)
- Install cargo tools (cargo-llvm-cov, cargo-mutants, cargo-watch)
- Verify LLVM/Clang installation
- Build the decy workspace

#### Option 2: From crates.io (Future)

```bash
# Not yet published, coming in v0.3.0
cargo install decy
```

### Verification

Check that decy is installed correctly:

```bash
# From source build
cd decy
cargo build --release
./target/release/decy --version

# Should output: decy 0.2.0
```

### Troubleshooting Installation

**Issue**: `clang-sys` build fails with "libclang not found"
```bash
# Solution: Set LIBCLANG_PATH explicitly
export LIBCLANG_PATH=/usr/lib/llvm-14/lib  # Ubuntu/Debian
# or
export LIBCLANG_PATH="$(brew --prefix llvm@14)/lib"  # macOS
```

**Issue**: LLVM version mismatch
```bash
# Solution: Verify LLVM 14 is installed
llvm-config-14 --version  # Should show 14.x.x
```

For more installation help, see [INSTALL.md](../INSTALL.md) or [Getting Help](#getting-help).

## Quick Start

This 5-minute tutorial will get you transpiling C code to Rust.

### Your First Transpilation

**Step 1**: Create a simple C file

```bash
# Create hello.c
cat > hello.c << 'EOF'
#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(10, 20);
    printf("Result: %d\n", result);
    return 0;
}
EOF
```

**Step 2**: Transpile to Rust

```bash
cd decy
./target/release/decy transpile ../hello.c -o ../hello.rs
```

**Step 3**: View the generated Rust

```rust
// Generated Rust code (hello.rs)
fn add(mut a: i32, mut b: i32) -> i32 {
    return a + b;
}

fn main() {
    let mut result: i32 = add(10, 20);
    println!("Result: {}", result);
    std::process::exit(0);
}
```

**Step 4**: Compile and run the Rust code

```bash
rustc hello.rs -o hello
./hello
# Output: Result: 30
```

🎉 **Success!** You've transpiled your first C program to Rust!

### Transpiling a Project

For multi-file projects, use `transpile-project`:

**Step 1**: Create a small C project

```bash
mkdir my-c-project
cd my-c-project

# main.c
cat > main.c << 'EOF'
int multiply(int a, int b);

int main() {
    int result = multiply(5, 6);
    return result;
}
EOF

# math.c
cat > math.c << 'EOF'
int multiply(int a, int b) {
    return a * b;
}
EOF
```

**Step 2**: Transpile the entire project

```bash
cd ../decy
./target/release/decy transpile-project ../my-c-project -o ../my-rust-project
```

**Step 3**: Check what was generated

```bash
ls ../my-rust-project/
# Output: main.rs  math.rs
```

### Checking Build Order

Before transpiling, check the dependency order:

```bash
./target/release/decy check-project ../my-c-project
```

Output:
```
Project: ../my-c-project
Files found: 2

Build Order:
  1. math.c
  2. main.c

Dependencies detected:
  main.c depends on: math.c
```

### Understanding the Cache

Decy caches transpilation results for speed:

```bash
# First run: transpiles everything
time ./target/release/decy transpile-project ../my-c-project -o ../my-rust-project
# Takes ~2 seconds

# Second run: uses cache (10-20x faster!)
time ./target/release/decy transpile-project ../my-c-project -o ../my-rust-project
# Takes ~0.2 seconds

# View cache statistics
./target/release/decy cache-stats ../my-c-project
```

Output:
```
Cache Statistics for: ../my-c-project
Total files: 2
Cache hits: 2 (100%)
Cache misses: 0 (0%)
Cache size: 4.2 KB
```

### Next Steps

- Read [Common C-to-Rust Patterns](#common-c-to-rust-patterns) to see how C constructs map to Rust
- Check [Troubleshooting](#troubleshooting) if you encounter errors
- Learn about [Performance Optimization](#performance-optimization) for large projects

## Common C-to-Rust Patterns

[Content to be added...]

## Troubleshooting

[Content to be added...]

## Performance Optimization

[Content to be added...]

## Advanced Topics

[Content to be added...]

## FAQ

[Content to be added...]

## Known Limitations

[Content to be added...]

## Getting Help

[Content to be added...]
