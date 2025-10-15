# EXPR-ARITH-PTR Implementation Plan

**Status**: RED Phase Complete ✅
**Priority**: 5 (High)
**Sprint**: 3
**Ownership Inference Impact**: CRITICAL
**Unsafe Reduction**: Eliminates pointer arithmetic

## Overview

This document outlines the implementation plan for transforming C pointer arithmetic to safe Rust slice indexing, eliminating unsafe blocks.

**Goal**: Transform `ptr + offset` → `&arr[index + offset]` through ownership inference.

## Current State (RED Phase Complete)

### Failing Tests Created ✅

Created `/home/noahgift/src/decy/crates/decy-codegen/tests/pointer_arithmetic_safe_test.rs` with 9 tests:

1. ✅ `test_pointer_addition_to_slice_index` - FAILS (uses `unsafe { wrapping_add }`)
2. ✅ `test_pointer_subtraction_to_slice_index` - FAILS (uses `unsafe { wrapping_sub }`)
3. ✅ `test_pointer_difference_to_index_difference` - FAILS (uses `unsafe { offset_from }`)
4. ✅ `test_pointer_array_access_with_arithmetic` - FAILS (generates unsafe)
5. ✅ `test_pointer_arithmetic_in_loop` - FAILS (uses `unsafe { wrapping_add }`)
6. ✅ `test_pointer_arithmetic_negative_offset` - FAILS (generates unsafe)
7. ✅ `test_pointer_arithmetic_struct_array` - FAILS (generates unsafe)
8. ✅ `test_pointer_arithmetic_transformation_unsafe_count` - FAILS (found 2 unsafe blocks, expected 0)
9. ✅ `test_current_implementation_uses_unsafe` - PASSES (intentionally validates current unsafe implementation)

### Current Unsafe Implementation

Location: `/home/noahgift/src/decy/crates/decy-codegen/src/lib.rs:367-406`

```rust
// DECY-041: Detect pointer arithmetic using type context
if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
    if let HirExpression::Variable(var_name) = &**left {
        if ctx.is_pointer(var_name) {
            // This is pointer arithmetic - generate unsafe pointer method calls
            return match op {
                BinaryOperator::Add => {
                    format!(
                        "unsafe {{ {}.wrapping_add({} as usize) }}",
                        left_str, right_str
                    )
                }
                BinaryOperator::Subtract => {
                    // Check if right is also a pointer (ptr - ptr) or integer (ptr - offset)
                    if let HirExpression::Variable(right_var) = &**right {
                        if ctx.is_pointer(right_var) {
                            // ptr - ptr: calculate difference
                            format!(
                                "unsafe {{ {}.offset_from({}) as i32 }}",
                                left_str, right_str
                            )
                        } else {
                            // ptr - integer offset
                            format!(
                                "unsafe {{ {}.wrapping_sub({} as usize) }}",
                                left_str, right_str
                            )
                        }
                    } else {
                        // ptr - integer offset (literal or expression)
                        format!(
                            "unsafe {{ {}.wrapping_sub({} as usize) }}",
                            left_str, right_str
                        )
                    }
                }
                _ => unreachable!(),
            };
        }
    }
}
```

**Problem**: This generates 3 types of unsafe blocks:
1. `unsafe { ptr.wrapping_add(offset as usize) }` - pointer + integer
2. `unsafe { ptr.wrapping_sub(offset as usize) }` - pointer - integer
3. `unsafe { end.offset_from(start) as i32 }` - pointer - pointer

## Requirements for GREEN Phase

This is a **multi-crate, multi-phase implementation** that requires deep architectural changes:

### Phase 1: Ownership Inference Infrastructure (decy-ownership)

**Why Needed**: Must determine when a pointer is actually part of an array/slice.

**Requirements**:
1. Build pointer dataflow graph to track pointer origins
2. Detect array allocations (`malloc(n * sizeof(T))`, `int arr[N]`)
3. Classify pointers as:
   - `ArrayPointer { base: String, element_type: HirType }` - pointer into an array
   - `OwningPointer` - single allocation (Box)
   - `BorrowingPointer` - regular reference

**New Types Needed**:
```rust
// In decy-ownership crate
pub enum PointerClassification {
    ArrayPointer {
        base_array: String,         // Name of the array variable
        element_type: HirType,      // Element type
        base_index: Option<usize>,  // Known base index if constant
    },
    OwningPointer {
        pointee_type: HirType,
    },
    BorrowingPointer {
        is_mutable: bool,
        pointee_type: HirType,
    },
}

pub struct OwnershipContext {
    pointer_classifications: HashMap<String, PointerClassification>,
    array_allocations: HashMap<String, ArrayInfo>,
}
```

**Analysis Pass**:
```rust
impl OwnershipAnalyzer {
    pub fn analyze_pointer_usage(&mut self, hir: &HirModule) {
        // 1. Find all array allocations
        for stmt in &hir.statements {
            if let HirStatement::VariableDeclaration { name, var_type, initializer } = stmt {
                if let HirType::Array { element_type, size } = var_type {
                    self.record_array(name, element_type, size);
                }
                if let Some(HirExpression::FunctionCall { function, args }) = initializer {
                    if function == "malloc" && self.is_array_allocation(args) {
                        self.record_heap_array(name, ...);
                    }
                }
            }
        }

        // 2. Track pointer assignments
        for stmt in &hir.statements {
            if let HirStatement::VariableDeclaration { name, initializer } = stmt {
                if let Some(init_expr) = initializer {
                    if let Some(array_base) = self.extract_array_base(init_expr) {
                        self.classify_as_array_pointer(name, array_base);
                    }
                }
            }
        }

        // 3. Propagate classifications through assignments
        self.propagate_pointer_flow();
    }
}
```

### Phase 2: HIR Extensions (decy-hir)

**Why Needed**: Must represent slice indexing vs pointer arithmetic in HIR.

**New Expression Variants**:
```rust
pub enum HirExpression {
    // ... existing variants ...

    /// Slice indexing (safe): &arr[index]
    SliceIndex {
        slice: Box<HirExpression>,
        index: Box<HirExpression>,
    },

    /// Slice range: &arr[start..end]
    SliceRange {
        slice: Box<HirExpression>,
        start: Option<Box<HirExpression>>,
        end: Option<Box<HirExpression>>,
    },
}
```

**Transformation During HIR Construction**:
```rust
// In decy-hir, when building from AST
impl HirExpression {
    pub fn from_ast_with_ownership(
        ast_expr: &AstExpression,
        ownership: &OwnershipContext,
    ) -> Self {
        match ast_expr {
            AstExpression::BinaryOp { op: Add | Subtract, left, right } => {
                // Check if this is pointer arithmetic on an array pointer
                if let AstExpression::Variable(var_name) = left {
                    if let Some(PointerClassification::ArrayPointer { base_array, .. })
                        = ownership.get_classification(var_name)
                    {
                        // Transform: ptr + offset  →  SliceIndex { arr, base_index + offset }
                        return HirExpression::SliceIndex {
                            slice: Box::new(HirExpression::Variable(base_array.clone())),
                            index: Box::new(/* compute index from base + offset */),
                        };
                    }
                }
                // Fallback to normal binary op
                HirExpression::BinaryOp { /* ... */ }
            }
            _ => { /* ... */ }
        }
    }
}
```

### Phase 3: Code Generation (decy-codegen)

**Why Needed**: Generate safe Rust slice indexing instead of unsafe pointer arithmetic.

**Modifications**:
```rust
impl CodeGenerator {
    fn generate_expression_with_ownership(
        &self,
        expr: &HirExpression,
        ownership: &OwnershipContext,
    ) -> String {
        match expr {
            HirExpression::SliceIndex { slice, index } => {
                // Generate: arr[index]
                format!("{}[{}]",
                    self.generate_expression(slice),
                    self.generate_expression(index)
                )
            }

            HirExpression::BinaryOp { op: Add | Subtract, left, right } => {
                // If ownership analysis didn't catch this, fall back to unsafe
                // but ideally this should never happen
                if self.is_pointer_arithmetic(left, right, ownership) {
                    eprintln!("WARNING: Uncaught pointer arithmetic - generating unsafe");
                    // Generate current unsafe implementation
                } else {
                    // Normal arithmetic
                    format!("{} {} {}",
                        self.generate_expression(left),
                        self.generate_operator(op),
                        self.generate_expression(right)
                    )
                }
            }

            _ => { /* ... */ }
        }
    }
}
```

**Remove Current Unsafe Generation**:
```diff
- if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
-     if let HirExpression::Variable(var_name) = &**left {
-         if ctx.is_pointer(var_name) {
-             // This is pointer arithmetic - generate unsafe pointer method calls
-             return match op {
-                 BinaryOperator::Add => {
-                     format!(
-                         "unsafe {{ {}.wrapping_add({} as usize) }}",
-                         left_str, right_str
-                     )
-                 }
-                 // ... unsafe generation ...
-             };
-         }
-     }
- }
+ // Pointer arithmetic should be transformed to slice indexing at HIR level
+ // If we reach here with pointer arithmetic, it's a bug in ownership inference
```

### Phase 4: Type Signature Changes

**Why Needed**: Pointers that are array pointers should have slice type signatures.

**Function Signature Transformation**:
```rust
// Before: fn process(arr: *mut i32, index: i32) -> i32
// After:  fn process(arr: &[i32], index: usize) -> i32

impl CodeGenerator {
    fn generate_function_parameter(
        &self,
        param: &HirParameter,
        ownership: &OwnershipContext,
    ) -> String {
        if let HirType::Pointer(inner) = &param.param_type {
            if let Some(PointerClassification::ArrayPointer { element_type, .. })
                = ownership.get_classification(&param.name)
            {
                // Transform to slice parameter
                return format!("{}: &[{}]", param.name, self.map_type(element_type));
            }
        }
        // Default parameter generation
        format!("{}: {}", param.name, self.map_type(&param.param_type))
    }
}
```

## Implementation Complexity Assessment

### Effort Estimate
- **Phase 1 (Ownership Inference)**: 3-5 days
  - Build pointer dataflow analysis
  - Implement array allocation detection
  - Create pointer classification system
  - Write ownership inference tests

- **Phase 2 (HIR Extensions)**: 1-2 days
  - Add SliceIndex and SliceRange variants
  - Modify HIR construction to use ownership context
  - Update all HIR traversal code

- **Phase 3 (Code Generation)**: 1-2 days
  - Remove unsafe pointer arithmetic generation
  - Add slice indexing generation
  - Update type mapping

- **Phase 4 (Testing & Quality)**: 2-3 days
  - Ensure all 9 tests pass
  - Add property tests for ownership inference
  - Verify unsafe count is 0
  - Run mutation testing

**Total**: ~7-12 days of focused development

### Dependencies

This task depends on:
1. **Working parser** (✅ complete - can parse pointer arithmetic)
2. **Stable HIR representation** (✅ complete)
3. **Basic type system** (✅ complete)
4. **Ownership inference infrastructure** (❌ **NOT COMPLETE** - critical blocker)

### Blocking Issues

**CRITICAL**: The `decy-ownership` crate does not yet have the infrastructure needed for:
- Pointer dataflow analysis
- Array allocation detection
- Pointer classification system
- Ownership propagation

## Recommended Approach

### Option A: Build Foundation First (RECOMMENDED)

1. **Complete simpler ownership inference tasks first**:
   - PREP-INCLUDE (priority 11) - module system
   - PREP-DEFINE-MACRO (priority 12) - function macros
   - Other non-ownership tasks to build experience

2. **Build ownership inference foundation**:
   - Create pointer dataflow analysis framework in `decy-ownership`
   - Implement array allocation detection
   - Build pointer classification system
   - Test with simpler cases (malloc/free → Box already works)

3. **Return to EXPR-ARITH-PTR**:
   - With foundation in place, implement the transformation
   - All infrastructure will be ready
   - Higher chance of success

### Option B: Force Implementation Now (NOT RECOMMENDED)

Attempt to implement all phases simultaneously:
- Risk: High complexity, many moving parts
- Risk: May miss edge cases without proper foundation
- Risk: Technical debt if rushed
- Risk: May need to refactor later anyway

## Decision

**RECOMMENDATION**: Pause EXPR-ARITH-PTR at RED phase.

**Rationale**:
- Toyota Way: Don't rush without proper foundation
- RED phase tests provide clear acceptance criteria
- Building ownership inference foundation will make this task easier
- Other tasks can provide learning and infrastructure

**Next Steps**:
1. ✅ Commit RED phase tests (done)
2. ✅ Document requirements (this file)
3. Update roadmap: mark EXPR-ARITH-PTR as "blocked" pending ownership infrastructure
4. Continue with simpler tasks (priority 11+) to build experience
5. Create DECY tickets for ownership inference foundation work
6. Return to EXPR-ARITH-PTR in Sprint 3 when infrastructure is ready

## References

- **C Standard**: K&R §5.3, ISO C99 §6.5.6 (Pointer Arithmetic)
- **Current Implementation**: `decy-codegen/src/lib.rs:367-406`
- **RED Phase Tests**: `decy-codegen/tests/pointer_arithmetic_safe_test.rs`
- **Roadmap**: `docs/C-VALIDATION-ROADMAP.yaml` (EXPR-ARITH-PTR, priority 5)

## Related Tickets

**Should Create**:
- `DECY-046`: Build pointer dataflow analysis framework
- `DECY-047`: Implement array allocation detection
- `DECY-048`: Create pointer classification system
- `DECY-049`: Implement pointer arithmetic transformation (depends on DECY-046, DECY-047, DECY-048)

## Success Criteria (for GREEN Phase)

When returning to this task, all 9 tests must pass with:
- ✅ 0 unsafe blocks in generated code
- ✅ Safe slice indexing (`arr[index]`) instead of pointer arithmetic
- ✅ Correct type signatures (`&[T]` instead of `*mut T`)
- ✅ All quality gates passing (fmt, clippy, coverage ≥80%)
- ✅ Property tests for ownership inference
- ✅ Documentation with SAFETY comments if any unsafe remains
