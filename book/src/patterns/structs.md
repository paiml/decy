# Structs and Enums

Decy transforms C structs to Rust structs with proper ownership semantics, including heap-allocated structs, linked lists, and flexible array members.

## Basic Struct Transformation

### C Input

```c
struct Point {
    int x;
    int y;
};

struct Point create_point(int x, int y) {
    struct Point p;
    p.x = x;
    p.y = y;
    return p;
}
```

### Rust Output

```rust
struct Point {
    x: i32,
    y: i32,
}

fn create_point(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[test]
fn test_create_point() {
    let p = create_point(10, 20);
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}
```

**What Changed**: Field initialization syntax, implicit return

## Pattern 1: Designated Initializers

C99 allows initializing specific struct fields by name.

### C Input

```c
struct Config {
    int width;
    int height;
    int depth;
};

int main() {
    struct Config cfg = { .width = 800, .height = 600 };
    return cfg.width;
}
```

### Rust Output

```rust
#[derive(Default)]
struct Config {
    width: i32,
    height: i32,
    depth: i32,
}

fn main() -> i32 {
    let cfg = Config {
        width: 800,
        height: 600,
        ..Default::default()
    };
    cfg.width
}

#[test]
fn test_designated_init() {
    let cfg = Config {
        width: 800,
        height: 600,
        ..Default::default()
    };
    assert_eq!(cfg.width, 800);
    assert_eq!(cfg.height, 600);
    assert_eq!(cfg.depth, 0); // Default value
}
```

**What Changed**:
- `#[derive(Default)]` added for partial initialization
- `..Default::default()` fills unspecified fields

## Pattern 2: Heap-Allocated Structs

When C allocates a struct on the heap with `malloc`, Decy transforms it to `Box<T>`.

### C Input

```c
struct Node {
    int value;
    struct Node *next;
};

struct Node* create_node(int value) {
    struct Node *node = malloc(sizeof(struct Node));
    node->value = value;
    node->next = NULL;
    return node;
}
```

### Rust Output

```rust
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

fn create_node(value: i32) -> Box<Node> {
    Box::new(Node {
        value,
        next: None,
    })
}

#[test]
fn test_create_node() {
    let node = create_node(42);
    assert_eq!(node.value, 42);
    assert!(node.next.is_none());
}
```

**What Changed**:
- `malloc(sizeof(struct Node))` → `Box::new(Node { ... })`
- `struct Node *next` → `Option<Box<Node>>` (nullable pointer)
- `NULL` → `None`

## Pattern 3: Linked List Traversal

Decy handles linked list traversal patterns, converting pointer-based iteration to idiomatic Rust.

### C Input

```c
int sum_list(struct Node *head) {
    int sum = 0;
    struct Node *current = head;
    while (current != NULL) {
        sum += current->value;
        current = current->next;
    }
    return sum;
}
```

### Rust Output

```rust
fn sum_list(head: Option<&Node>) -> i32 {
    let mut sum = 0;
    let mut current = head;
    while let Some(node) = current {
        sum += node.value;
        current = node.next.as_deref();
    }
    sum
}

#[test]
fn test_sum_list() {
    let node2 = Box::new(Node { value: 30, next: None });
    let node1 = Box::new(Node { value: 20, next: Some(node2) });
    let head = Node { value: 10, next: Some(node1) };

    assert_eq!(sum_list(Some(&head)), 60);
    assert_eq!(sum_list(None), 0);
}
```

**What Changed**:
- `while (current != NULL)` → `while let Some(node) = current`
- `current = current->next` → `current = node.next.as_deref()`
- Pattern matching for safe null handling

## Pattern 4: Flexible Array Members

C99 flexible array members (FAM) at the end of structs are transformed to `Vec<T>`.

### C Input

```c
struct Message {
    int length;
    char data[];  // Flexible array member
};

struct Message* create_message(const char *text, int len) {
    struct Message *msg = malloc(sizeof(struct Message) + len);
    msg->length = len;
    // copy text to data...
    return msg;
}
```

### Rust Output

```rust
struct Message {
    length: i32,
    data: Vec<u8>,
}

fn create_message(text: &[u8], len: i32) -> Box<Message> {
    Box::new(Message {
        length: len,
        data: text.to_vec(),
    })
}

#[test]
fn test_flexible_array() {
    let msg = create_message(b"hello", 5);
    assert_eq!(msg.length, 5);
    assert_eq!(msg.data.len(), 5);
}
```

**What Changed**:
- `char data[]` → `Vec<u8>` (dynamic array)
- Manual size calculation eliminated
- Memory safety guaranteed

## Pattern 5: Nested Structs

### C Input

```c
struct Inner {
    int value;
};

struct Outer {
    struct Inner inner;
    int count;
};

int get_inner_value(struct Outer *outer) {
    return outer->inner.value;
}
```

### Rust Output

```rust
struct Inner {
    value: i32,
}

struct Outer {
    inner: Inner,
    count: i32,
}

fn get_inner_value(outer: &Outer) -> i32 {
    outer.inner.value
}

#[test]
fn test_nested() {
    let outer = Outer {
        inner: Inner { value: 42 },
        count: 1,
    };
    assert_eq!(get_inner_value(&outer), 42);
}
```

## How Decy Decides

### Struct Field Types

```
Is field a pointer to same struct type?
├─ Yes → Option<Box<Self>> (self-referential)
└─ No → Is it a pointer?
   ├─ Yes → &T, &mut T, or Box<T>
   └─ No → T (value type)
```

### Initialization Patterns

```
Does initializer have designated fields?
├─ Yes → Use struct update syntax with Default
└─ No → Use positional or named initialization
```

## Summary

| C Pattern | Rust Type | Use Case |
|-----------|-----------|----------|
| `struct T` | `T` | Value type |
| `struct T *` (heap) | `Box<T>` | Owned heap allocation |
| `struct T *` (nullable) | `Option<Box<T>>` | Nullable owned pointer |
| `struct T *next` (self-ref) | `Option<Box<T>>` | Linked structures |
| `char data[]` (FAM) | `Vec<T>` | Flexible array member |
| `{ .field = val }` | `T { field: val, ..Default::default() }` | Partial initialization |

## Next

- [Memory Management](./memory.md) - Deep dive into ownership
- [Pointers to References](./pointers.md) - Pointer transformations

---

**Note**: All code examples in this chapter are tested in CI. If they don't compile, our release is blocked!
