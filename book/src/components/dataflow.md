# Dataflow Analysis

Dataflow analysis tracks how data flows through a program to understand variable usage, dependencies, and potential issues like use-after-free.

## Purpose

Dataflow analysis enables:

1. **Ownership inference**: Determine which variables own their data
2. **Lifetime analysis**: Track when variables are created and destroyed
3. **Safety checks**: Detect use-after-free, double-free, null dereferences
4. **Optimization**: Identify dead code and unused variables

## Architecture

```
HIR → Dataflow Graph Builder → Dataflow Graph → Analysis → Insights
```

## Dataflow Graph

### Node Types

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub enum DataflowNode {
    // Variable declaration
    Decl { name: String, ty: HirType },

    // Assignment to variable
    Assign { target: String, source: DataflowValue },

    // Function call
    Call { name: String, args: Vec<String> },

    // Memory operation
    Malloc { var: String, size: usize },
    Free { var: String },

    // Pointer operation
    Dereference { var: String },
    AddressOf { var: String },
}
```

### Example Graph

For this C code:

```c
int* create_array(int size) {
    int* arr = malloc(size * sizeof(int));
    arr[0] = 10;
    return arr;
}
```

The dataflow graph:

```
[1] Decl(arr, int*)
[2] Malloc(arr, size*4)
[3] Assign(arr[0], 10)
[4] Return(arr)

Dependencies:
[2] → [1]  (malloc depends on declaration)
[3] → [2]  (assignment depends on malloc)
[4] → [3]  (return depends on assignment)
```

## Building the Graph

### Verification Test

```rust,ignore
#[test]
fn test_build_dataflow_graph() {
    let hir_func = create_malloc_function();

    let graph = DataflowGraph::from_hir(&hir_func).unwrap();

    // Verify nodes
    assert_eq!(graph.nodes().len(), 4);

    // Verify dependencies
    assert!(graph.has_edge(1, 0));  // malloc → decl
    assert!(graph.has_edge(2, 1));  // assign → malloc
    assert!(graph.has_edge(3, 2));  // return → assign
}
```

## Use-After-Free Detection

```rust,ignore
#[test]
fn test_detect_use_after_free() {
    let c_code = r#"
        void bad_function() {
            int* p = malloc(sizeof(int));
            free(p);
            *p = 10;  // Use after free!
        }
    "#;

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();

    let analysis = DataflowAnalysis::new(&graph);
    let errors = analysis.check_safety();

    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], SafetyError::UseAfterFree { .. }));
}
```

## Pointer Tracking

Track pointer allocations and deallocations:

```rust,ignore
#[test]
fn test_track_pointer_lifecycle() {
    let c_code = r#"
        int* create_int() {
            int* p = malloc(sizeof(int));
            *p = 42;
            return p;
        }
    "#;

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();

    let tracker = PointerTracker::new(&graph);
    let info = tracker.analyze_variable("p").unwrap();

    assert!(info.is_allocated);
    assert!(!info.is_freed);
    assert!(info.escapes_scope);  // Returned from function
}
```

## Property Tests

### Property: Graph is Acyclic

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_dataflow_graph_is_dag(hir_func in valid_hir_function()) {
        let graph = DataflowGraph::from_hir(&hir_func).unwrap();

        // Property: Dataflow graph must be a DAG (no cycles)
        prop_assert!(!graph.has_cycle());
    }
}
```

### Property: All Variables Declared Before Use

```rust,ignore
proptest! {
    #[test]
    fn prop_variables_declared_before_use(hir_func in valid_hir_function()) {
        let graph = DataflowGraph::from_hir(&hir_func).unwrap();

        // Property: Every use has a prior declaration
        for node in graph.nodes() {
            if let DataflowNode::Assign { target, .. } = node {
                prop_assert!(
                    graph.has_prior_declaration(target),
                    "Variable {} used before declaration", target
                );
            }
        }
    }
}
```

## Summary

Dataflow analysis provides:

✅ **Variable tracking**: Follow data through the program
✅ **Safety checks**: Detect use-after-free, double-free
✅ **Ownership hints**: Identify owning vs borrowed variables
✅ **Dependency graph**: Understand variable relationships

## Next Steps

- [Ownership Inference](./ownership.md) - Using dataflow to infer ownership
- [Borrow Generation](./borrow.md) - Converting pointers to borrows
