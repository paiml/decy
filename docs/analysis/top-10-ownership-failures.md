# Top-10 Ownership Inference Failures Analysis (DECY-ML-006)

**Version**: 1.0.0
**Status**: COMPLETED
**Sprint**: N+1 (Phase 2: Quality Foundation)
**Story Points**: 3

---

## Executive Summary

This analysis identifies and categorizes the top-10 ownership inference failures in the Decy C-to-Rust transpiler. Data is sourced from:

1. **Coreutils corpus**: 500 functions, 9,108 errors (oracle-integration-spec.md)
2. **Reprorusted-c-cli corpus**: 47 functions, 1,328 errors (training-oracle-spec.md)
3. **ML defect taxonomy**: DECY-O-001 through DECY-O-008 (improvements-ml-techniques.md)

**Key Finding**: 74% of all transpilation errors are ownership/lifetime-related, validating the critical importance of the `decy-ownership` crate.

---

## Top-10 Failure Patterns (Ranked by Frequency)

### 1. E0506: Cannot Assign to Borrowed Variable (31%)

| Metric | Value |
|--------|-------|
| Frequency | 31% (2,847 coreutils occurrences) |
| Root Cause | Reassignment during loop iterations with active borrows |
| DECY Category | DECY-O-004 (AliasViolation) |
| Fix Strategy | Restructure lifetimes with scope splits |

**C Pattern (problematic)**:
```c
void process(int* arr, size_t n) {
    for (size_t i = 0; i < n; i++) {
        arr[i] = transform(arr[i]);  // borrow conflict
    }
}
```

**Rust Error**:
```
error[E0506]: cannot assign to `arr[i]` because it is borrowed
```

**Resolution**: Use indexing with explicit scopes or iterator patterns.

---

### 2. E0499: Multiple Mutable Borrows (21%)

| Metric | Value |
|--------|-------|
| Frequency | 21% (1,923 coreutils occurrences) |
| Root Cause | Multiple `&mut` references from same pointer |
| DECY Category | DECY-O-004 (AliasViolation) |
| Fix Strategy | Use RefCell, Mutex, or restructure ownership |

**C Pattern (problematic)**:
```c
void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}
// Called as: swap(&arr[i], &arr[j])
```

**Resolution**: Use `std::mem::swap` or split borrows with temporary variables.

---

### 3. E0382: Use After Move (16%)

| Metric | Value |
|--------|-------|
| Frequency | 16% (1,456 coreutils occurrences) |
| Root Cause | Reuse after `free()` equivalent or value move |
| DECY Category | DECY-O-003 (DanglingPointerRisk) |
| Fix Strategy | Clone or restructure ownership |

**C Pattern (problematic)**:
```c
char* buf = malloc(128);
strcpy(dest, buf);  // buf still usable in C
free(buf);
// buf used again later
```

**Resolution**: Explicit lifetime tracking, consider `Rc<T>` for shared ownership.

---

### 4. E0308: Type Mismatch (10%)

| Metric | Value |
|--------|-------|
| Frequency | 10% (892 coreutils occurrences) |
| Root Cause | C-specific: `int*` vs `size_t` confusion, cast failures |
| DECY Category | DECY-O-001 (PointerMisclassification) |
| Fix Strategy | Add dereference or change borrow type |

**C Pattern (problematic)**:
```c
void process(int* data) {
    size_t offset = (size_t)data;  // pointer arithmetic
}
```

**Resolution**: Use `usize` for pointer-sized integers, explicit conversions.

---

### 5. E0133: Unsafe Required (7%)

| Metric | Value |
|--------|-------|
| Frequency | 7% (634 coreutils occurrences) |
| Root Cause | C raw pointers where safe abstraction not detected |
| DECY Category | DECY-O-005 (UnsafeMinimizationFailure) |
| Fix Strategy | Wrap in safe abstraction or use raw pointer |

**C Pattern (problematic)**:
```c
void* memcpy(void* dest, const void* src, size_t n);
```

**Resolution**: Use safe wrappers (`slice::copy_from_slice`), minimize unsafe blocks.

---

### 6. E0597: Does Not Live Long Enough (6%)

| Metric | Value |
|--------|-------|
| Frequency | 6% (521 coreutils occurrences) |
| Root Cause | Reference lifetime doesn't match referent lifetime |
| DECY Category | DECY-O-002 (LifetimeInferenceGap) |
| Fix Strategy | Extend lifetime or clone |

**C Pattern (problematic)**:
```c
char* get_temp_buffer() {
    char buf[128];
    return buf;  // dangling pointer
}
```

**Resolution**: Return owned value, use `'static` or heap allocation.

---

### 7. E0515: Cannot Return Reference to Local (5%)

| Metric | Value |
|--------|-------|
| Frequency | 5% (423 coreutils occurrences) |
| Root Cause | Returning reference to stack-allocated data |
| DECY Category | DECY-O-002 (LifetimeInferenceGap) |
| Fix Strategy | Return owned value instead |

**C Pattern (problematic)**:
```c
int* get_default() {
    int val = 42;
    return &val;  // dangling reference
}
```

**Resolution**: Return by value or use Box/Vec for heap allocation.

---

### 8. DECY-O-001: Pointer Misclassification (Decy-Specific)

| Metric | Value |
|--------|-------|
| Frequency | Estimated 8-12% of ML inference errors |
| Root Cause | Owning pointer classified as borrowing or vice versa |
| Example | `malloc` result treated as `&T` instead of `Box<T>` |
| Fix Strategy | Improve allocation-site detection in dataflow |

**Misclassification Matrix**:
| Actual | Predicted Box | Predicted &T | Predicted &mut T |
|--------|---------------|--------------|------------------|
| Box<T> | ✓ | High Error | Moderate Error |
| &T | Moderate | ✓ | High Error |
| &mut T | Low Error | High Error | ✓ |

---

### 9. DECY-O-004: Alias Violation (Decy-Specific)

| Metric | Value |
|--------|-------|
| Frequency | Subsumed under E0499/E0506 (~52% combined) |
| Root Cause | Multiple mutable aliases generated |
| Example | Two `&mut T` to same memory location |
| Fix Strategy | Enhanced alias analysis using dataflow graphs |

**Detection Challenge**: C allows arbitrary pointer aliasing. Detecting non-aliasing requires:
1. Restrict-qualifier analysis
2. Points-to analysis from control flow
3. Function signature analysis (`restrict` parameters)

---

### 10. DECY-O-008: Mutability Mismatch (Decy-Specific)

| Metric | Value |
|--------|-------|
| Frequency | Estimated 3-5% of inference errors |
| Root Cause | `const` pointer vs mutable reference error |
| Example | `const int*` → `&mut i32` |
| Fix Strategy | Propagate const-correctness through dataflow |

**Common Patterns**:
```c
// C allows this
void modify(const int* ptr) {
    int* mut_ptr = (int*)ptr;  // cast away const
    *mut_ptr = 42;
}
```

---

## Correlation Analysis

### Top-3 Patterns (68% of Errors)

```
┌────────────────────────────────────────────────────────────┐
│     E0506 (31%)     │     E0499 (21%)     │  E0382 (16%) │
│   Borrow Conflict   │  Multiple &mut T    │ Use After Move│
│         ├───────────────────┤                              │
│         │  Alias Detection  │                              │
│         │   (Root Cause)    │                              │
│         └───────────────────┘                              │
└────────────────────────────────────────────────────────────┘
```

**Shared Root Cause**: Insufficient alias/ownership tracking in dataflow analysis.

### ML Feature Importance (Hypothesized)

Based on error distribution, prioritize features:

1. **allocation_site** - malloc/calloc/stack detection (E0382, DECY-O-001)
2. **deallocation_count** - free() tracking (E0382)
3. **alias_count** - pointer aliasing (E0499, E0506, DECY-O-004)
4. **write_count** - mutation tracking (E0506, DECY-O-008)
5. **escape_scope** - lifetime detection (E0597, E0515)

---

## Success Rate by Pattern

From training-oracle-spec.md corpus analysis:

| Error Code | Pattern Count | Success Rate | Improvement Potential |
|------------|---------------|--------------|----------------------|
| E0506 | 78 patterns | 85% | +10% with loop analysis |
| E0499 | 52 patterns | 88% | +7% with alias detection |
| E0382 | 41 patterns | 82% | +13% with dataflow tracking |
| E0308 | 32 patterns | 75% | +15% with type inference |
| E0133 | 18 patterns | 70% | +20% with pattern library |
| E0597 | 15 patterns | 80% | +15% with lifetime inference |
| E0515 | 11 patterns | 78% | +17% with escape analysis |

**Target**: Increase average success rate from ~80% to 90%+ through ML enhancement.

---

## Recommendations

### Immediate Actions (Sprint N+1)

1. **Enhance alias tracking** in `decy-ownership/src/dataflow.rs`
   - Add points-to analysis
   - Track `restrict` qualifiers

2. **Improve allocation detection** in `decy-ownership/src/inference.rs`
   - Pattern match `malloc/calloc/realloc/free` chains
   - Handle wrapper functions (e.g., `xmalloc`)

3. **Add loop-aware borrowing** in lifetime inference
   - Detect borrow conflicts across iterations
   - Generate iterator-based alternatives

### ML Integration (Sprint N+2)

1. Train classifier on 142-dimension feature vectors
2. Use Tarantula scores to weight training samples
3. Apply hybrid classification with 0.65 confidence threshold

---

## References

- `/home/noah/src/decy/docs/specifications/oracle-integration-spec.md`
- `/home/noah/src/decy/docs/specifications/training-oracle-spec.md`
- `/home/noah/src/decy/docs/specifications/improvements-ml-techniques.md`
- `/home/noah/src/decy/crates/decy-ownership/src/ml_features.rs`
- Jones & Harrold, "Tarantula Automatic Fault-Localization" (ASE 2005)
