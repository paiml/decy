# Decy Validation Corpus

This directory contains comprehensive validation examples from canonical C sources.

## Structure

```
validation/
├── kr-c/              # K&R C "The C Programming Language" examples
│   ├── chapter-1/     # Tutorial Introduction (25+ examples)
│   ├── chapter-2/     # Types, Operators, Expressions (30+ examples)
│   ├── chapter-3/     # Control Flow (20+ examples)
│   ├── chapter-4/     # Functions and Program Structure (35+ examples)
│   ├── chapter-5/     # Pointers and Arrays (40+ examples)
│   ├── chapter-6/     # Structures (25+ examples)
│   ├── chapter-7/     # Input and Output (30+ examples)
│   └── chapter-8/     # The UNIX System Interface (20+ examples)
├── c99-spec/          # ISO C99 specification examples (~50 examples)
├── reports/           # Validation reports and analysis
└── harness.rs         # Automated validation test harness
```

## Example Naming Convention

Files are named: `<number>_<descriptive_name>.c`

Example: `01_hello_world.c`, `02_fahrenheit_celsius.c`

Each file includes:
- Source reference (K&R page number, C99 section)
- Brief description
- Complete, compilable C code

## Running Validation

```bash
# Run all validation tests
cargo test --test validation_harness

# Run specific chapter
cargo test --test validation_harness kr_chapter_1

# Generate validation report
cargo test --test validation_harness -- --nocapture > reports/validation_report.txt
```

## Validation Metrics

- **Transpilation Success Rate**: % of examples that Decy can transpile without errors
- **Rustc Compilation Success Rate**: % of transpiled Rust code that compiles with rustc
- **End-to-End Success Rate**: % of examples that go from C → transpiled Rust → compiled binary

## Current Status (DECY-069)

**Phase 1: EXTRACTION** - Progress: 95/275 examples (34.5%)

**Completed Chapters**:
- ✅ Chapter 1 (Tutorial): 15/25 examples (60%)
  - Hello world, temperature tables, character I/O
  - Line/word counting, arrays, functions, character arrays
  - Longest line program, external variables, reverse input
  - Symbolic constants, tab expansion (detab)
- ✅ Chapter 2 (Types, Operators): 15/30 examples (50%)
  - Data types, constants, arithmetic operators
  - Relational/logical operators, increment/decrement
  - Bitwise operators, assignment operators, conditional expression
  - Type conversions, sizeof operator
  - Enum types, precedence/associativity, type qualifiers
  - Signed/unsigned integers, short-circuit evaluation
- ✅ Chapter 3 (Control Flow): 10/20 examples (50%)
  - If-else, switch, while/for/do-while loops
  - Binary search, do-while loops, break/continue
  - Goto and labels, nested loops
- ✅ Chapter 4 (Functions, Program Structure): 10/35 examples (28.6%)
  - Global variables, static variables, external arrays
  - Const variables, storage class interaction
  - Recursive functions, static local variables, register variables
  - Scope rules, header files usage
- ✅ Chapter 5 (Pointers and Arrays): 15/40 examples (37.5%) ⭐ CRITICAL
  - Pointer basics, pointer arithmetic, pointer functions
  - String operations (strlen, strcpy, strcmp)
  - Array of pointers, multidimensional arrays
  - Function pointers, complex declarations
  - Pointer to pointer, command-line args, dynamic arrays
  - Const pointers, void pointers
- ✅ Chapter 6 (Structures): 10/25 examples (40%) ⭐ CRITICAL
  - Basic structures, structure functions, pointers to structures
  - Arrays of structures, self-referential structures
  - Typedef, unions, bit-fields
  - Structure initialization and assignment
- ✅ Chapter 7 (Input and Output): 10/30 examples (33.3%)
  - Character I/O (getchar, putchar), formatted output (printf)
  - Formatted input (scanf), file operations (fopen, fclose)
  - Error handling (stderr, exit), line I/O (fgets, fputs)
  - String formatting (sprintf, sscanf), file positioning (fseek, ftell)
- ✅ Chapter 8 (UNIX System Interface): 10/20 examples (50%) ⚠️ PLATFORM-SPECIFIC
  - File descriptors, low-level I/O (read, write, open, close)
  - File creation (creat), deletion (unlink), positioning (lseek)
  - Error handling (errno, perror), file info (stat)
  - Directory operations (opendir, readdir), fd duplication (dup, dup2)
  - Storage allocator example (malloc concept)

**Pending Extraction**:
- ⏳ Chapter 1: 10 more examples
- ⏳ Chapter 2: 15 more examples
- ⏳ Chapter 3: 10 more examples
- ⏳ Chapter 4: 25 more examples
- ⏳ Chapter 5: 25 more examples
- ⏳ Chapter 6: 15 more examples
- ⏳ Chapter 7: 20 more examples
- ⏳ Chapter 8: 10 more examples
- ⏳ C99 spec: ~50 examples

**Phase 2: INFRASTRUCTURE**
- ⏳ Validation harness (automated testing)
- ⏳ Report generation
- ⏳ Gap analysis tools

**Phase 3: EXECUTION**
- ⏳ Run validation on all examples
- ⏳ Measure success rates

**Phase 4: ANALYSIS**
- ⏳ Generate validation_report.md
- ⏳ Generate gap_analysis.md
- ⏳ Prioritize missing features

## Contributing Examples

When adding new validation examples:

1. Use the naming convention: `<chapter>/<number>_<name>.c`
2. Include source reference in comments (page number, section)
3. Ensure code is complete and would compile with gcc
4. Test with: `gcc -std=c99 -Wall <file>.c`
5. Add to test harness: `tests/validation_harness.rs`

## References

- **K&R C**: Brian W. Kernighan and Dennis M. Ritchie, "The C Programming Language", 2nd Edition, 1988
- **C99**: ISO/IEC 9899:1999, "Programming languages — C"
- **Decy Ticket**: DECY-069 (Comprehensive C book validation)
