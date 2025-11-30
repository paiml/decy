#!/bin/bash
# Transpile all C examples to Rust using DECY
# This script finds all .c files and attempts transpilation

set -e

DECY_BIN="cargo run --bin decy --"
EXAMPLES_DIR="examples"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  DECY Example Transpiler - Batch Transpilation            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Find all C files
C_FILES=$(find "$EXAMPLES_DIR" -name "*.c" | sort)
TOTAL=$(echo "$C_FILES" | wc -l)
SUCCESS=0
FAIL=0

echo "Found $TOTAL C files to transpile"
echo ""

for C_FILE in $C_FILES; do
    RUST_FILE="${C_FILE%.c}.rs"

    echo "▶ Transpiling: $C_FILE"

    # Try to transpile
    if $DECY_BIN transpile "$C_FILE" > "$RUST_FILE" 2>/dev/null; then
        echo "  ✅ Generated: $RUST_FILE"

        # Try to compile the generated Rust
        if rustc --crate-type lib --edition 2021 "$RUST_FILE" -o "${RUST_FILE}.rlib" 2>/dev/null; then
            echo "  ✅ Compiles successfully!"
            rm -f "${RUST_FILE}.rlib"
            SUCCESS=$((SUCCESS + 1))
        else
            echo "  ⚠️  Generated but doesn't compile"
            FAIL=$((FAIL + 1))
        fi
    else
        echo "  ❌ Transpilation failed"
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Summary                                                   ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Total:    $TOTAL"
echo "Success:  $SUCCESS"
echo "Failed:   $FAIL"
echo ""

if [ $SUCCESS -eq $TOTAL ]; then
    echo "🎉 All examples transpiled and compiled!"
    exit 0
else
    echo "⚠️  Some examples failed"
    exit 1
fi
