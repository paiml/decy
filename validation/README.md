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

**Phase 1: EXTRACTION**
- ✅ Chapter 1: 3/25 examples (hello world, temperature tables)
- ✅ Chapter 4: 3/35 examples (global variables, static variables)
- ⏳ Chapter 2-3, 5-8: Pending extraction
- ⏳ C99 spec: Pending extraction

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
