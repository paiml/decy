# Ownership Inference

Ownership inference determines which Rust ownership pattern (owning, borrowed, or raw) best represents each C pointer.

## Ownership Patterns

### Three Categories

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipPattern {
    // Variable owns its data (Box<T>)
    Owning,

    // Variable borrows data (&T or &mut T)
    Borrowed,

    // Unsafe raw pointer (*const T or *mut T)
    Raw,
}
```

### Pattern Mapping

| C Pattern | Ownership | Rust Type |
|-----------|-----------|-----------|
| `malloc` → variable | Owning | `Box<T>` |
| Function parameter pointer | Borrowed | `&T` or `&mut T` |
| Returned pointer | Owning | `Box<T>` |
| Pointer arithmetic | Raw | `*mut T` |
| NULL checks | Borrowed | `Option<&T>` |

## Inference Algorithm

### Step 1: Classify by Source

```rust,ignore
fn classify_by_source(var: &str, graph: &DataflowGraph) -> OwnershipPattern {
    let source = graph.get_source(var);

    match source {
        Source::Malloc => OwnershipPattern::Owning,
        Source::Parameter => OwnershipPattern::Borrowed,
        Source::AddressOf(_) => OwnershipPattern::Borrowed,
        Source::PointerArithmetic => OwnershipPattern::Raw,
        _ => OwnershipPattern::Raw,
    }
}
```

### Verification Test

```rust,ignore
#[test]
fn test_malloc_is_owning() {
    let c_code = "int* p = malloc(sizeof(int));";

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let ownership = infer_ownership(&graph);

    assert_eq!(ownership.get("p"), Some(&OwnershipPattern::Owning));
}

#[test]
fn test_parameter_is_borrowed() {
    let c_code = "void func(int* p) { *p = 10; }";

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let ownership = infer_ownership(&graph);

    assert_eq!(ownership.get("p"), Some(&OwnershipPattern::Borrowed));
}
```

## Mutability Inference

Determine if borrows should be mutable:

```rust,ignore
fn infer_mutability(var: &str, graph: &DataflowGraph) -> bool {
    // Check if variable is ever assigned to
    for node in graph.nodes() {
        if let DataflowNode::Assign { target, .. } = node {
            if target == var {
                return true;  // Mutable
            }
        }
        if let DataflowNode::Dereference { var: v } = node {
            if v == var && graph.is_lvalue(node) {
                return true;  // Mutable (used as lvalue)
            }
        }
    }
    false  // Immutable
}
```

### Verification Test

```rust,ignore
#[test]
fn test_infer_mutable_borrow() {
    let c_code = "void increment(int* p) { *p = *p + 1; }";

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let analysis = OwnershipAnalysis::new(&graph);

    let info = analysis.analyze_variable("p").unwrap();

    assert_eq!(info.pattern, OwnershipPattern::Borrowed);
    assert!(info.is_mutable);  // Should be &mut, not &
}

#[test]
fn test_infer_immutable_borrow() {
    let c_code = "int get_value(const int* p) { return *p; }";

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let analysis = OwnershipAnalysis::new(&graph);

    let info = analysis.analyze_variable("p").unwrap();

    assert_eq!(info.pattern, OwnershipPattern::Borrowed);
    assert!(!info.is_mutable);  // Should be &, not &mut
}
```

## Confidence Scoring

Assign confidence to each inference:

```rust,ignore
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub pattern: OwnershipPattern,
    pub is_mutable: bool,
    pub confidence: f64,  // 0.0 to 1.0
    pub reasoning: Vec<String>,
}

fn calculate_confidence(var: &str, graph: &DataflowGraph) -> f64 {
    let mut confidence = 0.5;  // Start neutral

    // Strong indicators increase confidence
    if has_malloc_call(var, graph) {
        confidence += 0.4;  // malloc → definitely owning
    }
    if is_function_parameter(var, graph) {
        confidence += 0.3;  // parameters → likely borrowed
    }

    // Weak indicators
    if has_null_check(var, graph) {
        confidence += 0.1;
    }

    confidence.min(1.0)
}
```

### Verification Test

```rust,ignore
#[test]
fn test_confidence_scores() {
    let test_cases = vec![
        ("int* p = malloc(sizeof(int));", 0.9),  // High confidence
        ("void func(int* p) {}", 0.8),            // High confidence
        ("int* p;", 0.5),                         // Low confidence (unknown)
    ];

    for (c_code, expected_min_confidence) in test_cases {
        let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
        let graph = DataflowGraph::from_hir(&hir).unwrap();
        let analysis = OwnershipAnalysis::new(&graph);
        let info = analysis.analyze_variable("p").unwrap();

        assert!(
            info.confidence >= expected_min_confidence,
            "Expected confidence ≥ {}, got {}",
            expected_min_confidence,
            info.confidence
        );
    }
}
```

## Property Tests

### Property: malloc Always Owning

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_malloc_always_owning(var_name in "[a-z]+", size in 1..1024usize) {
        let c_code = format!("{}* {} = malloc({});", "int", var_name, size);

        let hir = lower_to_hir(&parse(&c_code).unwrap()).unwrap();
        let graph = DataflowGraph::from_hir(&hir).unwrap();
        let ownership = infer_ownership(&graph);

        // Property: All malloc allocations are owning
        prop_assert_eq!(
            ownership.get(&var_name),
            Some(&OwnershipPattern::Owning)
        );
    }
}
```

### Property: Function Parameters Borrowed

```rust,ignore
proptest! {
    #[test]
    fn prop_parameters_borrowed(
        func_name in "[a-z]+",
        param_name in "[a-z]+",
    ) {
        let c_code = format!(
            "void {}(int* {}) {{ *{} = 0; }}",
            func_name, param_name, param_name
        );

        let hir = lower_to_hir(&parse(&c_code).unwrap()).unwrap();
        let graph = DataflowGraph::from_hir(&hir).unwrap();
        let ownership = infer_ownership(&graph);

        // Property: Function parameters are borrowed
        prop_assert_eq!(
            ownership.get(&param_name),
            Some(&OwnershipPattern::Borrowed)
        );
    }
}
```

## Escape Analysis

Determine if a variable escapes its scope:

```rust,ignore
fn escapes_scope(var: &str, graph: &DataflowGraph) -> bool {
    for node in graph.nodes() {
        if let DataflowNode::Return(expr) = node {
            if expr.contains_var(var) {
                return true;  // Returned from function
            }
        }
        if let DataflowNode::Call { args, .. } = node {
            if args.contains(&var.to_string()) {
                return true;  // Passed to another function
            }
        }
    }
    false
}
```

### Verification Test

```rust,ignore
#[test]
fn test_escaping_variable() {
    let c_code = r#"
        int* create_int() {
            int* p = malloc(sizeof(int));
            return p;  // p escapes!
        }
    "#;

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let analysis = OwnershipAnalysis::new(&graph);
    let info = analysis.analyze_variable("p").unwrap();

    assert!(info.escapes_scope);
    assert_eq!(info.pattern, OwnershipPattern::Owning);
}

#[test]
fn test_non_escaping_variable() {
    let c_code = r#"
        void process() {
            int* p = malloc(sizeof(int));
            free(p);  // p does not escape
        }
    "#;

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let analysis = OwnershipAnalysis::new(&graph);
    let info = analysis.analyze_variable("p").unwrap();

    assert!(!info.escapes_scope);
}
```

## Reasoning Trace

Provide human-readable explanations:

```rust,ignore
#[test]
fn test_inference_reasoning() {
    let c_code = "int* p = malloc(sizeof(int));";

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let analysis = OwnershipAnalysis::new(&graph);
    let info = analysis.analyze_variable("p").unwrap();

    assert_eq!(info.pattern, OwnershipPattern::Owning);
    assert!(info.reasoning.contains(&"Allocated with malloc".to_string()));
    assert!(info.reasoning.contains(&"No evidence of borrowing".to_string()));
}
```

## Integration Test

Complete ownership inference pipeline:

```rust,ignore
#[test]
fn test_end_to_end_ownership_inference() {
    let c_code = r#"
        int* create_and_modify(int* input) {
            int* output = malloc(sizeof(int));
            *output = *input * 2;
            return output;
        }
    "#;

    let hir = lower_to_hir(&parse(c_code).unwrap()).unwrap();
    let graph = DataflowGraph::from_hir(&hir).unwrap();
    let ownership = infer_ownership(&graph);

    // input: borrowed (parameter, not modified)
    assert_eq!(
        ownership.get("input"),
        Some(&OwnershipPattern::Borrowed)
    );

    // output: owning (malloc, returned)
    assert_eq!(
        ownership.get("output"),
        Some(&OwnershipPattern::Owning)
    );
}
```

## Summary

Ownership inference provides:

✅ **Pattern classification**: Owning, Borrowed, or Raw
✅ **Mutability detection**: &T vs &mut T
✅ **Confidence scoring**: How certain is the inference?
✅ **Escape analysis**: Does the variable leave its scope?
✅ **Reasoning traces**: Why was this pattern chosen?

## Next Steps

- [Borrow Generation](./borrow.md) - Converting inferred patterns to Rust code
- [Lifetime Analysis](./lifetime.md) - Determining lifetime annotations
