# C++ Transpilation

Decy supports transpiling a significant subset of C++ to idiomatic Rust. This chapter covers the supported features and how they map to Rust constructs.

## Supported Features

| C++ Feature | Rust Mapping | Status |
|-------------|-------------|--------|
| Classes | `struct` + `impl` | Complete |
| Namespaces | `pub mod` | Complete |
| Constructors | `pub fn new() -> Self` | Complete |
| Destructors | `impl Drop` | Complete |
| `new`/`delete` | `Box::new()` / `drop()` | Complete |
| Operator overloading | `std::ops` traits | Complete |
| Single inheritance | Composition + `Deref` | Complete |
| CUDA `.cu` files | C++ mode parsing | Complete |
| CUDA `__global__` kernels | `extern "C"` FFI | Complete |

## Classes

C++ classes transpile to Rust structs with `impl` blocks:

```cpp
// C++ input
class Counter {
public:
    int count;
    Counter(int initial) : count(initial) {}
    int get() { return count; }
    ~Counter() {}
};
```

```rust
// Rust output
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counter {
    pub count: i32,
}

impl Counter {
    pub fn new(initial: i32) -> Self {
        Self { count: initial }
    }

    pub fn get(&mut self) -> i32 {
        return self.count;
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        // Destructor body
    }
}
```

**Key mappings:**
- Constructor parameters are mapped to struct fields by name first, then by position
- `const` methods get `&self`, non-const get `&mut self`
- Implicit `this->member` access becomes `self.member`

## Namespaces

C++ namespaces map to Rust modules. Nested namespaces become nested modules.

```cpp
namespace math {
    int square(int x) { return x * x; }
    namespace constants {
        int pi_approx() { return 3; }
    }
}
```

```rust
pub mod math {
    fn square(mut x: i32) -> i32 {
        return x * x;
    }

    pub mod constants {
        fn pi_approx() -> i32 {
            return 3;
        }
    }
}
```

## Operator Overloading

Overloaded operators map to `std::ops` trait implementations:

| C++ Operator | Rust Trait |
|-------------|-----------|
| `operator+` | `impl std::ops::Add` |
| `operator-` | `impl std::ops::Sub` |
| `operator*` | `impl std::ops::Mul` |
| `operator/` | `impl std::ops::Div` |
| `operator%` | `impl std::ops::Rem` |
| `operator==` | `impl PartialEq` |
| `operator+=` | `impl std::ops::AddAssign` |

Regular methods stay in the `impl` block; operator methods generate separate trait implementations.

## Inheritance

Single inheritance uses Rust's composition pattern with `Deref`/`DerefMut`:

```cpp
class Shape {
public:
    int color;
    int get_color() { return color; }
};

class Circle : public Shape {
public:
    int radius;
};
```

```rust
pub struct Shape {
    pub color: i32,
}

impl Shape {
    pub fn get_color(&mut self) -> i32 {
        return self.color;
    }
}

pub struct Circle {
    pub base: Shape,  // Base class embedded as field
    pub radius: i32,
}

impl std::ops::Deref for Circle {
    type Target = Shape;
    fn deref(&self) -> &Self::Target { &self.base }
}

impl std::ops::DerefMut for Circle {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}
```

The `Deref` impl allows calling `circle.get_color()` transparently.

## CUDA Support

Decy accepts `.cu` files and handles CUDA-specific constructs:

- **`__global__` kernels** generate `extern "C"` FFI declarations (kernels run on GPU, not transpiled)
- **`__device__` functions** are annotated as GPU-only
- **`__host__` functions** transpile normally
- **Host-side C/C++ code** uses the standard transpilation pipeline

```cpp
// input.cu
__global__ void vector_add(float* a, float* b, float* c, int n) {
    // GPU kernel — not transpiled
}

void launch_kernel(float* a, float* b, float* c, int n) {
    // Host code — transpiled normally
}
```

```rust
extern "C" {
    /// CUDA kernel: vector_add (compiled separately)
    fn vector_add(a: *mut f32, b: *mut f32, c: *mut f32, n: i32);
}

fn launch_kernel(a: *mut f32, b: *mut f32, c: *mut f32, n: i32) {
    // Transpiled host code
}
```

## Running the Examples

```bash
# C++ feature demo (4 demos with assertions)
cargo run -p decy-core --example cpp_class_transpile_demo

# CUDA transpilation demo (3 demos)
cargo run -p decy-core --example cuda_transpile_demo

# Dogfood validation (5 tests compiled with rustc)
cargo run -p decy-core --example dogfood_validation_demo
```

## Limitations

- **Template metaprogramming** is not supported (requires manual rewrite)
- **Multiple inheritance** is not supported
- **Exceptions** (`try`/`catch`/`throw`) are not yet transpiled
- **Lambda expressions** are not yet supported
- **Virtual dispatch** generates `dyn Trait` (planned)
- **Operator method bodies** currently generate `Default::default()` placeholder
- **Method calls on self** — `brightness()` in a method body generates `brightness()` (free function) instead of `self.brightness()`. Workaround: use explicit `this->brightness()` in C++ source.
- **`new` type inference** — `T* p = new T(args)` correctly generates `Box::new(T::new(args))` but the variable type stays `*mut T`. Needs ownership inference to upgrade to `Box<T>`.
- **Default constructors** — `T()` without parameters doesn't generate `new()`. Use `T::default()` in Rust.
