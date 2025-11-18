# DECY C Examples - Heterogeneous Test Collection

This directory contains **18+ real-world C programs** organized by category to validate DECY's transpilation capabilities. These examples represent diverse C programming patterns found in production codebases.

## Quick Start

### Run Validation Suite
```bash
# Compile and run validator
rustc examples/validation_runner.rs -o /tmp/validation_runner
/tmp/validation_runner

# Or transpile all examples
chmod +x scripts/transpile_examples.sh
./scripts/transpile_examples.sh
```

### Test a Single File
```bash
# Transpile
cargo run --bin decy -- transpile examples/simple/arithmetic.c

# Transpile and save
cargo run --bin decy -- transpile examples/cli/word_count.c > examples/cli/word_count.rs

# Check if it compiles
rustc examples/cli/word_count.rs -o /dev/null
```

## Test Categories (9 total)

### 📊 Current Status

| Category | Files | Compiles | Coverage | Blocker |
|----------|-------|----------|----------|---------|
| Simple | 3 | 🔲 | 0% | Needs transpilation |
| Moderate | 1 | 🔲 | 0% | Needs transpilation |
| Pointer Arithmetic | 3 | 🔲 | 0% | Needs transpilation |
| Real-World | 4 | ⚠️ | 0% | Generated but broken |
| **CLI Tools** | 2 | 🔲 | **0%** | **File I/O (DECY-089)** |
| **Data Structures** | 2 | 🔲 | **0%** | **Lifetimes** |
| **File I/O** | 1 | 🔲 | **0%** | **FILE* APIs (DECY-089)** |
| **Threading** | 1 | 🔲 | **0%** | **Condvars** |
| **Strings** | 1 | 🔲 | **0%** | **String detection (DECY-088)** |
| **TOTAL** | **18** | **0/18** | **0%** | See roadmap |

## Examples by Category

### 1. Simple (Baseline Validation)
```
examples/simple/
├── arithmetic.c        # Basic arithmetic (+, -, *, /)
├── minimal.c           # Minimal valid C program
└── return_value.c      # Return value handling
```
**Tests**: Parser correctness, basic codegen

### 2. Moderate (Intermediate)
```
examples/moderate/
└── control_flow.c      # if/else, loops, switch
```
**Tests**: Control flow, block scoping

### 3. Pointer Arithmetic (Safe Transformation)
```
examples/pointer_arithmetic/
├── compound_assignments.c    # ptr += n, ptr -= n
├── increment_decrement.c     # ptr++, ptr--
└── real_world_patterns.c     # Common idioms
```
**Tests**: DECY-069 (slice indexing), DECY-070 (safe bounds)

### 4. Real-World (Production Patterns)
```
examples/real-world/
├── linked_list.c      # Singly-linked list
├── buffer_ops.c       # Buffer manipulation
├── math_utils.c       # Math utilities
└── string_utils.c     # String processing
```
**Tests**: malloc/free → Box/Vec, real APIs

### 5. CLI Tools ⭐ (Command-Line Utilities)
```
examples/cli/
├── word_count.c       # Line by line: wc -w implementation
│   Tests: fopen, fgets, fclose, isspace()
│   Blocker: DECY-089 (File I/O APIs)
│
└── simple_grep.c      # Pattern matching: grep implementation
    Tests: strstr(), output parameters (DECY-083 ✅)
    Blocker: DECY-089 (File I/O APIs)
```

**Why These Matter**:
- ✅ Tests output parameters (`int grep_file(..., int* match_count)`)
- 🔲 Tests FILE* operations (critical gap!)
- 🔲 Tests argc/argv handling
- Real-world utility pattern

### 6. Data Structures ⭐ (Memory Management)
```
examples/data_structures/
├── binary_tree.c      # Recursive BST
│   Tests: Recursive types, malloc/free patterns
│   Blocker: Lifetime inference
│
└── hash_table.c       # Hash table with chaining
    Tests: Arrays of pointers, linked lists, string hashing
    Blocker: Complex ownership
```

**Why These Matter**:
- Tests ownership inference (DECY-ownership crate)
- Recursive structures (`struct TreeNode* left`)
- Dynamic arrays of pointers

### 7. File I/O ⭐ (BLOCKER Category)
```
examples/file_io/
└── csv_parser.c       # CSV file parsing
    Tests: fopen, fgets, strtok, realloc
    Blocker: DECY-089, DECY-090, DECY-091
```

**Critical Gap**: No FILE* API support yet!

**Needs**:
- `fopen/fclose` → `File::open/drop`
- `fgets` → `BufRead::read_line`
- `fprintf` → `write!` macro

### 8. Threading ⭐ (Concurrency)
```
examples/threading/
└── producer_consumer.c
    Tests: pthread_create, pthread_mutex, pthread_cond
    Partial Support: DECY-078 (mutex ✅), condvars 🔲
```

**Why It Matters**:
- pthread → std::thread (DECY-078 complete)
- Mutex → Mutex<T> (DECY-078 complete)
- Condition variables → Condvar (not yet implemented)

### 9. String Processing ⭐ (String vs &str)
```
examples/strings/
└── string_builder.c   # Dynamic string builder
    Tests: realloc, strcpy, strlen
    Blocker: DECY-088 (String detection)
```

**Why It Matters**:
- Detects `char* buf = malloc(100)` → `String`
- vs `char* s = "hello"` → `&str`
- Tests realloc → Vec::reserve

## Gap Analysis: What's Blocking Each Category?

### ✅ Works Today
- malloc/free → Box/Vec (DECY-070, DECY-072)
- pthread_mutex → Mutex<T> (DECY-078)
- Pointer arithmetic → safe slices (DECY-069)
- Output parameter **detection** (DECY-083, just done!)

### 🔲 High-Priority Gaps

#### 1. File I/O APIs (Blocks: CLI Tools, File I/O)
**Missing**: fopen, fgets, fclose, fprintf
**Roadmap**: DECY-089, DECY-090, DECY-091
**Impact**: 3 examples (word_count, simple_grep, csv_parser)

#### 2. Output Params → Result/Option (Blocks: CLI Tools)
**Missing**: Transform detected output params to Rust types
**Roadmap**: DECY-084
**Impact**: simple_grep.c uses `int grep_file(..., int* match_count)`

#### 3. String Detection (Blocks: Strings)
**Missing**: char* → String vs &str heuristics
**Roadmap**: DECY-088
**Impact**: string_builder.c

#### 4. Lifetime Inference (Blocks: Data Structures)
**Missing**: Automatic lifetime annotations for trees
**Roadmap**: Phase 3 (unsafe reduction 20% → 10%)
**Impact**: binary_tree.c, hash_table.c

#### 5. Condition Variables (Blocks: Threading)
**Missing**: pthread_cond → Condvar
**Roadmap**: Not yet scheduled
**Impact**: producer_consumer.c

## Testing Strategy

### Phase 1: Baseline (Sprint 25)
1. Transpile all `simple/` examples
2. Validate parser correctness
3. **Target**: 3/3 simple examples compile

### Phase 2: Core Features (Sprint 25-26)
4. Implement DECY-084 (output params → Result)
5. Test on `simple_grep.c`
6. Implement DECY-086-088 (arrays, strings)
7. **Target**: 6/18 examples compile

### Phase 3: File I/O (Sprint 27)
8. Implement DECY-089-091 (FILE* APIs)
9. Test on `word_count.c`, `csv_parser.c`
10. **Target**: 9/18 examples compile

### Phase 4: Advanced (Sprint 28-30)
11. Lifetime inference
12. Condition variables
13. **Target**: 18/18 examples compile (100%)

## Adding New Examples

### Example Template
```c
// Brief description (one line)
// Tests: feature1, feature2, feature3
// Blockers: DECY-XXX, DECY-YYY (if known)
// Expected: Should transpile after Sprint X

#include <stdio.h>

int main(void) {
    // Your code here
    return 0;
}
```

### Checklist
- [ ] Create C file in appropriate category
- [ ] Document what it tests in comment header
- [ ] Note blockers (which roadmap tickets needed)
- [ ] Keep it simple (<100 lines)
- [ ] Run validation: `/tmp/validation_runner`
- [ ] File GitHub issue if new gap discovered

## Running Tests

### Validation Runner
```bash
# Build and run
rustc examples/validation_runner.rs -o /tmp/validation_runner
/tmp/validation_runner

# Expected output:
# ✅ PASS - Transpiled and compiles
# ⚠️  PARTIAL - Transpiled but doesn't compile (gap!)
# ❌ FAIL - Parser failed (bug!)
```

### Transpile All Examples
```bash
chmod +x scripts/transpile_examples.sh
./scripts/transpile_examples.sh
```

### Manual Test
```bash
# Transpile
cargo run --bin decy -- transpile examples/cli/word_count.c

# Check syntax
cargo run --bin decy -- check examples/cli/word_count.c

# Debug AST
cargo run --bin decy -- debug --visualize-ast examples/cli/word_count.c
```

## Metrics & Targets

### Current (Baseline)
- **Total Examples**: 18
- **Categories**: 9
- **Compiling**: 0/18 (0%)
- **Partially Working**: 4/18 (22%) - real-world examples

### Sprint 25 Target
- **Compiling**: 6/18 (33%)
- **Focus**: Simple + output params + arrays

### Sprint 27 Target
- **Compiling**: 12/18 (67%)
- **Focus**: File I/O complete

### Sprint 30 Target (Jan 2026)
- **Compiling**: 18/18 (100%)
- **Goal**: All heterogeneous examples work

## FAQ

**Q: Why aren't examples transpiled yet?**
A: DECY is under active development. Run `./scripts/transpile_examples.sh` to generate Rust files.

**Q: Why do some examples fail to compile?**
A: They test features not yet implemented (see "Blockers" in each category).

**Q: How do I know what's blocking an example?**
A: Check the comment header or TEST_COLLECTION.md gap analysis.

**Q: Can I add my own examples?**
A: Yes! Follow the template and PR to the repo.

**Q: Which examples should work today?**
A: Simple examples + moderate should work once transpiled. Others need roadmap features.

## References

- **Full Gap Analysis**: `TEST_COLLECTION.md`
- **Validation Roadmap**: `docs/C-VALIDATION-ROADMAP.yaml`
- **Project Roadmap**: `roadmap.yaml`
- **Transpiler Spec**: `docs/specifications/decy-spec-v1.md`

---

**Last Updated**: 2025-11-18
**DECY Version**: 1.0.1
**Completion**: 0/18 examples (0%) - baseline established
