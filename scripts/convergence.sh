#!/usr/bin/env bash
# DECY-191: Corpus convergence measurement
#
# Iterates validation/kr-c/chapter-{1..8}/*.c, transpiles each,
# attempts rustc compilation, reports per-chapter pass/fail/rate.
#
# Usage: ./scripts/convergence.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$PROJECT_DIR/validation/kr-c"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Build decy first
echo "Building decy..."
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
cargo build -p decy --quiet 2>/dev/null || cargo build -p decy
DECY="$PROJECT_DIR/target/debug/decy"

echo ""
echo "## Corpus Convergence Report"
echo ""
echo "| Chapter | Files | Transpile OK | Compile OK | Transpile Rate | Compile Rate |"
echo "|---------|-------|-------------|-----------|----------------|-------------|"

total_files=0
total_transpile_ok=0
total_compile_ok=0

for chapter_dir in "$CORPUS_DIR"/chapter-*; do
    [ -d "$chapter_dir" ] || continue

    chapter_name=$(basename "$chapter_dir")
    files=0
    transpile_ok=0
    compile_ok=0

    for c_file in "$chapter_dir"/*.c; do
        [ -f "$c_file" ] || continue
        files=$((files + 1))

        basename_file=$(basename "$c_file" .c)
        rs_file="$TEMP_DIR/${chapter_name}_${basename_file}.rs"

        # Try transpile
        if "$DECY" transpile "$c_file" -o "$rs_file" 2>/dev/null; then
            transpile_ok=$((transpile_ok + 1))

            # Try compile (metadata only - no binary needed)
            if rustc --edition 2021 --emit=metadata -o /dev/null "$rs_file" 2>/dev/null; then
                compile_ok=$((compile_ok + 1))
            fi
        fi
    done

    if [ "$files" -gt 0 ]; then
        transpile_rate=$(echo "scale=1; $transpile_ok * 100 / $files" | bc)
        compile_rate=$(echo "scale=1; $compile_ok * 100 / $files" | bc)
    else
        transpile_rate="0.0"
        compile_rate="0.0"
    fi

    echo "| $chapter_name | $files | $transpile_ok | $compile_ok | ${transpile_rate}% | ${compile_rate}% |"

    total_files=$((total_files + files))
    total_transpile_ok=$((total_transpile_ok + transpile_ok))
    total_compile_ok=$((total_compile_ok + compile_ok))
done

if [ "$total_files" -gt 0 ]; then
    overall_transpile=$(echo "scale=1; $total_transpile_ok * 100 / $total_files" | bc)
    overall_compile=$(echo "scale=1; $total_compile_ok * 100 / $total_files" | bc)
else
    overall_transpile="0.0"
    overall_compile="0.0"
fi

echo "| **Total** | **$total_files** | **$total_transpile_ok** | **$total_compile_ok** | **${overall_transpile}%** | **${overall_compile}%** |"
echo ""
echo "Generated: $(date -Iseconds)"
