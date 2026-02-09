#!/usr/bin/env bash
# DECY-195: Semantic equivalence validation
#
# For each C file in the validation corpus:
# 1. Compile and run with gcc → capture stdout + exit code
# 2. Transpile with decy, compile with rustc, run → capture stdout + exit code
# 3. Diff outputs
#
# Usage: ./scripts/validate-equivalence.sh [corpus_dir]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="${1:-$PROJECT_DIR/validation/kr-c}"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Build decy first
echo "Building decy..."
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
cargo build -p decy --quiet 2>/dev/null || cargo build -p decy
DECY="$PROJECT_DIR/target/debug/decy"

echo ""
echo "## Semantic Equivalence Report"
echo ""

total=0
equivalent=0
divergent=0
errors=0
divergence_details=""

# Find all C files with main() (only executable programs can be tested)
for c_file in $(find "$CORPUS_DIR" -name "*.c" -type f | sort); do
    # Skip files without main()
    if ! grep -q "int main" "$c_file" 2>/dev/null; then
        continue
    fi

    total=$((total + 1))
    basename_file=$(basename "$c_file" .c)
    relative_path="${c_file#$CORPUS_DIR/}"

    # 1. Compile with gcc and run
    gcc_bin="$TEMP_DIR/gcc_${basename_file}"
    gcc_stdout="$TEMP_DIR/gcc_${basename_file}.out"
    gcc_exit=0

    if ! gcc -std=c99 -o "$gcc_bin" "$c_file" -lm 2>/dev/null; then
        errors=$((errors + 1))
        continue
    fi

    timeout 5 "$gcc_bin" > "$gcc_stdout" 2>/dev/null || gcc_exit=$?

    # 2. Transpile with decy
    rs_file="$TEMP_DIR/decy_${basename_file}.rs"
    if ! "$DECY" transpile "$c_file" -o "$rs_file" 2>/dev/null; then
        errors=$((errors + 1))
        continue
    fi

    # 3. Compile with rustc and run
    rust_bin="$TEMP_DIR/rust_${basename_file}"
    rust_stdout="$TEMP_DIR/rust_${basename_file}.out"
    rust_exit=0

    if ! rustc --edition 2021 -o "$rust_bin" "$rs_file" 2>/dev/null; then
        errors=$((errors + 1))
        continue
    fi

    timeout 5 "$rust_bin" > "$rust_stdout" 2>/dev/null || rust_exit=$?

    # 4. Compare
    if diff -q "$gcc_stdout" "$rust_stdout" >/dev/null 2>&1 && [ "$gcc_exit" = "$rust_exit" ]; then
        equivalent=$((equivalent + 1))
    else
        divergent=$((divergent + 1))
        divergence_details="${divergence_details}\n- **${relative_path}**: exit ${gcc_exit} vs ${rust_exit} | stdout differs: $(! diff -q "$gcc_stdout" "$rust_stdout" >/dev/null 2>&1 && echo "yes" || echo "no")"
    fi
done

# Report
if [ "$total" -gt 0 ]; then
    rate=$(echo "scale=1; $equivalent * 100 / $total" | bc)
else
    rate="0.0"
fi

echo "| Metric | Value |"
echo "|--------|-------|"
echo "| Total Files | $total |"
echo "| Equivalent | $equivalent |"
echo "| Divergent | $divergent |"
echo "| Errors | $errors |"
echo "| Equivalence Rate | ${rate}% |"

if [ "$divergent" -gt 0 ]; then
    echo ""
    echo "### Divergences"
    echo -e "$divergence_details"
fi

echo ""
echo "Generated: $(date -Iseconds)"
