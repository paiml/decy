#!/usr/bin/env bash
# Decy Setup Verification Script
# Verifies that all dependencies are correctly installed

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FAILED=0

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Decy Setup Verification${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 1. Check Rust installation
echo -e "${BLUE}[1/8] Checking Rust installation...${NC}"
if command -v rustc >/dev/null 2>&1; then
    VERSION=$(rustc --version)
    echo -e "${GREEN}✅ Rust installed: ${VERSION}${NC}"
else
    echo -e "${RED}❌ Rust not found${NC}"
    echo "   Install with: make install-rust"
    FAILED=1
fi

# 2. Check Cargo
echo -e "${BLUE}[2/8] Checking Cargo...${NC}"
if command -v cargo >/dev/null 2>&1; then
    VERSION=$(cargo --version)
    echo -e "${GREEN}✅ Cargo installed: ${VERSION}${NC}"
else
    echo -e "${RED}❌ Cargo not found${NC}"
    FAILED=1
fi

# 3. Check rustfmt
echo -e "${BLUE}[3/8] Checking rustfmt...${NC}"
if command -v rustfmt >/dev/null 2>&1; then
    VERSION=$(rustfmt --version)
    echo -e "${GREEN}✅ rustfmt installed: ${VERSION}${NC}"
else
    echo -e "${RED}❌ rustfmt not found${NC}"
    echo "   Install with: rustup component add rustfmt"
    FAILED=1
fi

# 4. Check clippy
echo -e "${BLUE}[4/8] Checking clippy...${NC}"
if cargo clippy --version >/dev/null 2>&1; then
    VERSION=$(cargo clippy --version)
    echo -e "${GREEN}✅ clippy installed: ${VERSION}${NC}"
else
    echo -e "${RED}❌ clippy not found${NC}"
    echo "   Install with: rustup component add clippy"
    FAILED=1
fi

# 5. Check LLVM/Clang
echo -e "${BLUE}[5/8] Checking LLVM/Clang...${NC}"

# Set environment variables for Debian/Ubuntu
if [ -f /etc/debian_version ]; then
    export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
    export LIBCLANG_PATH=/usr/lib/llvm-14/lib
fi

LLVM_FOUND=0
if command -v llvm-config-14 >/dev/null 2>&1; then
    VERSION=$(llvm-config-14 --version)
    echo -e "${GREEN}✅ llvm-config-14: ${VERSION}${NC}"
    LLVM_FOUND=1
elif command -v llvm-config >/dev/null 2>&1; then
    VERSION=$(llvm-config --version)
    echo -e "${GREEN}✅ llvm-config: ${VERSION}${NC}"
    LLVM_FOUND=1
else
    echo -e "${RED}❌ llvm-config not found${NC}"
    FAILED=1
fi

CLANG_FOUND=0
if command -v clang-14 >/dev/null 2>&1; then
    VERSION=$(clang-14 --version | head -1)
    echo -e "${GREEN}✅ clang-14: ${VERSION}${NC}"
    CLANG_FOUND=1
elif command -v clang >/dev/null 2>&1; then
    VERSION=$(clang --version | head -1)
    echo -e "${GREEN}✅ clang: ${VERSION}${NC}"
    CLANG_FOUND=1
else
    echo -e "${RED}❌ clang not found${NC}"
    FAILED=1
fi

if [ $LLVM_FOUND -eq 0 ] || [ $CLANG_FOUND -eq 0 ]; then
    echo "   Install with: make install-llvm"
fi

# 6. Check libclang
echo -e "${BLUE}[6/8] Checking libclang library...${NC}"

LIBCLANG_FOUND=0
if [ -f /etc/debian_version ]; then
    if [ -f /usr/lib/llvm-14/lib/libclang.so ] || [ -f /usr/lib/x86_64-linux-gnu/libclang-14.so.1 ]; then
        echo -e "${GREEN}✅ libclang found in /usr/lib/llvm-14/lib${NC}"
        LIBCLANG_FOUND=1
    fi
elif [ "$(uname)" = "Darwin" ]; then
    if [ -f /usr/local/opt/llvm/lib/libclang.dylib ]; then
        echo -e "${GREEN}✅ libclang found in /usr/local/opt/llvm/lib${NC}"
        LIBCLANG_FOUND=1
    fi
else
    # Try to find libclang anywhere
    if ldconfig -p 2>/dev/null | grep -q libclang; then
        echo -e "${GREEN}✅ libclang found in system${NC}"
        LIBCLANG_FOUND=1
    fi
fi

if [ $LIBCLANG_FOUND -eq 0 ]; then
    echo -e "${RED}❌ libclang not found${NC}"
    echo "   Install with: make install-llvm"
    FAILED=1
fi

# 7. Check optional tools
echo -e "${BLUE}[7/8] Checking optional tools...${NC}"

if command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo -e "${GREEN}✅ cargo-llvm-cov installed${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-llvm-cov not installed (optional)${NC}"
    echo "   Install with: cargo install cargo-llvm-cov"
fi

if command -v cargo-mutants >/dev/null 2>&1; then
    echo -e "${GREEN}✅ cargo-mutants installed${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-mutants not installed (optional)${NC}"
    echo "   Install with: cargo install cargo-mutants"
fi

if command -v cargo-watch >/dev/null 2>&1; then
    echo -e "${GREEN}✅ cargo-watch installed${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-watch not installed (optional)${NC}"
    echo "   Install with: cargo install cargo-watch"
fi

# 8. Check environment variables
echo -e "${BLUE}[8/8] Checking environment variables...${NC}"

if [ -f /etc/debian_version ]; then
    if [ -n "$LLVM_CONFIG_PATH" ]; then
        echo -e "${GREEN}✅ LLVM_CONFIG_PATH set: $LLVM_CONFIG_PATH${NC}"
    else
        echo -e "${YELLOW}⚠️  LLVM_CONFIG_PATH not set${NC}"
        echo "   Add to ~/.bashrc: export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14"
    fi

    if [ -n "$LIBCLANG_PATH" ]; then
        echo -e "${GREEN}✅ LIBCLANG_PATH set: $LIBCLANG_PATH${NC}"
    else
        echo -e "${YELLOW}⚠️  LIBCLANG_PATH not set${NC}"
        echo "   Add to ~/.bashrc: export LIBCLANG_PATH=/usr/lib/llvm-14/lib"
    fi
fi

# Final summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All required dependencies are installed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Reload shell: source ~/.bashrc  (or source ~/.cargo/env)"
    echo "  2. Build project: make build"
    echo "  3. Run tests: make test"
    echo "  4. Quality check: make quality-gates"
else
    echo -e "${RED}❌ Some dependencies are missing${NC}"
    echo ""
    echo "To install all dependencies:"
    echo "  make install"
    echo ""
    echo "Or install individually:"
    echo "  make install-rust    # Install Rust toolchain"
    echo "  make install-llvm    # Install LLVM/Clang"
    echo "  make install-tools   # Install development tools"
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

exit $FAILED
