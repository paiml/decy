# Memory Management

Decy transforms C's manual memory management to Rust's ownership system, eliminating memory leaks, double-frees, and use-after-free bugs.

## The Problem

C requires manual memory management:
- `malloc`/`free` for heap allocation
- No compiler enforcement of ownership rules
- Common bugs: leaks, double-free, use-after-free

Rust's ownership system provides:
- **Automatic deallocation** (Drop trait)
- **Compile-time ownership verification**
- **Zero memory leaks** (when not using `unsafe`)

## Pattern 1: malloc to Box

Single heap allocations become `Box<T>`.

### C Input

```c
int* create_int(int value) {
    int *ptr = malloc(sizeof(int));
    *ptr = value;
    return ptr;
}

void use_int() {
    int *num = create_int(42);
    printf("%d\n", *num);
    free(num);
}
```

### Rust Output

```rust
fn create_int(value: i32) -> Box<i32> {
    Box::new(value)
}

fn use_int() {
    let num = create_int(42);
    println!("{}", *num);
    // Box automatically freed here
}

#[test]
fn test_create_int() {
    let num = create_int(42);
    assert_eq!(*num, 42);
}
```

**What Changed**:
- `malloc(sizeof(int))` → `Box::new()`
- `free(num)` → automatic (Drop)
- No memory leak possible

## Pattern 2: Array malloc to Vec

Array allocations become `Vec<T>`.

### C Input

```c
int* create_array(int n) {
    int *arr = malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) {
        arr[i] = i;
    }
    return arr;
}

void use_array() {
    int *arr = create_array(100);
    // use arr...
    free(arr);
}
```

### Rust Output

```rust
fn create_array(n: usize) -> Vec<i32> {
    (0..n).map(|i| i as i32).collect()
}

fn use_array() {
    let arr = create_array(100);
    // use arr...
    // Vec automatically freed here
}

#[test]
fn test_create_array() {
    let arr = create_array(5);
    assert_eq!(arr, vec![0, 1, 2, 3, 4]);
}
```

**What Changed**:
- `malloc(n * sizeof(int))` → `Vec` (iterator collect)
- Size tracked automatically by Vec
- No bounds overflow possible

## Pattern 3: realloc to Vec::resize

Dynamic resizing becomes safe Vec operations.

### C Input

```c
int* grow_array(int *arr, int old_size, int new_size) {
    arr = realloc(arr, new_size * sizeof(int));
    for (int i = old_size; i < new_size; i++) {
        arr[i] = 0;
    }
    return arr;
}
```

### Rust Output

```rust
fn grow_array(arr: &mut Vec<i32>, new_size: usize) {
    arr.resize(new_size, 0);
}

#[test]
fn test_grow_array() {
    let mut arr = vec![1, 2, 3];
    grow_array(&mut arr, 6);
    assert_eq!(arr, vec![1, 2, 3, 0, 0, 0]);
}
```

**What Changed**:
- `realloc()` → `Vec::resize()`
- No null check needed (Vec cannot fail allocation in safe code)
- Old data preserved automatically

## Pattern 4: Struct Allocation

Struct heap allocation becomes `Box<T>`.

### C Input

```c
typedef struct {
    int x;
    int y;
} Point;

Point* create_point(int x, int y) {
    Point *p = malloc(sizeof(Point));
    p->x = x;
    p->y = y;
    return p;
}

void use_point() {
    Point *p = create_point(10, 20);
    printf("(%d, %d)\n", p->x, p->y);
    free(p);
}
```

### Rust Output

```rust
struct Point {
    x: i32,
    y: i32,
}

fn create_point(x: i32, y: i32) -> Box<Point> {
    Box::new(Point { x, y })
}

fn use_point() {
    let p = create_point(10, 20);
    println!("({}, {})", p.x, p.y);
    // Box<Point> automatically freed here
}

#[test]
fn test_create_point() {
    let p = create_point(10, 20);
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}
```

## Pattern 5: Ownership Transfer

C pointer passing with ownership becomes Rust move semantics.

### C Input

```c
// Takes ownership - caller should not free
void consume_data(int *data) {
    // uses data...
    free(data);  // Consumer frees
}

void producer() {
    int *data = malloc(sizeof(int));
    *data = 42;
    consume_data(data);
    // data is now invalid - don't use!
}
```

### Rust Output

```rust
fn consume_data(data: Box<i32>) {
    // uses data...
    // Box dropped here automatically
}

fn producer() {
    let data = Box::new(42);
    consume_data(data);
    // data is moved - compiler prevents use!
}

#[test]
fn test_ownership_transfer() {
    let data = Box::new(42);
    consume_data(data);
    // This would not compile:
    // println!("{}", *data);  // error: value borrowed after move
}
```

**What Changed**:
- Ownership transfer enforced by compiler
- Use-after-free impossible

## Pattern 6: Nullable Pointers to Option

NULL checks become `Option<T>`.

### C Input

```c
int* find_value(int *arr, int len, int target) {
    for (int i = 0; i < len; i++) {
        if (arr[i] == target) {
            return &arr[i];
        }
    }
    return NULL;
}

void use_find() {
    int arr[] = {1, 2, 3, 4, 5};
    int *found = find_value(arr, 5, 3);
    if (found != NULL) {
        printf("Found: %d\n", *found);
    }
}
```

### Rust Output

```rust
fn find_value(arr: &[i32], target: i32) -> Option<&i32> {
    arr.iter().find(|&&x| x == target)
}

fn use_find() {
    let arr = [1, 2, 3, 4, 5];
    if let Some(found) = find_value(&arr, 3) {
        println!("Found: {}", found);
    }
}

#[test]
fn test_find_value() {
    let arr = [1, 2, 3, 4, 5];
    assert_eq!(find_value(&arr, 3), Some(&3));
    assert_eq!(find_value(&arr, 10), None);
}
```

**What Changed**:
- `NULL` → `None`
- Pointer return → `Option<&T>`
- Null check enforced by type system

## How Decy Decides

### Allocation Type

```
malloc(sizeof(T)) where T is single item?
├─ Yes → Box<T>
└─ No → malloc(n * sizeof(T))?
   ├─ Yes → Vec<T>
   └─ No → Analyze context...
```

### Ownership vs Borrowing

```
Does function free the pointer?
├─ Yes → Takes ownership (move)
└─ No → Is pointer modified?
   ├─ Yes → &mut T (mutable borrow)
   └─ No → &T (immutable borrow)
```

## Safety Guarantees

Decy's transformations provide these guarantees:

| C Bug | Rust Prevention |
|-------|-----------------|
| Memory leak | Drop trait (automatic deallocation) |
| Double free | Move semantics (single owner) |
| Use-after-free | Borrow checker (lifetime tracking) |
| Null pointer dereference | Option type (no null) |
| Buffer overflow | Bounds checking |

## Summary

| C Pattern | Rust Type | Notes |
|-----------|-----------|-------|
| `malloc(sizeof(T))` | `Box<T>` | Single allocation |
| `malloc(n * sizeof(T))` | `Vec<T>` | Array allocation |
| `realloc(ptr, size)` | `vec.resize()` | Growth |
| `free(ptr)` | Automatic | Drop trait |
| `NULL` | `None` | Option type |
| Ownership transfer | Move semantics | Compiler enforced |

## Next

- [Pointers to References](./pointers.md) - Pointer patterns
- [Use-After-Free Safety](./use-after-free-safety.md) - Safety demonstrations

---

**Note**: All code examples in this chapter are tested in CI. If they don't compile, our release is blocked!
