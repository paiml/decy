# Installation

## From crates.io (Recommended)

```bash
cargo install decy
```

## From Source

```bash
git clone https://github.com/paiml/decy.git
cd decy
make install
cargo install --path crates/decy
```

## Verify Installation

```bash
decy --version
# decy 2.0.0
```

## Requirements

- **Rust**: 1.70+ (stable)
- **LLVM/Clang**: 14+ (for C parsing)
- **Platform**: Linux, macOS, Windows (WSL2)

The `make install` command automatically installs LLVM dependencies on supported platforms.

Next: [Quick Start](./quick-start.md)
