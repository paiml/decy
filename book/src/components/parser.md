# Parser Verification

The parser is the first stage of the DECY transpiler pipeline. It converts C source code into an Abstract Syntax Tree (AST).

## Architecture

```
C Source Code → tree-sitter → CST → AST Builder → AST
```

DECY uses `tree-sitter-c` for robust, error-tolerant parsing.

## Basic Function Parsing

### C Input

```c
int add(int a, int b) {
    return a + b;
}
```

### Expected AST Structure

```rust,ignore
Function {
    name: "add",
    return_type: Type::Int,
    parameters: [
        Parameter { name: "a", ty: Type::Int },
        Parameter { name: "b", ty: Type::Int },
    ],
    body: Block {
        statements: [
            Return(BinaryOp {
                op: Add,
                left: Var("a"),
                right: Var("b"),
            })
        ]
    }
}
```

### Verification Test

```rust,ignore
use decy_parser::CParser;

#[test]
fn test_parse_simple_function() {
    let c_code = "int add(int a, int b) { return a + b; }";

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.name(), "add");
    assert_eq!(func.parameters().len(), 2);
    assert_eq!(func.parameters()[0].name(), "a");
    assert_eq!(func.parameters()[1].name(), "b");
}
```

## Variable Declarations

### C Input

```c
int main() {
    int x = 5;
    int y;
    y = x + 10;
    return y;
}
```

### Verification Test

```rust,ignore
#[test]
fn test_parse_variable_declarations() {
    let c_code = r#"
        int main() {
            int x = 5;
            int y;
            y = x + 10;
            return y;
        }
    "#;

    let parser = CParser::new().unwrap();
    let ast = parser.parse(c_code).unwrap();

    let func = &ast.functions()[0];
    let statements = func.body().statements();

    // Check for variable declaration with initializer
    assert!(matches!(statements[0], Statement::VarDecl { .. }));

    // Check for variable declaration without initializer
    assert!(matches!(statements[1], Statement::VarDecl { .. }));

    // Check for assignment
    assert!(matches!(statements[2], Statement::Assignment { .. }));

    // Check for return
    assert!(matches!(statements[3], Statement::Return(_)));
}
```

## Pointer Declarations

### C Input

```c
int* create_int() {
    int* p = malloc(sizeof(int));
    *p = 42;
    return p;
}
```

### Verification Test

```rust,ignore
#[test]
fn test_parse_pointers() {
    let c_code = r#"
        int* create_int() {
            int* p = malloc(sizeof(int));
            *p = 42;
            return p;
        }
    "#;

    let parser = CParser::new().unwrap();
    let ast = parser.parse(c_code).unwrap();

    let func = &ast.functions()[0];

    // Check return type is pointer
    assert!(matches!(func.return_type(), Type::Pointer(_)));

    // Check malloc call
    let statements = func.body().statements();
    if let Statement::VarDecl { initializer: Some(init), .. } = &statements[0] {
        assert!(matches!(init, Expression::Call { name, .. } if name == "malloc"));
    } else {
        panic!("Expected variable declaration with malloc initializer");
    }

    // Check dereference assignment
    assert!(matches!(statements[1], Statement::Assignment {
        target: Expression::Dereference(_),
        ..
    }));
}
```

## Property Tests for Parser

### Property: Parser Never Panics

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_parser_never_panics(input in "\\PC*") {
        let parser = CParser::new().unwrap();
        // Should never panic, even with garbage input
        let _ = parser.parse(&input);
    }
}
```

### Property: Valid C Always Parses

```rust,ignore
proptest! {
    #[test]
    fn prop_valid_c_parses(
        func_name in "[a-z_][a-z0-9_]{2,20}",
        param_name in "[a-z_][a-z0-9_]{2,20}",
    ) {
        let c_code = format!(
            "int {}(int {}) {{ return {}; }}",
            func_name, param_name, param_name
        );

        let parser = CParser::new().unwrap();
        let result = parser.parse(&c_code);

        prop_assert!(result.is_ok(), "Valid C should parse successfully");

        let ast = result.unwrap();
        prop_assert_eq!(ast.functions().len(), 1);
        prop_assert_eq!(ast.functions()[0].name(), &func_name);
    }
}
```

### Property: Parse-Print Roundtrip

```rust,ignore
proptest! {
    #[test]
    fn prop_parse_print_roundtrip(c_code in valid_c_function()) {
        let parser = CParser::new().unwrap();

        let ast1 = parser.parse(&c_code).unwrap();
        let printed = ast1.to_string();
        let ast2 = parser.parse(&printed).unwrap();

        // Property: Parse → print → parse should preserve structure
        prop_assert_eq!(
            ast1.functions().len(),
            ast2.functions().len()
        );
    }
}
```

## Error Handling

### Missing Semicolon

```rust,ignore
#[test]
fn test_parse_error_missing_semicolon() {
    let c_code = "int x = 5";  // Missing semicolon

    let parser = CParser::new().unwrap();
    let result = parser.parse(c_code);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("semicolon"));
}
```

### Invalid Syntax

```rust,ignore
#[test]
fn test_parse_error_invalid_syntax() {
    let c_code = "int main() { return; }";  // return without value

    let parser = CParser::new().unwrap();
    let result = parser.parse(c_code);

    // tree-sitter may parse this, but semantic analysis should catch it
    if let Ok(ast) = result {
        let func = &ast.functions()[0];
        assert!(matches!(func.return_type(), Type::Int));

        // Return statement should have no value
        if let Statement::Return(value) = &func.body().statements()[0] {
            assert!(value.is_none());
        }
    }
}
```

## Performance Testing

### Benchmark: Small Functions

```rust,ignore
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse_small_function(c: &mut Criterion) {
    let c_code = "int add(int a, int b) { return a + b; }";
    let parser = CParser::new().unwrap();

    c.bench_function("parse_small_function", |b| {
        b.iter(|| {
            parser.parse(black_box(c_code)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_parse_small_function);
criterion_main!(benches);
```

### Benchmark: Large Files

```rust,ignore
fn benchmark_parse_large_file(c: &mut Criterion) {
    // 1000-line C file
    let c_code = generate_large_c_file(1000);
    let parser = CParser::new().unwrap();

    c.bench_function("parse_large_file", |b| {
        b.iter(|| {
            parser.parse(black_box(&c_code)).unwrap()
        });
    });
}
```

## Mutation Testing for Parser

### Original Code

```rust,ignore
pub fn parse_type(&mut self) -> Result<Type> {
    match self.current_token() {
        "int" => {
            self.advance();
            Ok(Type::Int)
        }
        "void" => {
            self.advance();
            Ok(Type::Void)
        }
        _ => Err(anyhow!("Expected type")),
    }
}
```

### Expected Mutants

1. Replace `"int"` with `"void"`
2. Replace `Ok(Type::Int)` with `Ok(Type::Void)`
3. Replace `"void"` with `"int"`
4. Remove `self.advance()`

### Tests to Catch Mutants

```rust,ignore
#[test]
fn test_parse_int_type() {
    let mut parser = create_parser("int");
    assert_eq!(parser.parse_type().unwrap(), Type::Int);
}

#[test]
fn test_parse_void_type() {
    let mut parser = create_parser("void");
    assert_eq!(parser.parse_type().unwrap(), Type::Void);
}

#[test]
fn test_parse_int_not_void() {
    let mut parser = create_parser("int");
    assert_ne!(parser.parse_type().unwrap(), Type::Void);
}

#[test]
fn test_parse_void_not_int() {
    let mut parser = create_parser("void");
    assert_ne!(parser.parse_type().unwrap(), Type::Int);
}

#[test]
fn test_parse_invalid_type() {
    let mut parser = create_parser("invalid");
    assert!(parser.parse_type().is_err());
}

#[test]
fn test_parse_type_advances_position() {
    let mut parser = create_parser("int x");
    parser.parse_type().unwrap();
    // Should now be at "x", not "int"
    assert_eq!(parser.current_token(), "x");
}
```

## Integration with tree-sitter

### Tree-sitter Configuration

```rust,ignore
use tree_sitter::{Parser as TSParser, Language};

extern "C" {
    fn tree_sitter_c() -> Language;
}

pub struct CParser {
    parser: TSParser,
}

impl CParser {
    pub fn new() -> Result<Self> {
        let mut parser = TSParser::new();
        let language = unsafe { tree_sitter_c() };
        parser.set_language(language)
            .context("Failed to set C language")?;

        Ok(Self { parser })
    }
}
```

### Verification Test

```rust,ignore
#[test]
fn test_tree_sitter_integration() {
    let parser = CParser::new().unwrap();
    let c_code = "int main() { return 0; }";

    let tree = parser.parser.parse(c_code, None).unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind(), "translation_unit");
    assert!(root.child_count() > 0);
}
```

## Coverage Requirements

Parser tests must achieve ≥80% coverage:

```bash
cargo llvm-cov --package decy-parser
```

Expected output:

```
decy-parser/src/lib.rs       87.3% coverage ✅
decy-parser/src/types.rs     91.2% coverage ✅
decy-parser/src/ast.rs       89.1% coverage ✅
──────────────────────────────────────────────
Overall                      89.2% coverage ✅
```

## Summary

Parser verification ensures:

✅ **Correct parsing**: C code → accurate AST
✅ **Error handling**: Invalid input → meaningful errors
✅ **Robustness**: No panics on malformed input
✅ **Performance**: Fast parsing (<1ms for simple functions)
✅ **Property compliance**: Invariants hold for all inputs
✅ **High coverage**: ≥80% test coverage
✅ **Mutation resistance**: ≥90% mutation kill rate

## Next Steps

- [HIR Verification](./hir.md) - Converting AST to High-level IR
- [Dataflow Analysis](./dataflow.md) - Analyzing variable usage
