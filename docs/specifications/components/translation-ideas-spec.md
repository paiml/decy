# Decy: Advanced Translation Techniques from Research

**Project**: Decy - Production-Grade C-to-Rust Transpiler
**Version**: 1.0
**Date**: 2025-11-06
**Based On**: CACM "Automatically Translating C to Rust" (Research Survey)
**Methodology**: EXTREME TDD + Toyota Way + PMAT Quality Gates

---

## Executive Summary

This specification extracts proven techniques from academic research on C-to-Rust translation and adapts them to decy's architecture. The key insight from the research community is that **multi-pass static analysis** combined with **progressive refinement** produces safer, more idiomatic Rust than single-pass translation.

**Core Research Finding**: C2Rust-style tools produce syntactically valid but unsafe Rust. The solution is applying specialized static analyses to progressively transform unsafe constructs into type-safe Rust idioms.

**Decy's Advantage**: Our 6-stage pipeline architecture is perfectly positioned to implement these techniques, with dedicated crates for each analysis pass.

---

## 1. Research-Backed Translation Techniques

### 1.1 Scalar Pointer Replacement (CRITICAL FOR UNSAFE MINIMIZATION)

**Research Source**: Zhang et al. (ownership identification), Emre et al. (lifetime computation)

**Problem**: Raw C pointers (`*T`) must become one of:
- `Box<T>` (owning pointer)
- `&T` (immutable borrow)
- `&mut T` (mutable borrow)
- `Vec<T>` (array/heap allocation)

**Two-Phase Analysis**:

#### Phase 1: Ownership Classification
Determine pointer semantics from usage patterns:

```c
// Pattern 1: Owning pointer (malloc/free)
int* p = malloc(sizeof(int));
*p = 42;
free(p);
// → Box<i32>

// Pattern 2: Immutable borrow (read-only)
void print_value(const int* p) {
    printf("%d\n", *p);
}
// → fn print_value(p: &i32)

// Pattern 3: Mutable borrow (in-place mutation)
void increment(int* p) {
    (*p)++;
}
// → fn increment(p: &mut i32)

// Pattern 4: Array allocation
int* arr = malloc(10 * sizeof(int));
for (int i = 0; i < 10; i++) arr[i] = i;
free(arr);
// → Vec<i32>
```

**Decy Implementation** (in `decy-ownership` crate):

```rust
pub enum PointerOwnership {
    /// malloc/free or single owner
    Owning,
    /// Read-only access, multiple readers
    ImmutableBorrow,
    /// In-place mutation, exclusive access
    MutableBorrow,
    /// Array with dynamic size
    HeapArray,
    /// Unknown (fallback to raw pointer with unsafe)
    Unknown,
}

pub struct OwnershipAnalysis {
    /// Pointer → ownership classification
    classifications: HashMap<PointerId, PointerOwnership>,
    /// Dataflow graph for tracking pointer usage
    dataflow: PointerDataflowGraph,
}

impl OwnershipAnalysis {
    /// Run ownership inference on HIR
    pub fn infer_ownership(&mut self, hir: &Hir) -> Result<OwnershipMap> {
        // 1. Build dataflow graph
        self.build_dataflow_graph(hir)?;

        // 2. Detect allocation/deallocation patterns
        self.detect_alloc_patterns()?;

        // 3. Analyze pointer usage (reads vs writes)
        self.analyze_usage_patterns()?;

        // 4. Classify each pointer
        self.classify_pointers()?;

        // 5. Validate borrowing rules
        self.validate_borrow_checker_rules()?;

        Ok(self.classifications.clone())
    }
}
```

**Testing Strategy** (EXTREME TDD):

```rust
// RED Phase: Write failing tests
#[test]
fn test_malloc_free_becomes_box() {
    let c_code = r#"
        int* p = malloc(sizeof(int));
        *p = 42;
        free(p);
    "#;
    let ownership = OwnershipAnalysis::new().infer(parse(c_code))?;
    assert_eq!(ownership.get("p"), Some(&PointerOwnership::Owning));
}

#[test]
fn test_const_pointer_becomes_immutable_borrow() {
    let c_code = r#"
        void print(const int* p) { printf("%d", *p); }
    "#;
    let ownership = OwnershipAnalysis::new().infer(parse(c_code))?;
    assert_eq!(ownership.get("p"), Some(&PointerOwnership::ImmutableBorrow));
}
```

**Property Tests**:

```rust
proptest! {
    #[test]
    fn prop_unique_owner_per_allocation(c_program in c_program_with_pointers()) {
        let ownership = OwnershipAnalysis::new().infer(&c_program)?;

        // Property: Each allocation has exactly one owning pointer
        for alloc in c_program.allocations() {
            let owners = ownership.find_owners(alloc);
            assert_eq!(owners.len(), 1, "Each allocation must have unique owner");
        }
    }

    #[test]
    fn prop_borrows_dont_outlive_owner(c_program in c_program_with_pointers()) {
        let ownership = OwnershipAnalysis::new().infer(&c_program)?;

        // Property: Borrows never outlive their owner
        for borrow in ownership.borrows() {
            let owner = ownership.find_owner(borrow);
            assert!(borrow.lifetime <= owner.lifetime);
        }
    }
}
```

#### Phase 2: Lifetime Computation

**Problem**: References need explicit lifetimes when scope isn't obvious.

```rust
// Simple case (lifetime elision applies)
fn get_first(arr: &[i32]) -> &i32 { &arr[0] }

// Complex case (explicit lifetimes needed)
fn choose<'a, 'b>(flag: bool, a: &'a i32, b: &'b i32) -> &'a i32 {
    if flag { a } else { b } // ERROR: mismatched lifetimes
}
```

**Decy Implementation** (in `decy-ownership` crate):

```rust
pub struct LifetimeAnalysis {
    /// Variable → lifetime scope
    lifetimes: HashMap<VarId, Lifetime>,
    /// Function → lifetime parameters needed
    fn_lifetimes: HashMap<FnId, Vec<LifetimeParam>>,
}

impl LifetimeAnalysis {
    /// Compute lifetimes from C variable scopes
    pub fn compute_lifetimes(&mut self, hir: &Hir, ownership: &OwnershipMap) -> Result<LifetimeMap> {
        // 1. Map C scopes to Rust lifetimes
        self.map_scopes_to_lifetimes(hir)?;

        // 2. Analyze borrow duration
        for (ptr, ownership_kind) in ownership.iter() {
            if matches!(ownership_kind, PointerOwnership::ImmutableBorrow | PointerOwnership::MutableBorrow) {
                self.compute_borrow_duration(ptr)?;
            }
        }

        // 3. Determine where explicit lifetime annotations needed
        self.identify_explicit_lifetime_sites()?;

        // 4. Generate lifetime parameter names ('a, 'b, etc.)
        self.generate_lifetime_params()?;

        Ok(self.lifetimes.clone())
    }
}
```

**Unsafe Reduction Metric**:
- Before: 100% of pointers become `*const T` / `*mut T` (unsafe)
- After Phase 1: ~50% reduction (Box, Vec)
- After Phase 2: ~20% unsafe remaining

---

### 1.2 Lock API Translation

**Research Source**: Static analysis for lock/data binding

**Problem**: C uses manual lock/unlock with separate data:

```c
pthread_mutex_t lock;
int shared_data;

void modify() {
    pthread_mutex_lock(&lock);
    shared_data++;  // Protected by lock
    pthread_mutex_unlock(&lock);
}
```

**Rust Idiom**: Merge lock and data, use guard for RAII:

```rust
use std::sync::Mutex;

struct State {
    shared_data: Mutex<i32>,
}

impl State {
    fn modify(&self) {
        let mut data = self.shared_data.lock().unwrap();
        *data += 1;
    } // Guard dropped here, lock automatically released
}
```

**Decy Implementation** (in `decy-analyzer` crate):

```rust
pub struct LockAnalysis {
    /// Map locks to data they protect
    lock_to_data: HashMap<LockId, HashSet<VarId>>,
    /// Map variables to protecting locks
    data_to_lock: HashMap<VarId, LockId>,
}

impl LockAnalysis {
    /// Analyze which locks protect which data
    pub fn analyze_locks(&mut self, hir: &Hir) -> Result<LockMapping> {
        // 1. Find all lock/unlock pairs
        let lock_regions = self.find_lock_regions(hir)?;

        // 2. Analyze data accessed within locked regions
        for region in lock_regions {
            let accessed_vars = self.find_accessed_vars(&region)?;
            self.lock_to_data.entry(region.lock)
                .or_default()
                .extend(accessed_vars);
        }

        // 3. Verify lock discipline (all accesses protected)
        self.verify_lock_discipline()?;

        Ok(LockMapping {
            lock_to_data: self.lock_to_data.clone(),
            data_to_lock: self.data_to_lock.clone(),
        })
    }
}
```

**Code Generation Pattern**:

```rust
// In decy-codegen
impl CodeGen {
    fn generate_mutex_wrapper(&self, lock_map: &LockMapping) -> TokenStream {
        for (lock_id, protected_vars) in lock_map.lock_to_data.iter() {
            quote! {
                struct State {
                    // Merge lock and data
                    #(#protected_vars: Mutex<#types>),*
                }
            }
        }
    }
}
```

**Testing Strategy**:

```rust
#[test]
fn test_pthread_mutex_becomes_rust_mutex() {
    let c_code = r#"
        pthread_mutex_t lock;
        int data;

        void increment() {
            pthread_mutex_lock(&lock);
            data++;
            pthread_mutex_unlock(&lock);
        }
    "#;

    let rust_code = transpile(c_code)?;
    assert!(rust_code.contains("Mutex<i32>"));
    assert!(rust_code.contains(".lock().unwrap()"));
    assert!(!rust_code.contains("pthread_mutex_lock"));
}
```

---

### 1.3 Tagged Union Conversion

**Research Source**: Static analysis for tag/union binding

**Problem**: C uses separate tag + union (unsafe):

```c
enum TypeTag { INT, FLOAT, STRING };

struct Value {
    enum TypeTag tag;
    union {
        int as_int;
        float as_float;
        char* as_string;
    } data;
};

int get_int(struct Value* v) {
    if (v->tag == INT) {
        return v->data.as_int;  // Unsafe: no compiler verification
    }
    return 0;
}
```

**Rust Idiom**: Type-safe enums:

```rust
enum Value {
    Int(i32),
    Float(f64),
    String(String),
}

fn get_int(v: &Value) -> Option<i32> {
    match v {
        Value::Int(i) => Some(*i),
        _ => None,
    }
}
```

**Decy Implementation** (in `decy-analyzer` crate):

```rust
pub struct TaggedUnionAnalysis {
    /// Discovered tag-union pairs
    tagged_unions: Vec<TaggedUnion>,
}

#[derive(Debug)]
pub struct TaggedUnion {
    /// The enum used as tag
    tag: EnumId,
    /// The union type
    union: UnionId,
    /// Mapping: tag value → union field
    tag_to_field: HashMap<EnumVariant, UnionField>,
}

impl TaggedUnionAnalysis {
    pub fn discover_tagged_unions(&mut self, hir: &Hir) -> Result<Vec<TaggedUnion>> {
        // 1. Find struct with both enum and union fields
        for struct_def in hir.structs() {
            if let Some(tag_field) = struct_def.find_enum_field() {
                if let Some(union_field) = struct_def.find_union_field() {
                    // 2. Analyze usage to map tag values to union fields
                    let mapping = self.analyze_tag_usage(struct_def, tag_field, union_field)?;

                    self.tagged_unions.push(TaggedUnion {
                        tag: tag_field.enum_type,
                        union: union_field.union_type,
                        tag_to_field: mapping,
                    });
                }
            }
        }

        Ok(self.tagged_unions.clone())
    }

    /// Analyze code to find which union field is accessed for each tag value
    fn analyze_tag_usage(&self, struct_def: &StructDef, tag: EnumField, union: UnionField)
        -> Result<HashMap<EnumVariant, UnionField>> {
        let mut mapping = HashMap::new();

        // Find if (tag == VARIANT) { access union.field } patterns
        for function in struct_def.related_functions() {
            for stmt in function.statements() {
                if let Some((tag_check, field_access)) = self.match_tag_check_pattern(stmt) {
                    mapping.insert(tag_check.variant, field_access.field);
                }
            }
        }

        Ok(mapping)
    }
}
```

**Code Generation**:

```rust
impl CodeGen {
    fn generate_rust_enum(&self, tagged_union: &TaggedUnion) -> TokenStream {
        let variants: Vec<_> = tagged_union.tag_to_field.iter()
            .map(|(variant, field)| {
                let variant_name = format_ident!("{}", variant.name);
                let field_type = self.translate_type(&field.ty);
                quote! { #variant_name(#field_type) }
            })
            .collect();

        let enum_name = format_ident!("{}", tagged_union.name());
        quote! {
            enum #enum_name {
                #(#variants),*
            }
        }
    }
}
```

**Testing Strategy**:

```rust
#[test]
fn test_tagged_union_becomes_rust_enum() {
    let c_code = r#"
        enum Tag { INT, FLOAT };
        struct Value {
            enum Tag tag;
            union { int i; float f; } data;
        };

        int get_int(struct Value* v) {
            if (v->tag == INT) return v->data.i;
            return 0;
        }
    "#;

    let rust_code = transpile(c_code)?;
    assert!(rust_code.contains("enum Value"));
    assert!(rust_code.contains("Int(i32)"));
    assert!(rust_code.contains("Float(f64)"));
    assert!(rust_code.contains("match v"));
}
```

---

### 1.4 Output Parameter Elimination

**Research Source**: Static analysis for output parameter detection

**Problem**: C uses pointer parameters for output:

```c
// Return value via pointer
int parse_int(const char* str, int* result) {
    *result = atoi(str);
    return 0;  // 0 = success, -1 = error
}

// Usage
int value;
if (parse_int("123", &value) == 0) {
    printf("%d\n", value);
}
```

**Rust Idiom**: Return `Result<T>` or `Option<T>`:

```rust
fn parse_int(s: &str) -> Result<i32, ParseError> {
    s.parse()
}

// Usage
match parse_int("123") {
    Ok(value) => println!("{}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Decy Implementation** (in `decy-analyzer` crate):

```rust
pub struct OutputParamAnalysis {
    /// Functions with output parameters
    output_params: HashMap<FnId, Vec<OutputParam>>,
}

#[derive(Debug)]
pub struct OutputParam {
    /// Parameter index
    param_idx: usize,
    /// Parameter name
    name: String,
    /// Is the output partial (may fail)?
    is_fallible: bool,
}

impl OutputParamAnalysis {
    pub fn detect_output_params(&mut self, hir: &Hir) -> Result<HashMap<FnId, Vec<OutputParam>>> {
        for function in hir.functions() {
            for (idx, param) in function.params().iter().enumerate() {
                // Output parameter heuristics:
                // 1. Pointer parameter (not const)
                if !param.ty.is_pointer() || param.ty.is_const() {
                    continue;
                }

                // 2. Parameter is written to (not just read)
                if !self.is_param_written(function, param)? {
                    continue;
                }

                // 3. Not used as input (or input is discarded)
                if self.is_param_read_before_write(function, param)? {
                    continue;
                }

                // Detect if output is fallible (return value indicates success/failure)
                let is_fallible = function.return_type().is_error_code();

                self.output_params.entry(function.id)
                    .or_default()
                    .push(OutputParam {
                        param_idx: idx,
                        name: param.name.clone(),
                        is_fallible,
                    });
            }
        }

        Ok(self.output_params.clone())
    }
}
```

**Code Generation**:

```rust
impl CodeGen {
    fn transform_output_params(&self, function: &Function, output_params: &[OutputParam]) -> TokenStream {
        // Determine return type
        let return_type = if output_params.len() == 1 {
            let param = &output_params[0];
            let ty = self.translate_type(&function.params()[param.param_idx].ty.pointee());

            if param.is_fallible {
                quote! { Result<#ty, Error> }
            } else {
                quote! { #ty }
            }
        } else if output_params.len() > 1 {
            // Multiple outputs → return tuple
            let types: Vec<_> = output_params.iter()
                .map(|p| self.translate_type(&function.params()[p.param_idx].ty.pointee()))
                .collect();

            let is_fallible = output_params.iter().any(|p| p.is_fallible);
            if is_fallible {
                quote! { Result<(#(#types),*), Error> }
            } else {
                quote! { (#(#types),*) }
            }
        } else {
            quote! { () }
        };

        // Remove output params from parameter list
        let filtered_params = function.params().iter()
            .enumerate()
            .filter(|(idx, _)| !output_params.iter().any(|p| p.param_idx == *idx))
            .map(|(_, param)| self.translate_param(param));

        let fn_name = format_ident!("{}", function.name);
        quote! {
            fn #fn_name(#(#filtered_params),*) -> #return_type {
                // ... function body with return statement instead of output param writes
            }
        }
    }
}
```

**Testing Strategy**:

```rust
#[test]
fn test_output_param_becomes_return_value() {
    let c_code = r#"
        int parse(const char* str, int* result) {
            *result = atoi(str);
            return 0;
        }
    "#;

    let rust_code = transpile(c_code)?;
    assert!(rust_code.contains("fn parse(str: &str) -> Result<i32"));
    assert!(!rust_code.contains("result: &mut i32"));
}

#[test]
fn test_multiple_output_params_become_tuple() {
    let c_code = r#"
        void get_dimensions(int* width, int* height) {
            *width = 1920;
            *height = 1080;
        }
    "#;

    let rust_code = transpile(c_code)?;
    assert!(rust_code.contains("fn get_dimensions() -> (i32, i32)"));
}
```

---

## 2. Future Unsafe Features (Roadmap Items)

The research identifies four categories of unsafe features that decy should address in future sprints:

### 2.1 Array Pointers → `Vec`, `String`, Slices

**Problem**: C arrays have multiple semantic meanings:

```c
// Case 1: Fixed-size array → [T; N]
int arr[10];

// Case 2: Heap-allocated array → Vec<T>
int* arr = malloc(10 * sizeof(int));

// Case 3: String literal → &str or String
char* str = "hello";

// Case 4: Array slice (pointer + length) → &[T]
void process(int* arr, size_t len);
```

**Decy Strategy**:

1. **Fixed-size arrays**: Detect `int arr[N]` → `[i32; N]`
2. **Heap arrays**: Detect `malloc(n * sizeof(T))` → `Vec::with_capacity(n)`
3. **String handling**: Detect `char*` + string functions → `String` or `&str`
4. **Array parameters**: Detect `(ptr, len)` pairs → `&[T]` or `&mut [T]`

**Roadmap Ticket** (create in Sprint 21+):
```yaml
DECY-071:
  title: "Implement array parameter detection heuristics"
  sprint: 21
  type: feature
  priority: high
  description: |
    Detect when C pointer parameters are actually array slices.
    Heuristics:
    - Function takes (T* arr, size_t len) → &[T]
    - Loop iterates 0..len accessing arr[i] → &[T]
    - Pointer arithmetic within bounds → &[T]
```

### 2.2 File Operations → `File`, `Read`, `Write` Traits

**Problem**: C uses `FILE*` with manual error checking:

```c
FILE* f = fopen("data.txt", "r");
if (f == NULL) { /* error */ }
char buffer[256];
fgets(buffer, sizeof(buffer), f);
fclose(f);
```

**Rust Idiom**: RAII + traits:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn read_file() -> Result<(), std::io::Error> {
    let f = File::open("data.txt")?;
    let reader = BufReader::new(f);
    for line in reader.lines() {
        println!("{}", line?);
    }
    Ok(())
} // File automatically closed
```

**Roadmap Ticket**:
```yaml
DECY-080:
  title: "Translate FILE* to Rust File + RAII"
  sprint: 22
  type: feature
  priority: medium
```

### 2.3 Subprocess Handling → `Command` API

**Problem**: C uses `fork/exec/wait`:

```c
pid_t pid = fork();
if (pid == 0) {
    execl("/bin/ls", "ls", "-l", NULL);
    exit(1);
}
waitpid(pid, &status, 0);
```

**Rust Idiom**: `Command` builder:

```rust
use std::process::Command;

let output = Command::new("ls")
    .arg("-l")
    .output()?;
println!("{}", String::from_utf8_lossy(&output.stdout));
```

**Roadmap Ticket**:
```yaml
DECY-085:
  title: "Translate fork/exec to Command API"
  sprint: 23
  type: feature
  priority: low
```

### 2.4 Void Pointer Arguments → Generics

**Problem**: C uses `void*` for generic code:

```c
void swap(void* a, void* b, size_t size) {
    char temp[size];
    memcpy(temp, a, size);
    memcpy(a, b, size);
    memcpy(b, temp, size);
}
```

**Rust Idiom**: Generics with trait bounds:

```rust
fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}
```

**Roadmap Ticket**:
```yaml
DECY-090:
  title: "Translate void* to generic functions"
  sprint: 24
  type: feature
  priority: medium
```

---

## 3. LLM Integration (Future Research Direction)

The research identifies **LLM + static analysis** as a promising hybrid approach.

### 3.1 Current LLM Limitations

Research findings:
- LLMs produce unsafe code with frequent type errors
- 27-44% of translated functions fail to compile
- LLMs struggle with complex lifetime annotations

### 3.2 Hybrid Approach: Analysis-Guided LLM

**Workflow**:
1. Run static analysis first (ownership, lifetimes, lock bindings)
2. Pass analysis results as context to LLM
3. LLM generates idiomatic Rust guided by analysis
4. Verify correctness through compilation + testing
5. Iterate on errors

**Decy Integration** (Sprint 30+):

```rust
pub struct LlmCodeGen {
    /// Static analysis results
    ownership: OwnershipMap,
    lifetimes: LifetimeMap,
    locks: LockMapping,

    /// LLM client
    llm: LlmClient,
}

impl LlmCodeGen {
    pub async fn generate(&self, c_function: &CFunction) -> Result<RustCode> {
        // Build prompt with analysis results
        let prompt = format!(
            r#"
            Translate this C function to idiomatic Rust:

            ```c
            {}
            ```

            Static analysis results:
            - Ownership: {:?}
            - Lifetimes: {:?}
            - Lock bindings: {:?}

            Generate safe, idiomatic Rust code.
            "#,
            c_function.source,
            self.ownership.get(c_function.id),
            self.lifetimes.get(c_function.id),
            self.locks.get(c_function.id),
        );

        // Query LLM
        let rust_code = self.llm.complete(&prompt).await?;

        // Verify compilation
        if !rust_code.compiles()? {
            // Retry with compilation errors as feedback
            return self.generate_with_errors(c_function, &rust_code).await;
        }

        Ok(rust_code)
    }
}
```

**Note**: This is a research direction, not immediate implementation. Decy focuses on deterministic static analysis first (Sprints 1-25), with LLM integration in Sprint 30+.

---

## 4. Evaluation Metrics (Inspired by Research)

The research distinguishes multiple correctness dimensions:

### 4.1 Type-Checking Success

**Metric**: % of translated functions that compile

**Target**: 100% (decy enforces this through book-based verification)

**Measurement**:
```bash
# Compile all generated Rust
cargo build --manifest-path=book/Cargo.toml

# Track compilation success rate
decy verify --metric=compilation-rate
```

### 4.2 Behavioral Equivalence

**Metric**: % of translated functions passing behavioral tests

**Target**: 100% (tests ported from C)

**Measurement**:
```bash
# Run test suite comparing C vs Rust behavior
decy verify --compare-behavior

# Each test:
# 1. Run C version with input
# 2. Run Rust version with same input
# 3. Assert outputs match
```

### 4.3 Safety Guarantees

**Metric**: % of memory operations checked at compile time (not runtime)

**Target**: 95% (5% unsafe blocks per 1000 LOC)

**Measurement**:
```bash
# Count unsafe blocks
decy analyze --metric=unsafe-ratio

# Report format:
# Total LOC: 10000
# Unsafe blocks: 45
# Unsafe ratio: 0.45% ✅
```

### 4.4 Code Idiomaticity

**Metric**: Clippy warnings + manual review score

**Target**: 0 Clippy warnings, A grade on idiomaticity

**Measurement**:
```bash
# Clippy (automated)
cargo clippy --manifest-path=book/Cargo.toml -- -D warnings

# Idiomaticity review (manual)
decy review --metric=idiomaticity
# Checks:
# - Uses iterator methods (not manual loops)
# - Uses Result/Option (not sentinel values)
# - Uses match (not if/else chains)
# - Uses &str (not &String)
```

---

## 5. Integration with Decy Architecture

### 5.1 Crate Responsibilities

| Technique | Primary Crate | Supporting Crates |
|-----------|---------------|-------------------|
| Ownership Classification | `decy-ownership` | `decy-analyzer` (dataflow) |
| Lifetime Computation | `decy-ownership` | `decy-hir` (scopes) |
| Lock API Translation | `decy-analyzer` | `decy-codegen` (Mutex wrappers) |
| Tagged Union Conversion | `decy-analyzer` | `decy-codegen` (enum generation) |
| Output Param Elimination | `decy-analyzer` | `decy-codegen` (signature transform) |
| Array Detection | `decy-analyzer` | `decy-ownership` (Vec vs slice) |
| File API Translation | `decy-codegen` | N/A |
| Subprocess Translation | `decy-codegen` | N/A |
| Void Pointer → Generics | `decy-hir` | `decy-codegen` |

### 5.2 Pipeline Flow

```
C Source
    ↓
[decy-parser] Parse C AST
    ↓
[decy-hir] Build HIR
    ↓
[decy-analyzer] Static analyses:
    ├─ Lock analysis
    ├─ Tagged union detection
    ├─ Output param detection
    └─ Array pattern detection
    ↓
[decy-ownership] Ownership & lifetime inference
    ↓
[decy-verify] Validate safety properties
    ↓
[decy-codegen] Generate Rust:
    ├─ Apply ownership/lifetime annotations
    ├─ Transform lock APIs
    ├─ Convert tagged unions to enums
    ├─ Eliminate output params
    └─ Replace arrays with Vec/slices
    ↓
Rust Output (safe & idiomatic)
```

---

## 6. Roadmap Integration

### 6.1 Immediate Priorities (Sprint 21-25)

Based on research impact, prioritize:

1. **Sprint 21**: Ownership classification (scalar pointers)
   - `DECY-071`: Array parameter detection
   - `DECY-072`: Ownership inference algorithm

2. **Sprint 22**: Lifetime computation
   - `DECY-075`: Lifetime scope mapping
   - `DECY-076`: Explicit lifetime annotation sites

3. **Sprint 23**: Lock API translation
   - `DECY-080`: pthread_mutex → Mutex<T>
   - `DECY-081`: Lock discipline verification

4. **Sprint 24**: Tagged union conversion
   - `DECY-085`: Tag-union detection
   - `DECY-086`: Enum generation

5. **Sprint 25**: Output parameter elimination
   - `DECY-090`: Output param detection
   - `DECY-091`: Result/Option return generation

### 6.2 Future Work (Sprint 26-30)

1. **Sprint 26**: Array handling
2. **Sprint 27**: File API translation
3. **Sprint 28**: Subprocess handling
4. **Sprint 29**: Void pointer → generics
5. **Sprint 30**: LLM integration (research)

### 6.3 Quality Gates for Each Sprint

Every technique must meet:

1. **Coverage**: ≥85% (90% for `decy-ownership`)
2. **Property tests**: ≥3 per analysis function
3. **Mutation score**: ≥90%
4. **Unsafe reduction**: Measured against baseline
5. **Documentation**: Complete API docs + examples

---

## 7. EXTREME TDD Workflow (Per Technique)

### 7.1 RED Phase: Failing Tests

For each technique (e.g., lock API translation):

```rust
// tests/test_lock_translation.rs

// RED: These tests should FAIL initially
#[test]
fn test_pthread_mutex_becomes_mutex() {
    let c_code = r#"
        pthread_mutex_t lock;
        int data;
        void inc() {
            pthread_mutex_lock(&lock);
            data++;
            pthread_mutex_unlock(&lock);
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Expected transformations
    assert!(rust_code.contains("Mutex<i32>"));
    assert!(rust_code.contains(".lock().unwrap()"));
    assert!(!rust_code.contains("pthread"));
}

#[test]
fn test_multiple_locks_separate_mutexes() { /* ... */ }

#[test]
fn test_nested_locks_compiles() { /* ... */ }
```

**Commit**:
```bash
git commit --no-verify -m "[RED] DECY-080: Add lock translation tests (10 failing)"
```

### 7.2 GREEN Phase: Minimal Implementation

Implement just enough to pass tests:

```rust
// crates/decy-analyzer/src/lock_analysis.rs

impl LockAnalysis {
    pub fn analyze_locks(&mut self, hir: &Hir) -> Result<LockMapping> {
        // Minimal implementation to pass tests
        todo!("Implement lock analysis")
    }
}
```

**Commit**:
```bash
cargo test  # Tests pass
git commit -m "[GREEN] DECY-080: Minimal lock analysis implementation"
```

### 7.3 REFACTOR Phase: Quality Gates

Refactor to meet quality standards:

```bash
# Add docs
cargo doc --open

# Check coverage (must be ≥85%)
cargo llvm-cov --lcov --output-path=lcov.info

# Add property tests
# Add mutation tests

# Verify quality gates
make quality-gates  # Must pass

git commit -m "[REFACTOR] DECY-080: Lock analysis meets quality gates"
```

### 7.4 Final: Squash and Close

```bash
git rebase -i HEAD~3  # Squash RED/GREEN/REFACTOR

git commit -m "DECY-080: Translate pthread_mutex to Rust Mutex

- Lock-to-data binding analysis
- Mutex<T> wrapper generation
- RAII guard usage
- Coverage: 87% ✅
- Mutation score: 91% ✅
- Unsafe reduction: 15% → 12% ✅

Closes #80"
```

---

## 8. Success Criteria

Decy successfully implements research-backed techniques when:

1. **Ownership inference**: 90%+ of pointers classified correctly
2. **Lifetime computation**: 95%+ of references use elided lifetimes (no manual annotations)
3. **Lock translation**: 100% of pthread locks become Rust Mutex
4. **Tagged unions**: 100% converted to type-safe enums
5. **Output params**: 90%+ eliminated in favor of Result/Option
6. **Unsafe ratio**: <5 unsafe blocks per 1000 LOC (target from spec-v1)
7. **Compilation rate**: 100% of generated Rust compiles
8. **Behavioral equivalence**: 100% of tests pass
9. **Idiomaticity**: 0 Clippy warnings

---

## 9. References

1. **CACM Research Article**: "Automatically Translating C to Rust" (2024)
   - URL: https://cacm.acm.org/research/automatically-translating-c-to-rust/
   - Key insight: Multi-pass static analysis progressively refines unsafe code

2. **C2Rust Project**: Baseline C-to-Rust transpiler
   - Produces syntactically valid but unsafe Rust
   - Decy aims to surpass with advanced analyses

3. **Zhang et al.**: Ownership identification for pointers
   - Pattern detection for malloc/free, borrows, arrays

4. **Emre et al.**: Lifetime computation from variable scopes
   - Maps C scopes to Rust lifetime parameters

5. **Lock API Research**: Static analysis for lock-data binding
   - Merges separate locks and data into Mutex<T>

6. **Tagged Union Research**: Tag-union pair detection
   - Converts to type-safe Rust enums

7. **LLM Hybrid Approaches**: Analysis-guided generation
   - Future direction for idiomatic code generation

---

## 10. Conclusion

The research community has validated that **multi-pass static analysis** is the correct approach for C-to-Rust translation. Decy's 6-stage pipeline architecture perfectly aligns with this approach, with dedicated crates for each analysis type.

**Key Takeaways**:

1. **Ownership inference is critical** for unsafe minimization (50% reduction)
2. **Lifetime computation** further reduces unsafe (another 30% reduction)
3. **Specialized analyses** (locks, tagged unions, output params) improve idiomaticity
4. **LLM integration** is promising but not yet reliable alone
5. **Evaluation must cover 4 dimensions**: compilation, behavior, safety, idiomaticity

**Next Steps**:

1. Add roadmap tickets for each technique (DECY-071 through DECY-095)
2. Prioritize ownership/lifetime inference (Sprint 21-22)
3. Follow EXTREME TDD for each implementation
4. Track unsafe ratio reduction per sprint
5. Validate against C99/K&R test suite (see `C-VALIDATION-ROADMAP.yaml`)

---

**Status**: DRAFT (awaiting roadmap.yaml ticket creation)
**Last Updated**: 2025-11-06
**Related Docs**:
- `decy-spec-v1.md` - Main specification
- `decy-unsafe-minimization-strategy.md` - Unsafe reduction strategy
- `C-VALIDATION-ROADMAP.yaml` - Validation north star
- `../roadmap.yaml` - Ticket tracking
