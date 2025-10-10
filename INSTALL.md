# Decy Installation Guide

Complete, reproducible installation instructions for all platforms.

## Quick Installation (Recommended)

```bash
cd /path/to/decy
make install
```

This single command installs everything:
- Rust toolchain (latest stable)
- LLVM 14 + Clang development libraries
- Development tools (cargo-llvm-cov, cargo-mutants)
- System dependencies

## Verification

After installation, verify everything is working:

```bash
./scripts/verify-setup.sh
```

This checks:
- ✅ Rust and Cargo versions
- ✅ rustfmt and clippy
- ✅ LLVM/Clang installation
- ✅ libclang library
- ✅ Environment variables
- ✅ Optional tools

## Platform-Specific Instructions

### Ubuntu/Debian

```bash
# Full installation
make install

# Or step-by-step:
make install-rust      # Install Rust toolchain
make install-llvm      # Install LLVM/Clang (requires sudo)
make install-tools     # Install development tools
```

**What gets installed:**
- `llvm-14-dev` - LLVM development libraries
- `libclang-14-dev` - Clang development libraries
- `clang-14` - Clang compiler
- `build-essential` - GCC and build tools
- `pkg-config` - Package configuration tool

**Environment variables added to ~/.bashrc:**
```bash
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
```

**Reload shell after installation:**
```bash
source ~/.bashrc
source ~/.cargo/env
```

### macOS

```bash
# Install Homebrew first (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Full installation
make install
```

**What gets installed:**
- LLVM via Homebrew
- Rust via rustup

**Environment variables added to ~/.zshrc:**
```bash
export PATH="/usr/local/opt/llvm/bin:$PATH"
export LDFLAGS="-L/usr/local/opt/llvm/lib"
export CPPFLAGS="-I/usr/local/opt/llvm/include"
```

**Reload shell after installation:**
```bash
source ~/.zshrc
source ~/.cargo/env
```

### RHEL/CentOS/Fedora

```bash
# Full installation
make install
```

**Note:** Uses `yum` on RHEL/CentOS, `dnf` on Fedora.

## Manual Installation

If you prefer to install manually:

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update stable
rustup default stable
rustup component add rustfmt clippy llvm-tools-preview
```

### 2. Install LLVM/Clang

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    llvm-14-dev \
    libclang-14-dev \
    clang-14 \
    build-essential \
    pkg-config

# Add to ~/.bashrc
echo 'export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14' >> ~/.bashrc
echo 'export LIBCLANG_PATH=/usr/lib/llvm-14/lib' >> ~/.bashrc
source ~/.bashrc
```

**macOS:**
```bash
brew install llvm

# Add to ~/.zshrc
echo 'export PATH="/usr/local/opt/llvm/bin:$PATH"' >> ~/.zshrc
echo 'export LDFLAGS="-L/usr/local/opt/llvm/lib"' >> ~/.zshrc
echo 'export CPPFLAGS="-I/usr/local/opt/llvm/include"' >> ~/.zshrc
source ~/.zshrc
```

### 3. Install Development Tools

```bash
cargo install cargo-llvm-cov
cargo install cargo-mutants
cargo install cargo-watch
cargo install cargo-edit
```

### 4. Verify Installation

```bash
./scripts/verify-setup.sh
```

## Troubleshooting

### Error: "couldn't find libclang"

**Solution:** Set the LIBCLANG_PATH environment variable:

```bash
# Ubuntu/Debian
export LIBCLANG_PATH=/usr/lib/llvm-14/lib

# macOS
export LIBCLANG_PATH=/usr/local/opt/llvm/lib

# Add to shell config
echo "export LIBCLANG_PATH=<path>" >> ~/.bashrc  # or ~/.zshrc
```

### Error: "llvm-config not found"

**Solution:** Set the LLVM_CONFIG_PATH environment variable:

```bash
# Ubuntu/Debian
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14

# macOS
export LLVM_CONFIG_PATH=/usr/local/opt/llvm/bin/llvm-config

# Add to shell config
echo "export LLVM_CONFIG_PATH=<path>" >> ~/.bashrc  # or ~/.zshrc
```

### Error: "clang-sys build failed"

**Common causes:**
1. LLVM not installed → Run `make install-llvm`
2. Environment variables not set → Run `source ~/.bashrc` or `source ~/.zshrc`
3. Wrong LLVM version → Ensure LLVM 14+ is installed

**Full fix:**
```bash
# Reinstall LLVM
make install-llvm

# Reload shell
source ~/.bashrc
source ~/.cargo/env

# Clean and rebuild
make clean
make build
```

### Build is slow or fails with timeout

**Solution:** Increase parallel compilation:

```bash
# Set number of parallel jobs
export CARGO_BUILD_JOBS=4

# Or reduce jobs if running out of memory
export CARGO_BUILD_JOBS=1
```

### Permission denied errors

**Solution:** Ensure scripts are executable:

```bash
chmod +x scripts/*.sh
chmod +x .git/hooks/pre-commit
```

## Docker Installation (Alternative)

For a completely reproducible environment:

```dockerfile
# Coming soon: Dockerfile for Decy development
```

## Continuous Integration

The CI pipeline (`.github/workflows/quality.yml`) shows the exact steps used in automated builds. Reference this for CI/CD setup.

## Next Steps

After successful installation:

1. **Build the project:**
   ```bash
   make build
   ```

2. **Run tests:**
   ```bash
   make test
   ```

3. **Check quality gates:**
   ```bash
   make quality-gates
   ```

4. **Read the documentation:**
   - [GETTING_STARTED.md](GETTING_STARTED.md) - Development guide
   - [README.md](README.md) - Project overview
   - [docs/specifications/decy-spec-v1.md](docs/specifications/decy-spec-v1.md) - Full specification

## Support

If you encounter issues:

1. Run verification: `./scripts/verify-setup.sh`
2. Check environment variables: `env | grep -E "(LLVM|CLANG)"`
3. Check LLVM installation: `llvm-config --version`
4. Open an issue with the output of the above commands

## Make Targets Reference

```bash
make install           # Complete installation
make install-rust      # Install Rust only
make install-llvm      # Install LLVM/Clang only
make install-tools     # Install dev tools only
make verify-install    # Verify installation
make build             # Build project
make test              # Run tests
make quality-gates     # Run quality checks
make help              # Show all targets
```

---

**Everything is reproducible. No manual configuration required.** ✅
