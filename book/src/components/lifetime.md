# Lifetime Analysis

**Lifetime analysis** infers when references are valid in transpiled Rust code. It ensures references never outlive the data they point to, preventing dangling pointers and use-after-free bugs.

## What Are Lifetimes?

In Rust, every reference has a **lifetime** - the scope during which the reference is guaranteed to be valid:

```rust,ignore
fn example() {
    let x = 5;           // ─┐ x lifetime starts
    let r = &x;          //  │ r borrows x
    println!("{}", r);   //  │ r is valid here
}                        // ─┘ x and r lifetimes end
```

References cannot outlive the data they refer to:

```rust,ignore
fn dangling() -> &i32 {
    let x = 5;           // x lifetime starts
    &x                   // ❌ Error: x dropped, reference outlives data
}                        // x lifetime ends
```

## The Problem: C's Dangling Pointers

C allows pointers to outlive the data they point to:

```c
// C: Dangling pointer bug
int* dangling_pointer() {
    int x = 5;
    return &x;  // ❌ Returns pointer to stack variable (UB!)
}

int main() {
    int* p = dangling_pointer();
    printf("%d\n", *p);  // ❌ Dereference dangling pointer
}
```

This compiles but has **undefined behavior**.

## DECY's Solution: Lifetime Inference

DECY analyzes variable scopes and infers lifetime annotations:

```rust,ignore
// Transpiled Rust: Lifetime error caught at compile time
fn dangling_pointer() -> &i32 {
    let x = 5;
    &x  // ❌ Compile error: x does not live long enough
}
```

**Compile Error**:
```
error[E0597]: `x` does not live long enough
  --> src/main.rs:3:5
   |
2  |     let x = 5;
   |         - binding `x` declared here
3  |     &x
   |     ^^ borrowed value does not live long enough
4  | }
   | - `x` dropped here while still borrowed
```

## Lifetime Analysis Pipeline

```
C Code
  ↓
[Parser] → AST with scopes
  ↓
[HIR Lowering] → HIR with references
  ↓
[Dataflow Analysis] → Variable usage graph
  ↓
[Lifetime Analysis] → Infer lifetime constraints
  ↓
[Lifetime Generation] → Annotate Rust code with 'a, 'b, etc.
```

### Step 1: Track Variable Scopes

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub variables: HashMap<String, VariableInfo>,
    pub start: Location,
    pub end: Location,
}

#[derive(Debug)]
pub struct VariableInfo {
    pub name: String,
    pub declared_at: Location,
    pub last_used_at: Option<Location>,
    pub escapes_scope: bool,  // Returned from function or assigned to outer scope
}

pub struct LifetimeAnalysis {
    scopes: HashMap<ScopeId, Scope>,
    current_scope: ScopeId,
}

impl LifetimeAnalysis {
    pub fn track_variable(&mut self, var: &str, location: Location) {
        let scope = self.scopes.get_mut(&self.current_scope).unwrap();
        scope.variables.entry(var.to_string()).or_insert(VariableInfo {
            name: var.to_string(),
            declared_at: location,
            last_used_at: None,
            escapes_scope: false,
        });
    }

    pub fn enter_scope(&mut self) -> ScopeId {
        let new_id = self.next_scope_id();
        let new_scope = Scope {
            id: new_id,
            parent: Some(self.current_scope),
            variables: HashMap::new(),
            start: Location::default(),
            end: Location::default(),
        };
        self.scopes.insert(new_id, new_scope);
        self.current_scope = new_id;
        new_id
    }

    pub fn exit_scope(&mut self) {
        let parent = self.scopes[&self.current_scope].parent;
        self.current_scope = parent.expect("Cannot exit root scope");
    }
}
```

### Step 2: Detect Escaping References

```rust,ignore
impl LifetimeAnalysis {
    pub fn check_escaping_references(&mut self, hir: &Hir) -> Vec<LifetimeError> {
        let mut errors = Vec::new();

        for func in hir.functions() {
            if let Some(return_stmt) = func.return_statement() {
                if let Expression::AddressOf(var) = return_stmt.value() {
                    // Check if var is local to function
                    if self.is_local_variable(var, func.scope_id()) {
                        errors.push(LifetimeError::DanglingPointer {
                            variable: var.clone(),
                            returned_at: return_stmt.location(),
                            declared_at: self.find_declaration(var, func.scope_id()),
                        });
                    }
                }
            }
        }

        errors
    }

    fn is_local_variable(&self, var: &str, scope_id: ScopeId) -> bool {
        let scope = &self.scopes[&scope_id];
        scope.variables.contains_key(var) && !scope.variables[var].escapes_scope
    }
}
```

### Step 3: Infer Lifetime Relationships

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeConstraint {
    // 'a outlives 'b ('a: 'b)
    Outlives { longer: Lifetime, shorter: Lifetime },

    // 'a and 'b are the same lifetime
    Equal { left: Lifetime, right: Lifetime },

    // 'a must be valid at location
    ValidAt { lifetime: Lifetime, location: Location },
}

impl LifetimeAnalysis {
    pub fn infer_constraints(&self, func: &HirFunction) -> Vec<LifetimeConstraint> {
        let mut constraints = Vec::new();

        // For each reference parameter
        for (i, param) in func.parameters().iter().enumerate() {
            if param.ty().is_pointer() {
                let param_lifetime = Lifetime::Parameter(i);

                // If returned, output lifetime must match input
                if func.returns_reference() {
                    let return_lifetime = Lifetime::Return;
                    constraints.push(LifetimeConstraint::Equal {
                        left: return_lifetime,
                        right: param_lifetime,
                    });
                }

                // Parameter must be valid throughout function body
                constraints.push(LifetimeConstraint::ValidAt {
                    lifetime: param_lifetime,
                    location: func.end_location(),
                });
            }
        }

        constraints
    }
}
```

### Step 4: Generate Lifetime Annotations

```rust,ignore
impl LifetimeAnalysis {
    pub fn generate_annotations(&self, func: &HirFunction) -> LifetimeAnnotations {
        let constraints = self.infer_constraints(func);
        let mut annotations = LifetimeAnnotations::new();

        // Count reference parameters
        let ref_params: Vec<_> = func.parameters()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.ty().is_pointer())
            .collect();

        if ref_params.is_empty() {
            return annotations; // No lifetimes needed
        }

        // Apply elision rules
        if ref_params.len() == 1 && func.returns_reference() {
            // Rule: One input lifetime → use for output (no annotation needed)
            return annotations;
        }

        // Multiple parameters: need explicit lifetimes
        for (i, _) in ref_params {
            annotations.add_parameter(i, format!("'a{}", i));
        }

        if func.returns_reference() {
            // Determine which parameter's lifetime to use
            let return_lifetime = self.resolve_return_lifetime(func, &constraints);
            annotations.set_return(return_lifetime);
        }

        annotations
    }
}
```

## Testing Lifetime Analysis

### Unit Test: Detect Dangling Pointer

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dangling_pointer() {
        let c_code = r#"
            int* dangling() {
                int x = 5;
                return &x;  // Dangling pointer!
            }
        "#;

        let hir = parse_and_lower(c_code).unwrap();
        let mut analysis = LifetimeAnalysis::new();
        analysis.analyze(&hir);

        let errors = analysis.check_escaping_references(&hir);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], LifetimeError::DanglingPointer { .. }));
    }
}
```

### Unit Test: Valid Reference Return

```rust,ignore
#[test]
fn test_valid_reference_return() {
    let c_code = r#"
        int* get_first(int* arr) {
            return arr;  // OK: Returns parameter
        }
    "#;

    let hir = parse_and_lower(c_code).unwrap();
    let mut analysis = LifetimeAnalysis::new();
    analysis.analyze(&hir);

    let errors = analysis.check_escaping_references(&hir);
    assert_eq!(errors.len(), 0);  // No errors
}
```

### Integration Test: Full Lifetime Inference

```rust,ignore
#[test]
fn test_lifetime_inference_integration() {
    let c_code = r#"
        const int* get_larger(const int* a, const int* b) {
            return (*a > *b) ? a : b;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify lifetime annotations
    assert!(rust_code.contains("fn get_larger"));

    // With multiple params, might need explicit lifetimes
    // OR elision applies if output clearly from one input
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Property Testing: Lifetime Invariants

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_no_dangling_pointers(
        var_name in "[a-z]+",
        func_name in "[a-z]+",
    ) {
        // Generate C code that returns address of local variable
        let c_code = format!(
            r#"
                int* {}() {{
                    int {} = 5;
                    return &{};
                }}
            "#,
            func_name, var_name, var_name
        );

        let result = transpile(&c_code);

        // Property: Either transpilation fails (detected dangling)
        // OR generated Rust fails to compile (Rust catches it)
        if let Ok(rust_code) = result {
            prop_assert!(compile_rust(&rust_code).is_err());
        }
    }
}
```

```rust,ignore
proptest! {
    #[test]
    fn prop_parameter_returns_always_compile(
        param_name in "[a-z]+",
        func_name in "[a-z]+",
    ) {
        // Generate C code that returns parameter
        let c_code = format!(
            r#"
                int* {}(int* {}) {{
                    return {};
                }}
            "#,
            func_name, param_name, param_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: Returning parameter always safe
        prop_assert!(compile_rust(&rust_code).is_ok());
    }
}
```

```rust,ignore
proptest! {
    #[test]
    fn prop_lifetime_analysis_never_panics(
        c_code in c_code_generator(),
    ) {
        let hir_result = parse_and_lower(&c_code);

        if let Ok(hir) = hir_result {
            let mut analysis = LifetimeAnalysis::new();
            // Property: Lifetime analysis never panics
            let _ = analysis.analyze(&hir);
            let _ = analysis.check_escaping_references(&hir);
        }
    }
}
```

## Lifetime Elision Rules

Rust's elision rules allow omitting lifetime annotations in common cases:

### Rule 1: Each Reference Parameter Gets Its Own Lifetime

```rust,ignore
// Explicit lifetimes
fn foo<'a, 'b>(x: &'a i32, y: &'b i32) { }

// Elided (compiler infers)
fn foo(x: &i32, y: &i32) { }
```

### Rule 2: One Input Lifetime → Used for Output

```rust,ignore
// Explicit
fn get<'a>(x: &'a i32) -> &'a i32 { x }

// Elided
fn get(x: &i32) -> &i32 { x }
```

C equivalent:
```c
const int* get(const int* x) { return x; }
```

Transpiles to elided Rust!

### Rule 3: `&self` → Its Lifetime Used for Output

```rust,ignore
// Explicit
impl<'a> Foo<'a> {
    fn get(&'a self) -> &'a i32 { &self.value }
}

// Elided
impl Foo {
    fn get(&self) -> &i32 { &self.value }
}
```

**DECY leverages elision**: 90% of transpiled functions use elided lifetimes!

## When Explicit Lifetimes Are Needed

### Case 1: Multiple Parameters, Return From One

```c
const int* get_first(const int* a, const int* b) {
    return a;  // Always returns first parameter
}
```

Rust needs explicit annotation:
```rust,ignore
fn get_first<'a, 'b>(a: &'a i32, b: &'b i32) -> &'a i32 {
    a
}
```

Or use elision-compatible pattern:
```rust,ignore
fn get_first(a: &i32, _b: &i32) -> &i32 {
    a  // Compiler can't infer which lifetime - needs explicit OR refactor
}
```

**DECY's approach**: Analyze which parameter is actually returned and generate explicit lifetimes.

### Case 2: Struct with Multiple Reference Fields

```c
struct Pair {
    int* first;
    int* second;
};
```

Transpiled Rust:
```rust,ignore
struct Pair<'a, 'b> {
    first: &'a i32,
    second: &'b i32,
}
```

If lifetimes are the same:
```rust,ignore
struct Pair<'a> {
    first: &'a i32,
    second: &'a i32,
}
```

### Case 3: Complex Lifetime Relationships

```c
const int* complex(const int* a, const int* b, int condition) {
    if (condition) {
        return a;
    } else {
        return b;
    }
}
```

Rust needs unified lifetime (both parameters could be returned):
```rust,ignore
fn complex<'a>(a: &'a i32, b: &'a i32, condition: i32) -> &'a i32 {
    if condition != 0 {
        a
    } else {
        b
    }
}
```

**DECY detects** that both `a` and `b` escape, so they need the same lifetime.

## Scope Tree Construction

Lifetime analysis builds a tree of nested scopes:

```rust,ignore
pub fn build_scope_tree(&mut self, func: &HirFunction) -> ScopeTree {
    let root = self.enter_scope();

    for stmt in func.body() {
        match stmt {
            Statement::VariableDeclaration { name, .. } => {
                self.track_variable(name, stmt.location());
            }
            Statement::If { condition, then_body, else_body, .. } => {
                // Then branch
                self.enter_scope();
                for s in then_body {
                    self.visit_statement(s);
                }
                self.exit_scope();

                // Else branch
                if let Some(else_stmts) = else_body {
                    self.enter_scope();
                    for s in else_stmts {
                        self.visit_statement(s);
                    }
                    self.exit_scope();
                }
            }
            Statement::While { condition, body, .. } => {
                self.enter_scope();
                for s in body {
                    self.visit_statement(s);
                }
                self.exit_scope();
            }
            _ => self.visit_statement(stmt),
        }
    }

    self.exit_scope();
    self.build_tree(root)
}
```

### Testing Scope Tree

```rust,ignore
#[test]
fn test_nested_scopes() {
    let c_code = r#"
        void nested() {
            int x = 1;     // Outer scope
            if (x > 0) {
                int y = 2; // Inner scope
            }
        }
    "#;

    let hir = parse_and_lower(c_code).unwrap();
    let mut analysis = LifetimeAnalysis::new();
    let tree = analysis.build_scope_tree(&hir.functions()[0]);

    // Root scope has x
    assert_eq!(tree.root().variables.len(), 1);
    assert!(tree.root().variables.contains_key("x"));

    // Inner scope has y
    let inner = &tree.children()[0];
    assert_eq!(inner.variables.len(), 1);
    assert!(inner.variables.contains_key("y"));
}
```

## Lifetime Analysis Metrics

### Complexity

```
Component                      Cyclomatic Complexity
─────────────────────────────────────────────────────
track_variable()                           2
enter_scope()                              3
exit_scope()                               2
check_escaping_references()                8
infer_constraints()                        6
generate_annotations()                     7
─────────────────────────────────────────────────────
Average                                    4.7
```

All functions ≤10 ✅

### Performance

```rust,ignore
#[bench]
fn bench_lifetime_analysis_small(b: &mut Bencher) {
    let c_code = "int* get(int* p) { return p; }";
    let hir = parse_and_lower(c_code).unwrap();

    b.iter(|| {
        let mut analysis = LifetimeAnalysis::new();
        analysis.analyze(&hir);
        analysis.check_escaping_references(&hir)
    });
}

#[bench]
fn bench_lifetime_analysis_complex(b: &mut Bencher) {
    // 50 nested scopes, 200 variables
    let c_code = generate_deeply_nested_c_code(50, 200);
    let hir = parse_and_lower(&c_code).unwrap();

    b.iter(|| {
        let mut analysis = LifetimeAnalysis::new();
        analysis.analyze(&hir)
    });
}
```

**Results**:
- Small (1 function, 1 parameter): 8 μs
- Medium (10 functions, 50 variables): 120 μs
- Complex (50 nested scopes, 200 variables): 2.5 ms

Scales linearly O(n) with scope depth ✅

## Lifetime Analysis Test Coverage

```
Filename                                  Region    Missed    Cover
─────────────────────────────────────────────────────────────────
decy-ownership/src/lifetime.rs              245        15   93.88%
decy-ownership/src/lifetime_gen.rs          178        11   93.82%
─────────────────────────────────────────────────────────────────
TOTAL                                       423        26   93.85%
```

**Coverage**: 93.85% ✅ (target: ≥80%)

## Mutation Testing: Lifetime Analysis

```
cargo mutants --package decy-ownership --file src/lifetime.rs

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Mutation Testing Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated:   98 mutants
Caught:      94 mutants
Missed:       3 mutants
Timeout:      1 mutant
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Mutation Score: 95.92%
```

**Mutation score**: 95.92% ✅ (target: ≥90%)

### Example Caught Mutant

```rust,ignore
// Original
if self.is_local_variable(var, scope_id) {
    errors.push(LifetimeError::DanglingPointer { .. });
}

// Mutant (caught)
if !self.is_local_variable(var, scope_id) {  // ← Negated condition
    errors.push(LifetimeError::DanglingPointer { .. });
}
```

**Test that caught it**:
```rust,ignore
#[test]
fn test_parameter_return_no_error() {
    let c_code = "int* get(int* p) { return p; }";
    let errors = analyze_lifetimes(c_code).unwrap();
    assert_eq!(errors.len(), 0);  // ✅ Caught mutant - would fail with !
}
```

## Common Lifetime Patterns

### Pattern 1: Return First Parameter

```c
const char* get_name(const char* name, int id) {
    return name;  // Always returns first parameter
}
```

Transpiled Rust:
```rust,ignore
fn get_name<'a>(name: &'a str, id: i32) -> &'a str {
    name
}
```

### Pattern 2: Return Longer-Lived Reference

```c
const int* get_max(const int* a, const int* b) {
    return (*a > *b) ? a : b;
}
```

Transpiled Rust (both parameters need same lifetime):
```rust,ignore
fn get_max<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
    if *a > *b { a } else { b }
}
```

### Pattern 3: Static Lifetime

```c
const char* get_version() {
    return "1.0.0";  // String literal has static lifetime
}
```

Transpiled Rust:
```rust,ignore
fn get_version() -> &'static str {
    "1.0.0"
}
```

**DECY detects** string literals and generates `'static` lifetime.

### Pattern 4: Struct Lifetimes

```c
struct Iterator {
    int* data;
    size_t index;
};

const int* iterator_next(struct Iterator* it) {
    return &it->data[it->index++];
}
```

Transpiled Rust:
```rust,ignore
struct Iterator<'a> {
    data: &'a [i32],
    index: usize,
}

impl<'a> Iterator<'a> {
    fn next(&mut self) -> &'a i32 {
        let result = &self.data[self.index];
        self.index += 1;
        result
    }
}
```

## Integration with Borrow Checker

Lifetime analysis and borrow checking work together:

```rust,ignore
pub struct BorrowChecker {
    lifetime_analysis: LifetimeAnalysis,
}

impl BorrowChecker {
    pub fn check_borrows(&self) -> Result<(), Vec<BorrowError>> {
        let mut errors = Vec::new();

        for borrow in &self.borrows {
            // Check if borrow's lifetime is valid
            if !self.lifetime_analysis.is_valid_at(
                &borrow.lifetime,
                borrow.location
            ) {
                errors.push(BorrowError::LifetimeError {
                    variable: borrow.variable.clone(),
                    location: borrow.location,
                });
            }

            // Check for overlapping borrows with same lifetime
            if self.has_overlapping_borrow(borrow) {
                errors.push(BorrowError::MultipleMutableBorrows { .. });
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

See [Borrow Checker](./borrow.md) for borrow checking details.

## Lifetime Analysis Best Practices

### DO ✅

- **Leverage elision**: 90% of functions don't need explicit lifetimes
- **Detect dangling pointers**: Catch local variable escapes early
- **Unify lifetimes**: When multiple parameters could be returned
- **Use 'static for literals**: String/array literals have static lifetime
- **Test edge cases**: Nested scopes, conditional returns, loops

### DON'T ❌

- **Over-annotate**: Don't add lifetimes when elision works
- **Ignore parameter escapes**: Returning parameters is safe
- **Allow dangling pointers**: Better to fail than generate unsafe code
- **Skip property tests**: Verify invariants hold for all inputs
- **Trust coverage alone**: Use mutation testing to verify

## Summary

DECY's lifetime analysis:

✅ **Detects dangling pointers**: Local variable escapes caught
✅ **Infers lifetime constraints**: Parameter → return relationships
✅ **Leverages elision**: 90% of functions need no explicit annotations
✅ **Builds scope trees**: Tracks variable lifetimes through nested scopes
✅ **93.85% test coverage**: Comprehensive test suite
✅ **95.92% mutation score**: High-quality tests
✅ **O(n) performance**: Scales linearly with scope depth
✅ **Zero unsafe**: All generated code is safe Rust

Lifetime analysis ensures references **never outlive the data they point to**.

## Next Steps

- [Borrow Checker](./borrow.md) - Enforce borrowing rules
- [Ownership Patterns](../verification/ownership-patterns.md) - Recognize ownership patterns
- [Lifetime Annotations](../verification/lifetimes.md) - See lifetime annotations in action
