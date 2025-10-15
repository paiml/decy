# DECY Validation Strategy

## Problem Statement

As identified in the development process, DECY faced a **major conceptual problem**: lacking a systematic way to validate transpilation against an authoritative C language reference (similar to how the bashrs sister project validates against the GNU Bash Manual).

## Solution: C99/K&R as the "North Star"

Following the bashrs validation methodology, DECY now uses **ISO C99 Standard and K&R C (2nd Edition)** as the authoritative validation references.

### Authoritative References

1. **ISO C99 Standard** (ISO/IEC 9899:1999)
   - Complete C language specification
   - 300+ sections covering all language constructs
   - Available at: https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf

2. **K&R C (2nd Edition)** (Kernighan & Ritchie)
   - Canonical C reference with practical examples
   - 8 chapters + 6 appendix sections
   - ISBN: 0131103628

3. **Supplementary References**
   - GCC Documentation (for extensions)
   - Clang Documentation (for AST structure)

## Validation Roadmap: C-VALIDATION-ROADMAP.yaml

The validation roadmap (`docs/C-VALIDATION-ROADMAP.yaml`) provides:

- **150 C language constructs** mapped to Rust transformations
- **ISO C99 and K&R section references** for each construct
- **Example C code → Expected Rust code** for every transformation
- **Unsafe block tracking** per construct (target: <5 per 1000 LOC)
- **Test coverage** and completion status

### Roadmap Structure (8 Major Chapters)

1. **Lexical Elements (C99 §6.4)** - Keywords, identifiers, constants, literals
2. **Types (C99 §6.2.5, §6.7)** - Basic types, pointers, arrays, structs, qualifiers
3. **Expressions (C99 §6.5)** - Operators, precedence, sizeof, pointer arithmetic
4. **Statements (C99 §6.8)** - Control flow, loops, jumps
5. **Functions (C99 §6.9.1)** - Declarations, definitions, parameters, variadic
6. **Standard Library (C99 §7)** - stdio, stdlib, string functions
7. **Preprocessor (C99 §6.10)** - #include, #define, #ifdef
8. **Unsafe Minimization** - 4-phase strategy to reduce unsafe blocks

### Example Validation Entry

```yaml
TYPE-PTR-MALLOC:
  title: "Document malloc/free pattern → Box"
  status: "completed"
  version: "v0.3.0"
  examples:
    - c: "int* p = malloc(sizeof(int)); free(p);"
      rust: "let p = Box::new(0i32);"
      unsafe_count: 0
  validation_reference: "K&R §8.7, ISO C99 §7.20.3"
  test_name: "test_malloc_to_box_transformation"
  completed_ticket: "DECY-044"
```

## STOP THE LINE Protocol (Andon Cord)

Inspired by Toyota Production System and adapted from bashrs methodology.

### When to Pull the Andon Cord

Halt all feature development immediately when validation reveals:

- Transpiled code doesn't compile
- Transpiled code has different behavior than C
- Unsafe block count exceeds target (<5 per 1000 LOC)
- Parser fails on valid C99 construct
- Ownership inference creates memory leak
- Verification stage fails safety checks

### P0 Bug Handling Procedure

1. **STOP** all feature development immediately
2. **Create P0 ticket** in `roadmap.yaml`:

```yaml
P0-MALLOC-001:
  title: "[P0] Fix malloc(sizeof(T)) transpilation bug"
  type: bug
  priority: critical
  status: in_progress
  phase: RED
  discovered_during_validation: true
  validation_reference: "K&R §8.7, ISO C99 §7.20.3"
  c_input: |
    int* ptr = malloc(sizeof(int));
  expected_rust: |
    let ptr = Box::new(0i32);
  actual_rust: |
    let ptr = Sizeof { type_name: "int" };  // BUG: malloc lost
  safety_impact: "Memory allocation lost, potential memory leak"
```

3. **Apply EXTREME TDD fix** (RED-GREEN-REFACTOR)
   - RED: Write failing test with validation reference
   - GREEN: Minimal fix to pass test
   - REFACTOR: Meet quality gates, verify unsafe count

4. **Verify** fix meets requirements:
   - All tests pass (`make test`)
   - Quality gates pass (`make quality-gates`)
   - Coverage ≥80% (≥90% for decy-ownership)
   - Unsafe count didn't increase
   - Original validation case passes

5. **Resume** validation only after fix verified and committed

## Validation Workflow

### Step 1: Select C Construct from Roadmap

Choose next construct from `C-VALIDATION-ROADMAP.yaml` based on priority:

```bash
# Check current validation status
cat docs/C-VALIDATION-ROADMAP.yaml | grep "status: not_started" | head -5
```

### Step 2: Create Reference Test

Write test using C99/K&R reference:

```rust
#[test]
fn test_<construct>_validation() {
    // Reference: ISO C99 §X.Y.Z, K&R Chapter N
    let c_code = r#"
        <C code from spec>
    "#;

    let expected_rust = r#"
        <Expected Rust output>
    "#;

    let result = transpile(c_code)?;
    assert_eq!(result, expected_rust);

    // Verify unsafe count
    let unsafe_count = count_unsafe_blocks(&result);
    assert!(unsafe_count <= MAX_UNSAFE_PER_MODULE);
}
```

### Step 3: Validate Against Reference

Run test against C99/K&R examples:

```bash
# Run validation test
cargo test -p decy-core test_<construct>_validation

# If test fails, PULL ANDON CORD
# Create P0 ticket and apply EXTREME TDD fix
```

### Step 4: Update Roadmap

Mark construct as validated:

```yaml
<CONSTRUCT-ID>:
  status: completed
  validation_reference: "ISO C99 §X.Y.Z, K&R Chapter N"
  tests_added:
    - "test_<construct>_validation (unit test)"
    - "property test with 1000 cases"
  unsafe_blocks: 0
```

### Step 5: Document Transformation

Add to examples and documentation:

```bash
# Add example to examples/
echo "// C99 §X.Y.Z: <construct>" > examples/<construct>_demo.rs

# Update CHANGELOG.md
echo "- Validated <construct> against C99 §X.Y.Z" >> CHANGELOG.md
```

## Unsafe Minimization Tracking

Every validation includes unsafe block counting:

### 4-Phase Reduction Strategy

1. **Pattern-Based** (100% → 50%)
   - Status: IN_PROGRESS
   - Target constructs: malloc/free, calloc, array allocation

2. **Ownership Inference** (50% → 20%)
   - Status: IN_PROGRESS
   - Target constructs: pointer classification, Box/Vec/& detection
   - Critical crate: `decy-ownership`

3. **Lifetime Inference** (20% → 10%)
   - Status: NOT_STARTED
   - Target constructs: lifetime annotations, scope analysis

4. **Safe Wrappers** (10% → <5%)
   - Status: NOT_STARTED
   - Target constructs: remaining unsafe wrapped in safe abstractions

### Current Unsafe Count

**Current: 0 unsafe blocks per 1000 LOC**
**Target: <5 unsafe blocks per 1000 LOC**

All validated constructs maintain zero unsafe blocks:
- malloc/free → Box (0 unsafe)
- sizeof → std::mem::size_of (0 unsafe)
- Arrays → [T; N] or Vec<T> (0 unsafe)

## Comparison with bashrs Methodology

| Aspect | bashrs | DECY |
|--------|--------|------|
| **Reference** | GNU Bash Manual | ISO C99 + K&R C |
| **Roadmap Format** | YAML with sections | YAML with chapters |
| **STOP THE LINE** | Andon Cord protocol | Andon Cord protocol |
| **TDD Approach** | EXTREME TDD | EXTREME TDD |
| **Completion** | 38% (18/120 tasks) | 12% (18/150 tasks) |
| **Test Types** | Unit + Property | Unit + Property + Coverage |
| **Quality Metric** | Determinism | Unsafe count + Coverage |

## Benefits of Validation Approach

1. **Systematic Coverage**
   - All C99 constructs documented
   - No guessing about edge cases
   - Clear roadmap to 100% coverage

2. **Bug Prevention**
   - STOP THE LINE catches bugs early
   - Reference-driven tests prevent regressions
   - P0 tickets force immediate fixes

3. **Audit Trail**
   - Every construct links to C99/K&R section
   - Transformation rationale documented
   - Unsafe blocks justified with references

4. **Quality Assurance**
   - EXTREME TDD ensures correctness
   - Property tests verify invariants
   - Coverage ≥80% enforced

5. **Project Management**
   - Clear progress tracking (12% → 100%)
   - Prioritized task list
   - Predictable sprint planning

## Next Steps

1. **Start Validation Campaign** (Sprint 2)
   - Begin with high-priority constructs (TYPE-PTR-NULL, STDLIB-CALLOC)
   - Apply STOP THE LINE protocol for all bugs
   - Target: 20% completion by end of Sprint 2

2. **Ownership Inference Focus**
   - Prioritize decy-ownership crate improvements
   - Target: 90% coverage for ownership inference
   - Validate pointer arithmetic elimination

3. **Weekly Validation Reviews**
   - Review validation progress in sprint retrospectives
   - Track unsafe block count trend
   - Update C-VALIDATION-ROADMAP.yaml

4. **Documentation**
   - Add validation examples to examples/
   - Document transformation patterns
   - Create C99/K&R cross-reference guide

## References

- `docs/C-VALIDATION-ROADMAP.yaml` - Complete validation roadmap
- `../bashrs/docs/BASH-INGESTION-ROADMAP.yaml` - Sister project methodology
- `CLAUDE.md` - Project guidelines with STOP THE LINE protocol
- ISO C99 Standard: https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf
- K&R C (2nd Edition) ISBN: 0131103628

---

**Status**: Active
**Created**: 2025-10-15
**Last Updated**: 2025-10-15
**Owner**: DECY Core Team
**Approval**: Following bashrs model
