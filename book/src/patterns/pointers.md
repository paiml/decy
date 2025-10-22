# Pointers to References

One of the most important transformations Decy performs is converting C pointers to safe Rust references.

## The Problem

C uses pointers for everything:
- Borrowing data (`const int *ptr`)
- Mutating data (`int *ptr`)
- Owning heap memory (`int *ptr = malloc(...)`)

Rust distinguishes these cases with different types:
- **Immutable borrow**: `&T`
- **Mutable borrow**: `&mut T`  
- **Owned heap**: `Box<T>`

## Pattern 1: Immutable Reference

### C Input

```c
int get_value(const int *ptr) {
    return *ptr;
}
```

### Rust Output

```rust
fn get_value(ptr: &i32) -> i32 {
    *ptr
}

#[test]
fn test_get_value() {
    let x = 42;
    assert_eq!(get_value(&x), 42);
}
```

**What Changed**: `const int *` → `&i32` (immutable reference)

## Pattern 2: Mutable Reference

### C Input

```c
void increment(int *value) {
    *value += 1;
}
```

### Rust Output

```rust
fn increment(value: &mut i32) {
    *value += 1;
}

#[test]
fn test_increment() {
    let mut x = 5;
    increment(&mut x);
    assert_eq!(x, 6);
}
```

**What Changed**: `int *` with mutation → `&mut i32` (mutable reference)

## Pattern 3: Owned Heap Memory

### C Input

```c
int* create_number() {
    int *num = malloc(sizeof(int));
    *num = 42;
    return num;
}

void use_number() {
    int *n = create_number();
    // use n
    free(n);
}
```

### Rust Output

```rust
fn create_number() -> Box<i32> {
    Box::new(42)
}

fn use_number() {
    let n = create_number();
    // use n
    // Box automatically freed here
}

#[test]
fn test_create_number() {
    let n = create_number();
    assert_eq!(*n, 42);
    // n is automatically dropped (freed) here
}
```

**What Changed**: 
- `malloc` → `Box::new()` (heap allocation with ownership)
- `free` → automatic (Drop trait)

## How Decy Decides

Decy uses **dataflow analysis** to determine the correct type:

1. **Tracks pointer assignments** through the program
2. **Checks for mutations** (writes through the pointer)  
3. **Detects malloc/free patterns** for ownership
4. **Analyzes lifetimes** to ensure safety

### Decision Tree

```
Is the pointer from malloc?
├─ Yes → Box<T>
└─ No → Is it ever written through?
   ├─ Yes → &mut T
   └─ No → &T
```

## Edge Cases

### NULL Pointers

C:
```c
int* nullable() {
    return NULL;  // Can return NULL
}
```

Rust:
```rust
fn nullable() -> Option<Box<i32>> {
    None  // Option expresses nullability
}

#[test]
fn test_nullable() {
    assert_eq!(nullable(), None);
}
```

### Pointer Arithmetic

⚠️ **Not Yet Supported**

C pointer arithmetic like `ptr++` requires `unsafe` in Rust:

```c
void iterate(int *arr, int len) {
    for (int i = 0; i < len; i++) {
        arr[i] = i;  // Uses pointer arithmetic internally
    }
}
```

Decy converts this to safe array indexing:

```rust
fn iterate(arr: &mut [i32]) {
    for i in 0..arr.len() {
        arr[i] = i as i32;
    }
}

#[test]
fn test_iterate() {
    let mut arr = vec![0; 5];
    iterate(&mut arr);
    assert_eq!(arr, vec![0, 1, 2, 3, 4]);
}
```

## Summary

| C Pattern | Rust Type | Use Case |
|-----------|-----------|----------|
| `const T *` | `&T` | Read-only borrow |
| `T *` (mutated) | `&mut T` | Mutable borrow |
| `malloc/free` | `Box<T>` | Owned heap |
| `NULL` | `Option<&T>` | Nullable reference |

## Next

- [Arrays and Slices](./arrays.md) - Converting C arrays to Rust
- [Memory Management](./memory.md) - Deep dive into ownership

---

**Note**: All code examples in this chapter are tested in CI. If they don't compile, our release is blocked!
