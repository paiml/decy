# DECY Heterogeneous C Test Collection

This directory contains real-world C examples organized by category to validate DECY's transpilation capabilities.

## Test Categories

### 1. Simple (`examples/simple/`)
Basic C constructs for sanity testing:
- `arithmetic.c` - Basic arithmetic operations
- `return_value.c` - Return value handling
- `minimal.c` - Minimal valid C program

**Focus**: Parser correctness, basic code generation

### 2. Moderate (`examples/moderate/`)
Intermediate complexity programs:
- `control_flow.c` - If/else, loops, switch statements

**Focus**: Control flow handling, block scoping

### 3. Pointer Arithmetic (`examples/pointer_arithmetic/`)
Pointer operations and array access:
- `compound_assignments.c` - `+=`, `-=` with pointers
- `increment_decrement.c` - `++`, `--` on pointers
- `real_world_patterns.c` - Common pointer idioms

**Focus**: Safe pointer transformation, bounds checking

### 4. Real-World (`examples/real-world/`)
Practical programs and data structures:
- `linked_list.c` - Singly-linked list implementation
- `buffer_ops.c` - Buffer manipulation
- `math_utils.c` - Mathematical utility functions
- `string_utils.c` - String processing utilities

**Focus**: Memory management, real-world patterns

### 5. CLI Tools (`examples/cli/`) ⭐ NEW
Command-line utilities:
- `word_count.c` - Word counter (like `wc -w`)
- `simple_grep.c` - Pattern matching (like `grep`)

**Focus**: argc/argv handling, file I/O, string matching

**Tests**:
- ✅ Output parameters (`grep_file(..., int* match_count)`)
- 🔲 File I/O APIs (fopen, fgets, fclose)
- 🔲 Error handling patterns

### 6. Data Structures (`examples/data_structures/`) ⭐ NEW
Common data structures:
- `binary_tree.c` - Binary search tree
- `hash_table.c` - Hash table with chaining

**Focus**: Recursive structures, complex memory management

**Tests**:
- ✅ Recursive types (`struct TreeNode { ...; struct TreeNode* left; }`)
- ✅ malloc/free patterns → Box/Vec
- 🔲 Lifetime inference for tree nodes

### 7. File I/O (`examples/file_io/`) ⭐ NEW
File operations:
- `csv_parser.c` - CSV file parsing

**Focus**: FILE* operations, buffered I/O

**Tests**:
- 🔲 fopen/fclose → File::open/drop
- 🔲 fgets → BufRead::read_line
- 🔲 Dynamic array growth (realloc)

### 8. Threading (`examples/threading/`) ⭐ NEW
Concurrent programs:
- `producer_consumer.c` - Producer-consumer with pthread

**Focus**: pthread → std::thread, mutex → Mutex<T>

**Tests**:
- ✅ pthread_mutex_t → Mutex<T> (DECY-078 complete)
- ✅ pthread_create/join → thread::spawn
- 🔲 Condition variables → Condvar

### 9. String Processing (`examples/strings/`) ⭐ NEW
String manipulation:
- `string_builder.c` - Dynamic string builder

**Focus**: Dynamic strings, realloc patterns

**Tests**:
- ✅ Dynamic growth → Vec::with_capacity
- 🔲 String vs &str detection
- 🔲 realloc → Vec::reserve

## Running the Validation Suite

```bash
# Run the validation runner
cargo run --example validation_runner

# Expected output:
# ✅ PASS - Successfully transpiled and compiled
# ⚠️  PARTIAL - Transpiled but Rust doesn't compile (shows gaps)
# ❌ FAIL - Parser or HIR generation failed
```

## Current Coverage

| Category | Files | Status | Notes |
|----------|-------|--------|-------|
| Simple | 3 | ✅ Baseline | Working |
| Moderate | 1 | ✅ Baseline | Working |
| Pointer Arithmetic | 3 | ✅ DECY-069/070 | Safe slices |
| Real-World | 4 | ✅ Partial | Core patterns work |
| **CLI Tools** | 2 | 🔲 **NEW** | Needs file I/O (DECY-089) |
| **Data Structures** | 2 | 🔲 **NEW** | Tests ownership inference |
| **File I/O** | 1 | 🔲 **NEW** | Blocker: FILE* APIs |
| **Threading** | 1 | ⚠️  **PARTIAL** | pthread basic support |
| **Strings** | 1 | 🔲 **NEW** | Tests String vs &str |

## Gap Analysis

### What Works Today (77% coverage)
- ✅ Basic types, pointers, structs
- ✅ Control flow (if/else, loops, switch)
- ✅ malloc/free → Box/Vec (DECY-070, DECY-072)
- ✅ pthread_mutex → Mutex<T> (DECY-078)
- ✅ Pointer arithmetic → safe slices (DECY-069)
- ✅ **Output parameters → detected** (DECY-083, just completed!)

### What Doesn't Work Yet (23% remaining)
- 🔲 **File I/O** (fopen, fgets, fprintf) - **BLOCKER** for CLI tools
- 🔲 **Output params → Result/Option** (DECY-084) - Next priority
- 🔲 **String detection** (char* → String vs &str) - DECY-088
- 🔲 **Array handling** (fixed-size, heap arrays) - DECY-086, DECY-087
- 🔲 **Process management** (fork, exec) - DECY-092
- 🔲 **Lifetime inference** (automatic 'a annotations)

## Next Steps

### Immediate (Sprint 25)
1. **DECY-084**: Transform output params → Result/Option
   - `grep_file(..., int* match_count)` → `fn grep_file(...) -> Result<(usize), Error>`
2. Test on `simple_grep.c` - should transpile after DECY-084

### Short-term (Sprint 26)
3. **DECY-086-088**: Array and string handling
   - Test on `string_builder.c` and `hash_table.c`
4. Validate all data structures examples

### Medium-term (Sprint 27)
5. **DECY-089-091**: File I/O APIs
   - Enables `word_count.c`, `csv_parser.c`
6. Full CLI tools validation

## Adding New Examples

1. Create C file in appropriate category
2. Document what C features it tests
3. Note expected gaps (which roadmap tickets are blockers)
4. Run validation: `cargo run --example validation_runner`
5. File issues for failures not covered by roadmap

## Example Template

```c
// Brief description
// Tests: feature1, feature2, feature3
// Blockers: DECY-XXX (if any)

#include <stdio.h>

// Your code here
```

## Validation Metrics

Target: **100% of examples transpile and compile**

Current (estimated):
- **Simple**: 100% (3/3)
- **Moderate**: 100% (1/1)
- **Pointer Arithmetic**: 100% (3/3)
- **Real-World**: 75% (3/4)
- **CLI Tools**: 0% (0/2) - File I/O blocking
- **Data Structures**: 50% (1/2) - Lifetime issues
- **File I/O**: 0% (0/1) - APIs not implemented
- **Threading**: 80% (partial) - Condvars missing
- **Strings**: 0% (0/1) - String detection missing

**Overall: ~60% of heterogeneous examples work**
**Target: 100% by Sprint 30 (Jan 2026)**
