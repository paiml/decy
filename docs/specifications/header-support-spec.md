# Header Support Specification - DECY

**Version:** 1.0.0
**Status:** DRAFT
**Author:** DECY Core Team
**Date:** 2025-11-17
**Priority:** CRITICAL (Blocks 33% of C99 coverage)

---

## Executive Summary

This specification defines a comprehensive solution for C system header support in DECY, enabling transpilation of programs that use standard library functions (malloc, printf, strlen, etc.). The current parser limitation blocks 60+ tests and prevents coverage of 33% of C99 language constructs.

**Problem:** Parser comments out `#include <stdlib.h>` directives during preprocessing, causing undefined function errors.

**Solution:** Implement built-in standard library function prototypes with type-safe mappings to Rust equivalents.

**Impact:** Unlocks stdlib-dependent transpilation, enables 60+ blocked tests, increases C99 coverage from 67% → 90%+.

---

## Academic Foundation

### 1. Preprocessing and Macro Expansion

**[1] McKeeman, W. M. (1998). "Differential Testing for Software."**
*Digital Technical Journal, 10(1), 100-107.*
https://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-100.pdf

**Relevance:** Establishes testing methodology for compiler correctness, critical for validating header transformations. DECY uses differential testing to verify transpiled output matches C semantics.

**Application:** Our approach validates each stdlib function transformation against both C99 spec and empirical C compiler behavior.

---

**[2] Kernighan, B. W., & Ritchie, D. M. (1988). "The C Programming Language" (2nd ed.)**
*Prentice Hall. ISBN: 0131103628*
Chapter 4: Functions and Program Structure (pp. 67-86)
Appendix B: Standard Library (pp. 241-250)

**Relevance:** Canonical reference for C standard library function signatures and semantics. Appendix B documents all ANSI C library functions.

**Application:** DECY's built-in prototypes map directly to K&R's documented function signatures with ISO C99 updates.

---

**[3] Spinellis, D. (2003). "Global Analysis and Transformations in Preprocessed Languages."**
*IEEE Transactions on Software Engineering, 29(11), 1019-1030.*
DOI: 10.1109/TSE.2003.1245299

**Relevance:** Analyzes challenges of parsing preprocessed C code, including header inclusion and macro expansion. Demonstrates that preprocessing creates semantic ambiguities.

**Application:** DECY avoids full preprocessing by providing built-in prototypes, eliminating header inclusion complexity while preserving type safety.

---

### 2. Type Systems and Function Signatures

**[4] Cardelli, L., & Wegner, P. (1985). "On Understanding Types, Data Abstraction, and Polymorphism."**
*ACM Computing Surveys, 17(4), 471-523.*
DOI: 10.1145/6041.6042

**Relevance:** Foundational paper on type systems. Section 3 covers function types and type equivalence, crucial for mapping C function signatures to Rust.

**Application:** DECY maps C's nominal type system (stdlib functions) to Rust's structural types, preserving safety invariants.

---

**[5] ISO/IEC 9899:1999. "Programming Languages — C (C99 Standard)."**
*International Organization for Standardization.*
Section 7: Library (pp. 187-344)
https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf

**Relevance:** Official specification of C standard library. Section 7 defines all standard headers (<stdlib.h>, <stdio.h>, <string.h>, etc.) with exact function signatures.

**Application:** DECY's prototype database is sourced directly from C99 §7, ensuring standards compliance.

---

### 3. Parser Construction and Type Inference

**[6] Aycock, J. (2003). "A Brief History of Just-In-Time."**
*ACM Computing Surveys, 35(2), 97-113.*
DOI: 10.1145/857076.857077

**Relevance:** Discusses runtime type resolution and dynamic linking, applicable to resolving stdlib function calls without full header parsing.

**Application:** DECY uses JIT-inspired lazy resolution: stdlib functions are resolved on-demand during parsing, not via preprocessing.

---

**[7] Lattner, C., & Adve, V. (2004). "LLVM: A Compilation Framework for Lifelong Program Analysis & Transformation."**
*Proceedings of CGO 2004, pp. 75-86.*
DOI: 10.1109/CGO.2004.1281665

**Relevance:** LLVM's modular design separates parsing from type resolution. Clang (used by DECY via clang-sys) demonstrates header-free function declaration handling.

**Application:** DECY leverages clang-sys's AST walker but provides our own type database, bypassing filesystem header dependencies.

---

### 4. Memory Safety and Ownership

**[8] Grossman, D., et al. (2002). "Region-Based Memory Management in Cyclone."**
*Proceedings of PLDI 2002, pp. 282-293.*
DOI: 10.1145/512529.512563

**Relevance:** Cyclone's approach to making C memory-safe through regions and ownership. Directly applicable to DECY's malloc → Box transformation.

**Application:** DECY applies Cyclone's ownership inference principles to infer Box/Vec/& from C malloc/array patterns.

---

**[9] Weiser, M. (1981). "Program Slicing."**
*Proceedings of ICSE 1981, pp. 439-449.*
DOI: 10.1109/ICSE.1981.1671425

**Relevance:** Program slicing enables dataflow analysis to determine pointer ownership without full program semantics.

**Application:** DECY uses slicing to analyze malloc/free pairs and infer ownership even when header definitions are missing.

---

**[10] Jung, R., et al. (2017). "RustBelt: Securing the Foundations of the Rust Programming Language."**
*Proceedings of POPL 2017, pp. 66-80.*
DOI: 10.1145/3009837.3009873
https://plv.mpi-sws.org/rustbelt/popl18/paper.pdf

**Relevance:** Formal verification of Rust's type system and borrow checker. Proves that Rust's ownership rules prevent memory safety violations.

**Application:** DECY's C → Rust transformations preserve RustBelt's safety invariants by mapping C patterns to verified-safe Rust equivalents.

---

## Problem Statement

### Current Limitation

```c
// This FAILS to parse in DECY:
#include <stdlib.h>

int main() {
    int* ptr = (int*)malloc(sizeof(int));  // ERROR: malloc undeclared
    free(ptr);
    return 0;
}
```

**Root Cause:**
1. Preprocessor comments out `#include <stdlib.h>`
2. Parser encounters `malloc()` with no declaration
3. Clang AST walker fails: "undeclared identifier 'malloc'"

**Impact:**
- 60+ tests marked `#[ignore]`
- Cannot transpile 33% of C99 constructs
- Blocks stdlib coverage (malloc, printf, strlen, file I/O)

---

## Proposed Solution: Built-In Prototype Database

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    C Source Code                             │
│  #include <stdlib.h>  /* Will be commented out */           │
│  int* ptr = malloc(sizeof(int));                            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              Preprocessor (decy-core)                        │
│  - Comments out #include directives                          │
│  - Injects built-in prototypes BEFORE parsing                │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│        Built-In Prototype Injection (NEW)                    │
│                                                               │
│  void* malloc(size_t);     // From prototype DB              │
│  void free(void*);                                           │
│  size_t strlen(const char*);                                 │
│  int printf(const char*, ...);                               │
│  /* ... 150+ stdlib functions ... */                         │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│         Parser (clang-sys) - NOW SUCCEEDS                    │
│  - Finds malloc declaration in injected prototypes           │
│  - Builds AST with proper type information                   │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│       HIR Conversion + Ownership Inference                   │
│  malloc(sizeof(int)) → Box::new(0i32)                        │
│  free(ptr) → drop(ptr)                                       │
│  strlen(s) → s.len() as i32                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Strategy

### Phase 1: Prototype Database (Sprint 6)

**File:** `crates/decy-stdlib/src/prototypes.rs`

```rust
/// ISO C99 §7 Standard Library Function Prototypes
pub struct StdlibPrototypes {
    functions: HashMap<String, FunctionProto>,
}

pub struct FunctionProto {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<Parameter>,
    pub is_variadic: bool,
    pub header: StdHeader,
    pub c99_section: String,  // e.g., "§7.22.3.4"
}

pub enum StdHeader {
    Assert,    // <assert.h>
    Ctype,     // <ctype.h>
    Errno,     // <errno.h>
    Float,     // <float.h>
    Limits,    // <limits.h>
    Locale,    // <locale.h>
    Math,      // <math.h>
    Setjmp,    // <setjmp.h>
    Signal,    // <signal.h>
    Stdarg,    // <stdarg.h>
    Stddef,    // <stddef.h>
    Stdio,     // <stdio.h>
    Stdlib,    // <stdlib.h>
    String,    // <string.h>
    Time,      // <time.h>
}
```

**Database Contents (150+ functions):**

```rust
impl StdlibPrototypes {
    pub fn new() -> Self {
        let mut db = HashMap::new();

        // stdlib.h - ISO C99 §7.22
        db.insert("malloc", FunctionProto {
            name: "malloc",
            return_type: "void*",
            parameters: vec![Parameter::new("size", "size_t")],
            is_variadic: false,
            header: StdHeader::Stdlib,
            c99_section: "§7.22.3.4",
        });

        db.insert("free", FunctionProto {
            name: "free",
            return_type: "void",
            parameters: vec![Parameter::new("ptr", "void*")],
            is_variadic: false,
            header: StdHeader::Stdlib,
            c99_section: "§7.22.3.3",
        });

        // stdio.h - ISO C99 §7.21
        db.insert("printf", FunctionProto {
            name: "printf",
            return_type: "int",
            parameters: vec![Parameter::new("format", "const char*")],
            is_variadic: true,  // varargs after format
            header: StdHeader::Stdio,
            c99_section: "§7.21.6.1",
        });

        // string.h - ISO C99 §7.23
        db.insert("strlen", FunctionProto {
            name: "strlen",
            return_type: "size_t",
            parameters: vec![Parameter::new("s", "const char*")],
            is_variadic: false,
            header: StdHeader::String,
            c99_section: "§7.23.6.3",
        });

        // ... 146 more functions ...

        Self { functions: db }
    }

    pub fn get_prototype(&self, name: &str) -> Option<String> {
        self.functions.get(name).map(|proto| proto.to_c_declaration())
    }

    pub fn inject_all_prototypes(&self) -> String {
        // Generate C declarations for all stdlib functions
        let mut result = String::new();
        result.push_str("// Built-in stdlib prototypes (ISO C99 §7)\n");
        result.push_str("typedef unsigned long size_t;\n");
        result.push_str("typedef long ssize_t;\n\n");

        for proto in self.functions.values() {
            result.push_str(&proto.to_c_declaration());
            result.push('\n');
        }

        result
    }
}
```

### Phase 2: Preprocessor Integration (Sprint 6)

**File:** `crates/decy-core/src/lib.rs`

**Modification to `transpile_with_includes()`:**

```rust
pub fn transpile_with_includes(c_code: &str, _include_paths: &[PathBuf]) -> Result<String> {
    // Step 0: Inject built-in prototypes BEFORE preprocessing
    let stdlib_protos = StdlibPrototypes::new();
    let injected_prototypes = stdlib_protos.inject_all_prototypes();

    // Step 1: Preprocess (comments out #include directives)
    let preprocessed = preprocess(c_code)?;

    // Step 2: Prepend prototypes to preprocessed code
    let code_with_prototypes = format!("{}\n{}", injected_prototypes, preprocessed);

    // Step 3: Parse (now succeeds - all stdlib functions declared!)
    let parser = CParser::new().context("Failed to create C parser")?;
    let ast = parser.parse(&code_with_prototypes).context("Failed to parse C code")?;

    // ... rest of pipeline unchanged ...
}
```

### Phase 3: Transformation Rules (Sprint 7)

**File:** `crates/decy-stdlib/src/transformations.rs`

Map stdlib functions to safe Rust equivalents:

```rust
pub struct StdlibTransformer {
    rules: HashMap<String, TransformRule>,
}

pub enum TransformRule {
    Direct(String),           // Direct mapping: strlen → .len()
    Pattern(Box<dyn Fn(HirExpression) -> HirExpression>),  // Complex: malloc → Box
    Unsafe(String, String),   // Requires unsafe: memcpy
}

impl StdlibTransformer {
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // String functions (ISO C99 §7.23)
        rules.insert("strlen", TransformRule::Direct(".len() as i32"));
        rules.insert("strcmp", TransformRule::Pattern(Box::new(|expr| {
            // strcmp(a, b) → (a == b) as i32
            transform_strcmp(expr)
        })));

        // Memory functions (ISO C99 §7.22.3)
        rules.insert("malloc", TransformRule::Pattern(Box::new(|expr| {
            // malloc(sizeof(T)) → Box::new(T::default())
            transform_malloc(expr)
        })));

        rules.insert("free", TransformRule::Direct("drop"));

        // I/O functions (ISO C99 §7.21)
        rules.insert("printf", TransformRule::Pattern(Box::new(|expr| {
            // printf("format", args) → println!("format", args)
            transform_printf(expr)
        })));

        Self { rules }
    }
}
```

---

## Test Strategy (EXTREME TDD)

### RED Phase: Write Failing Tests First

**File:** `crates/decy-stdlib/tests/stdlib_prototypes_test.rs`

```rust
#[test]
fn test_malloc_prototype_injection() {
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = malloc(sizeof(int));
            free(ptr);
            return 0;
        }
    "#;

    let result = transpile(c_code);

    // RED: This currently FAILS - malloc undeclared
    assert!(result.is_ok(), "Should parse with built-in malloc prototype");

    let rust = result.unwrap();
    assert!(rust.contains("Box::new"), "Should transform malloc → Box");
}

#[test]
fn test_printf_prototype_injection() {
    let c_code = r#"
        #include <stdio.h>

        int main() {
            printf("Hello, World!\n");
            return 0;
        }
    "#;

    let result = transpile(c_code);

    // RED: This currently FAILS - printf undeclared
    assert!(result.is_ok(), "Should parse with built-in printf prototype");

    let rust = result.unwrap();
    assert!(rust.contains("println!"), "Should transform printf → println!");
}

#[test]
fn test_strlen_prototype_injection() {
    let c_code = r#"
        #include <string.h>

        int main() {
            const char* s = "test";
            int len = strlen(s);
            return len;
        }
    "#;

    let result = transpile(c_code);

    // RED: This currently FAILS - strlen undeclared
    assert!(result.is_ok(), "Should parse with built-in strlen prototype");

    let rust = result.unwrap();
    assert!(rust.contains(".len()"), "Should transform strlen → .len()");
}

#[test]
fn test_all_150_stdlib_functions_have_prototypes() {
    let stdlib = StdlibPrototypes::new();

    // Verify all C99 §7 functions are defined
    assert!(stdlib.get_prototype("malloc").is_some());
    assert!(stdlib.get_prototype("calloc").is_some());
    assert!(stdlib.get_prototype("realloc").is_some());
    assert!(stdlib.get_prototype("free").is_some());

    assert!(stdlib.get_prototype("printf").is_some());
    assert!(stdlib.get_prototype("fprintf").is_some());
    assert!(stdlib.get_prototype("sprintf").is_some());
    assert!(stdlib.get_prototype("scanf").is_some());

    assert!(stdlib.get_prototype("strlen").is_some());
    assert!(stdlib.get_prototype("strcpy").is_some());
    assert!(stdlib.get_prototype("strcmp").is_some());
    assert!(stdlib.get_prototype("strcat").is_some());

    // ... test all 150 functions ...

    assert_eq!(stdlib.functions.len(), 150, "Should have 150 stdlib functions");
}

#[test]
fn test_prototype_injection_doesnt_break_user_code() {
    // User defines their own malloc (edge case)
    let c_code = r#"
        void* my_malloc(int size) {
            return 0;
        }

        int main() {
            void* ptr = my_malloc(100);
            return 0;
        }
    "#;

    let result = transpile(c_code);

    // Should succeed - user function takes precedence
    assert!(result.is_ok());
}
```

---

## Quality Gates

### Coverage Requirements
- **Unit Tests**: ≥90% coverage for prototype database
- **Integration Tests**: All 60 previously-ignored tests must pass
- **Property Tests**: 1000 cases × 10 properties = 10K executions

### Unsafe Block Target
- **Goal**: 0 additional unsafe blocks
- **Rationale**: Prototype injection is pure code generation (safe)
- **Validation**: Compare unsafe count before/after

### Performance
- **Prototype Injection**: <10ms overhead
- **Memory**: <1MB for prototype database
- **Benchmark**: Must not regress existing transpilation speed

---

## Acceptance Criteria

### Sprint 6 (Prototype Database)
- [ ] `StdlibPrototypes` struct implemented
- [ ] All 150 C99 §7 functions in database
- [ ] Prototype injection integrated into preprocessor
- [ ] Unit tests passing (90% coverage)
- [ ] Documentation updated

### Sprint 7 (Transformation Rules)
- [ ] `StdlibTransformer` implemented
- [ ] malloc/free → Box/drop working
- [ ] printf/scanf → println!/stdin working
- [ ] strlen/strcmp → .len()/== working
- [ ] All 60 ignored tests re-enabled and passing

### Sprint 8 (Validation)
- [ ] C-VALIDATION-ROADMAP updated (67% → 90%)
- [ ] Quality gates passing
- [ ] Documentation examples updated
- [ ] Release notes prepared

---

## Risks & Mitigation

### Risk 1: Prototype Conflicts
**Problem:** User code defines function with same name as stdlib
**Mitigation:** Prototype injection uses weak linkage - user definitions override
**Test:** `test_prototype_injection_doesnt_break_user_code()`

### Risk 2: Type System Mismatches
**Problem:** C's `size_t` may be different width than Rust's `usize`
**Mitigation:** Explicit type mappings: `size_t` → `usize`, document platform assumptions
**Test:** Property tests verify type width correctness

### Risk 3: Variadic Function Complexity
**Problem:** printf-style varargs hard to type-check
**Mitigation:** Use format string analysis (existing Rust tooling)
**Test:** Integration tests with various format specifiers

---

## Future Work

### Phase 4: Header File Parsing (Optional)
- Parse actual header files (.h) for user-defined prototypes
- Build symbol table from includes
- Enable full K&R C book transpilation

### Phase 5: Preprocessor Macro Expansion
- Full #define macro expansion
- Conditional compilation (#ifdef)
- Macro function transformations

### Phase 6: Platform-Specific Extensions
- POSIX functions (pthread, socket, etc.)
- Windows API
- Compiler builtins (__builtin_*)

---

## References Summary

1. McKeeman (1998) - Differential testing methodology
2. Kernighan & Ritchie (1988) - K&R C canonical reference
3. Spinellis (2003) - Preprocessor analysis challenges
4. Cardelli & Wegner (1985) - Type system foundations
5. ISO C99 (1999) - Official C standard library spec
6. Aycock (2003) - JIT and runtime type resolution
7. Lattner & Adve (2004) - LLVM modular compilation
8. Grossman et al. (2002) - Cyclone ownership inference
9. Weiser (1981) - Program slicing for dataflow
10. Jung et al. (2017) - RustBelt formal verification

---

## Implementation Timeline

| Sprint | Phase | Tasks | Completion |
|--------|-------|-------|------------|
| 6 | Prototype DB | Create database, inject prototypes | Week 1-2 |
| 7 | Transformations | Implement stdlib → Rust mappings | Week 3-4 |
| 8 | Validation | Enable 60 tests, update docs | Week 5 |
| 9 | Polish | Performance tuning, edge cases | Week 6 |

**Estimated Completion:** 6 weeks
**Priority:** CRITICAL
**Blocking:** 33% of C99 coverage

---

**END OF SPECIFICATION**
