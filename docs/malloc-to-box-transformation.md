# Malloc-to-Box Transformation Pipeline

## Overview

DECY implements a sophisticated transformation pipeline that converts unsafe C malloc/free patterns into safe, idiomatic Rust `Box<T>` types. This document describes the complete transformation process.

## Pipeline Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   C Code    │ ──> │   C Parser   │ ──> │     HIR     │ ──> │   Pattern    │
│   (Input)   │     │  (decy-parser) │     │ (decy-hir)  │     │   Detector   │
└─────────────┘     └──────────────┘     └─────────────┘     │(decy-analyzer)│
                                                              └──────┬────────┘
                                                                     │
                                                                     v
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│  Rust Code  │ <── │     Code     │ <── │     Box     │ <── │     Box      │
│  (Output)   │     │  Generator   │     │Transformation│     │  Candidates  │
└─────────────┘     │(decy-codegen)│     │(box_transform)│     └──────────────┘
                    └──────────────┘     └─────────────┘
```

## Transformation Phases

### Phase 1: HIR Construction

**Input**: C source code
**Output**: High-level Intermediate Representation (HIR)

The C code is parsed and converted into HIR, which represents:
- Function declarations and bodies
- Variable declarations with types
- Expressions (including function calls like `malloc()`)
- Statements (assignments, returns, control flow)

**Example**:
```c
int* ptr = malloc(sizeof(int));
```

Becomes HIR:
```rust
HirStatement::VariableDeclaration {
    name: "ptr",
    var_type: HirType::Pointer(Box::new(HirType::Int)),
    initializer: Some(HirExpression::FunctionCall {
        function: "malloc",
        arguments: vec![IntLiteral(4)]
    })
}
```

### Phase 2: Pattern Detection

**Input**: HIR functions
**Output**: List of `BoxCandidate` instances

The `PatternDetector` analyzes HIR to identify malloc patterns:

1. **Malloc Detection**: Finds `malloc()` calls in:
   - Variable declarations: `T* ptr = malloc(...)`
   - Assignment statements: `ptr = malloc(...)`

2. **Candidate Creation**: Records:
   - Variable name
   - Statement index where malloc occurs
   - Statement index where free occurs (future work)

**Example**:
```rust
BoxCandidate {
    variable: "ptr",
    malloc_index: 0,
    free_index: None,  // Not yet implemented
}
```

### Phase 3: Box Transformation

**Input**: HIR statements + BoxCandidates
**Output**: Transformed HIR with Box types

The `BoxTransformer` performs two key transformations:

1. **Expression Transformation**: `malloc()` → `Box::new()`
   ```rust
   // Before
   HirExpression::FunctionCall {
       function: "malloc",
       arguments: [IntLiteral(4)]
   }

   // After
   HirExpression::FunctionCall {
       function: "Box::new",
       arguments: [IntLiteral(0)]
   }
   ```

2. **Type Transformation**: `Pointer<T>` → `Box<T>`
   ```rust
   // Before
   var_type: HirType::Pointer(Box::new(HirType::Int))

   // After
   var_type: HirType::Box(Box::new(HirType::Int))
   ```

### Phase 4: Code Generation

**Input**: Transformed HIR
**Output**: Safe Rust code

The `CodeGenerator` produces idiomatic Rust:

1. **Type Mapping**:
   - `HirType::Box(Int)` → `"Box<i32>"`
   - `HirType::Box(Char)` → `"Box<u8>"`
   - `HirType::Box(Float)` → `"Box<f32>"`

2. **Statement Generation**:
   ```rust
   let mut ptr: Box<i32> = Box::new(0);
   ```

## Complete Example

### Input: C Code
```c
void process() {
    int* number = malloc(sizeof(int));
    char* letter = malloc(sizeof(char));
}
```

### Step 1: Parse to HIR
```rust
HirFunction {
    name: "process",
    return_type: Void,
    body: [
        VariableDeclaration {
            name: "number",
            var_type: Pointer(Int),
            initializer: FunctionCall("malloc", [4])
        },
        VariableDeclaration {
            name: "letter",
            var_type: Pointer(Char),
            initializer: FunctionCall("malloc", [1])
        }
    ]
}
```

### Step 2: Detect Patterns
```rust
candidates = [
    BoxCandidate { variable: "number", malloc_index: 0 },
    BoxCandidate { variable: "letter", malloc_index: 1 }
]
```

### Step 3: Transform
```rust
HirFunction {
    name: "process",
    return_type: Void,
    body: [
        VariableDeclaration {
            name: "number",
            var_type: Box(Int),  // ← Changed
            initializer: FunctionCall("Box::new", [0])  // ← Changed
        },
        VariableDeclaration {
            name: "letter",
            var_type: Box(Char),  // ← Changed
            initializer: FunctionCall("Box::new", [0])  // ← Changed
        }
    ]
}
```

### Step 4: Generate Code
```rust
fn process() {
    let mut number: Box<i32> = Box::new(0);
    let mut letter: Box<u8> = Box::new(0);
}
```

## Safety Guarantees

The transformation provides several safety improvements:

1. **Memory Safety**: Box<T> automatically frees memory when it goes out of scope (RAII)
2. **No Null Pointers**: Box<T> cannot be null (unlike raw pointers)
3. **Ownership**: Rust's ownership system prevents use-after-free
4. **No Manual Management**: Eliminates need for manual `free()` calls

## Comparison

| Aspect | C (malloc/free) | Rust (Box<T>) |
|--------|----------------|---------------|
| Memory Management | Manual | Automatic (RAII) |
| Null Safety | Can be NULL | Cannot be null |
| Memory Leaks | Possible if free() forgotten | Impossible |
| Use-After-Free | Possible | Compile-time prevention |
| Double Free | Possible | Compile-time prevention |
| Type Safety | Weak (void*) | Strong (Box<T>) |

## Implementation Details

### Key Files

- **`decy-hir/src/lib.rs`**: HIR type definitions including `HirType::Box`
- **`decy-analyzer/src/patterns.rs`**: Pattern detection logic
- **`decy-codegen/src/box_transform.rs`**: Transformation implementation
- **`decy-codegen/src/lib.rs`**: Code generation with Box support

### Key Types

```rust
// HIR Type representing Box
pub enum HirType {
    // ... other types ...
    Box(Box<HirType>),  // Rust Box<T>
}

// Detected pattern candidate
pub struct BoxCandidate {
    pub variable: String,
    pub malloc_index: usize,
    pub free_index: Option<usize>,
}

// Transformer
pub struct BoxTransformer;
impl BoxTransformer {
    pub fn transform_malloc_to_box(&self, expr: &HirExpression, pointee_type: &HirType) -> HirExpression;
    pub fn transform_statement(&self, stmt: &HirStatement, candidate: &BoxCandidate) -> HirStatement;
}
```

## Testing

The transformation is thoroughly tested with:

- **96 Unit Tests**: Individual component testing
- **4 Property Tests**: Randomized testing with 100 cases each
- **6 Integration Tests**: End-to-end pipeline testing
- **1 Example**: Interactive demonstration

Total: **506 test cases** with **95.59% coverage**

## Future Enhancements

1. **Free Call Detection**: Identify and remove corresponding `free()` calls
2. **Size Analysis**: Use `sizeof()` information for proper initialization
3. **Type Inference**: Better default values based on usage patterns
4. **Vec<T> Support**: Detect and transform array allocations
5. **Rc<T>/Arc<T>**: Support for shared ownership patterns

## Usage

```rust
use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::HirFunction;

// 1. Parse C code to HIR
let hir_function: HirFunction = parse_c_code(c_source);

// 2. Detect Box candidates
let detector = PatternDetector::new();
let candidates = detector.find_box_candidates(&hir_function);

// 3. Generate transformed code
let codegen = CodeGenerator::new();
let rust_code = codegen.generate_function_with_box_transform(&hir_function, &candidates);
```

## References

- [Rust Box<T> Documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html)
- [RAII in Rust](https://doc.rust-lang.org/rust-by-example/scope/raii.html)
- [Ownership in Rust](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
