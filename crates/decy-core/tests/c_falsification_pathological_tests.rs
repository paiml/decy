//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C151-C175: Pathological C patterns - function pointers, variadic functions,
//! complex structs, dangerous pointer patterns, and memory/lifetime pathologies.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C151-C155: Function pointer patterns
//! - C156-C160: Variadic and complex function patterns
//! - C161-C165: Complex struct patterns
//! - C166-C170: Dangerous pointer patterns
//! - C171-C175: Memory and lifetime pathologies
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C151-C155: Function Pointer Patterns
// ============================================================================

#[test]
fn c151_function_pointer_typedef_and_call() {
    let c_code = r#"
typedef int (*op_fn)(int, int);
int add(int a, int b) { return a + b; }
int apply(op_fn f, int x, int y) {
    return f(x, y);
}
int main() {
    op_fn op = add;
    return apply(op, 3, 4);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C151: Function pointer typedef and call should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C151: Output should not be empty");
    assert!(
        code.contains("fn add"),
        "C151: Should contain add function"
    );
    assert!(
        code.contains("fn apply"),
        "C151: Should contain apply function"
    );
    assert!(
        code.contains("fn main"),
        "C151: Should contain main function"
    );
}

#[test]
fn c152_array_of_function_pointers() {
    let c_code = r#"
int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
int mul(int a, int b) { return a * b; }
typedef int (*binop)(int, int);
int dispatch(int op, int a, int b) {
    binop ops[3];
    ops[0] = add;
    ops[1] = sub;
    ops[2] = mul;
    if (op >= 0 && op < 3) return ops[op](a, b);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C152: Array of function pointers should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C152: Output should not be empty");
    assert!(
        code.contains("fn dispatch"),
        "C152: Should contain dispatch function"
    );
    assert!(
        code.contains("fn add"),
        "C152: Should contain add function"
    );
    assert!(
        code.contains("fn sub"),
        "C152: Should contain sub function"
    );
    assert!(
        code.contains("fn mul"),
        "C152: Should contain mul function"
    );
}

#[test]
fn c153_function_pointer_as_parameter() {
    let c_code = r#"
int apply_twice(int (*f)(int), int x) {
    return f(f(x));
}
int double_it(int n) { return n * 2; }
int main() {
    return apply_twice(double_it, 5);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C153: Function pointer as parameter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C153: Output should not be empty");
    assert!(
        code.contains("fn apply_twice"),
        "C153: Should contain apply_twice function"
    );
    assert!(
        code.contains("fn double_it"),
        "C153: Should contain double_it function"
    );
}

#[test]
fn c154_function_returning_function_pointer() {
    let c_code = r#"
int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
typedef int (*binop)(int, int);
binop get_op(int choice) {
    if (choice == 0) return add;
    return sub;
}
int main() {
    binop op = get_op(0);
    return op(10, 3);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C154: Function returning function pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C154: Output should not be empty");
    assert!(
        code.contains("fn get_op"),
        "C154: Should contain get_op function"
    );
    assert!(
        code.contains("fn main"),
        "C154: Should contain main function"
    );
}

#[test]
fn c155_callback_registration_struct() {
    let c_code = r#"
typedef void (*callback_fn)(int);
struct EventHandler {
    callback_fn on_event;
    int id;
};
void default_handler(int code) {}
void register_handler(struct EventHandler *eh, callback_fn cb, int id) {
    eh->on_event = cb;
    eh->id = id;
}
void fire_event(struct EventHandler *eh, int code) {
    if (eh->on_event != 0) {
        eh->on_event(code);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C155: Callback registration pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C155: Output should not be empty");
    assert!(
        code.contains("fn register_handler"),
        "C155: Should contain register_handler function"
    );
    assert!(
        code.contains("fn fire_event"),
        "C155: Should contain fire_event function"
    );
    assert!(
        code.contains("EventHandler"),
        "C155: Should contain EventHandler struct"
    );
}

// ============================================================================
// C156-C160: Variadic and Complex Function Patterns
// ============================================================================

#[test]
fn c156_variadic_function_printf_style() {
    let c_code = r#"
#include <stdarg.h>
int sum_ints(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    int i;
    for (i = 0; i < count; i++) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C156: Variadic function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C156: Output should not be empty");
    assert!(
        code.contains("fn sum_ints"),
        "C156: Should contain sum_ints function"
    );
}

#[test]
fn c157_const_and_restrict_pointers() {
    let c_code = r#"
void copy_array(int * restrict dst, const int * restrict src, int n) {
    int i;
    for (i = 0; i < n; i++) {
        dst[i] = src[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C157: const and restrict pointers should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C157: Output should not be empty");
    assert!(
        code.contains("fn copy_array"),
        "C157: Should contain copy_array function"
    );
}

#[test]
fn c158_static_inline_function() {
    let c_code = r#"
static inline int max(int a, int b) {
    return a > b ? a : b;
}
int compute(int x, int y, int z) {
    return max(max(x, y), z);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C158: Static inline function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C158: Output should not be empty");
    assert!(
        code.contains("fn compute"),
        "C158: Should contain compute function"
    );
}

#[test]
fn c159_recursive_function_with_pointer_params() {
    let c_code = r#"
struct TreeNode {
    int value;
    struct TreeNode *left;
    struct TreeNode *right;
};
int count_nodes(struct TreeNode *root) {
    if (root == 0) return 0;
    return 1 + count_nodes(root->left) + count_nodes(root->right);
}
int find_max(struct TreeNode *root) {
    if (root == 0) return -2147483648;
    int left_max = find_max(root->left);
    int right_max = find_max(root->right);
    int m = root->value;
    if (left_max > m) m = left_max;
    if (right_max > m) m = right_max;
    return m;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C159: Recursive function with pointer params should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C159: Output should not be empty");
    assert!(
        code.contains("fn count_nodes"),
        "C159: Should contain count_nodes function"
    );
    assert!(
        code.contains("fn find_max"),
        "C159: Should contain find_max function"
    );
    assert!(
        code.contains("TreeNode"),
        "C159: Should contain TreeNode struct"
    );
}

#[test]
fn c160_kr_style_function_definition() {
    let c_code = r#"
int add(a, b)
    int a;
    int b;
{
    return a + b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C160: K&R style function definition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C160: Output should not be empty");
    assert!(
        code.contains("fn add"),
        "C160: Should contain add function"
    );
}

// ============================================================================
// C161-C165: Complex Struct Patterns
// ============================================================================

#[test]
fn c161_flexible_array_member() {
    let c_code = r#"
struct Buffer {
    int length;
    char data[];
};
int get_length(struct Buffer *buf) {
    return buf->length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C161: Flexible array member should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C161: Output should not be empty");
    assert!(
        code.contains("fn get_length"),
        "C161: Should contain get_length function"
    );
    assert!(
        code.contains("Buffer"),
        "C161: Should contain Buffer struct"
    );
}

#[test]
fn c162_bitfield_struct() {
    let c_code = r#"
struct Flags {
    unsigned int readable : 1;
    unsigned int writable : 1;
    unsigned int executable : 1;
    unsigned int reserved : 29;
};
int is_readable(struct Flags *f) {
    return f->readable;
}
void set_writable(struct Flags *f, int val) {
    f->writable = val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C162: Bitfield struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C162: Output should not be empty");
    assert!(
        code.contains("fn is_readable"),
        "C162: Should contain is_readable function"
    );
    assert!(
        code.contains("fn set_writable"),
        "C162: Should contain set_writable function"
    );
    assert!(
        code.contains("Flags"),
        "C162: Should contain Flags struct"
    );
}

#[test]
fn c163_nested_anonymous_struct_union() {
    let c_code = r#"
struct Packet {
    int type;
    union {
        struct {
            int x;
            int y;
        } position;
        struct {
            int code;
            int severity;
        } error;
    };
};
int get_packet_x(struct Packet *p) {
    return p->position.x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C163: Nested anonymous struct/union should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C163: Output should not be empty");
    assert!(
        code.contains("fn get_packet_x"),
        "C163: Should contain get_packet_x function"
    );
    assert!(
        code.contains("Packet"),
        "C163: Should contain Packet struct"
    );
}

#[test]
fn c164_self_referential_struct_linked_list() {
    let c_code = r#"
struct DListNode {
    int data;
    struct DListNode *prev;
    struct DListNode *next;
};
int traverse_forward(struct DListNode *head) {
    int count = 0;
    struct DListNode *cur = head;
    while (cur != 0) {
        count++;
        cur = cur->next;
    }
    return count;
}
int traverse_backward(struct DListNode *tail) {
    int count = 0;
    struct DListNode *cur = tail;
    while (cur != 0) {
        count++;
        cur = cur->prev;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C164: Self-referential struct (doubly-linked list) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C164: Output should not be empty");
    assert!(
        code.contains("fn traverse_forward"),
        "C164: Should contain traverse_forward function"
    );
    assert!(
        code.contains("fn traverse_backward"),
        "C164: Should contain traverse_backward function"
    );
    assert!(
        code.contains("DListNode"),
        "C164: Should contain DListNode struct"
    );
}

#[test]
fn c165_struct_with_function_pointer_dispatch() {
    let c_code = r#"
typedef int (*method_fn)(void *, int);
struct VTable {
    method_fn process;
    method_fn validate;
};
struct Object {
    struct VTable *vtable;
    int state;
};
int call_process(struct Object *obj, int input) {
    return obj->vtable->process(obj, input);
}
int call_validate(struct Object *obj, int input) {
    return obj->vtable->validate(obj, input);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C165: Struct with function pointer dispatch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C165: Output should not be empty");
    assert!(
        code.contains("fn call_process"),
        "C165: Should contain call_process function"
    );
    assert!(
        code.contains("fn call_validate"),
        "C165: Should contain call_validate function"
    );
    assert!(
        code.contains("VTable"),
        "C165: Should contain VTable struct"
    );
    assert!(
        code.contains("Object"),
        "C165: Should contain Object struct"
    );
}

// ============================================================================
// C166-C170: Dangerous Pointer Patterns
// ============================================================================

#[test]
fn c166_triple_pointer() {
    let c_code = r#"
int deref_triple(int ***ppp) {
    return ***ppp;
}
void set_triple(int ***ppp, int val) {
    ***ppp = val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C166: Triple pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C166: Output should not be empty");
    assert!(
        code.contains("fn deref_triple"),
        "C166: Should contain deref_triple function"
    );
    assert!(
        code.contains("fn set_triple"),
        "C166: Should contain set_triple function"
    );
}

#[test]
fn c167_pointer_to_array() {
    let c_code = r#"
int sum_row(int (*arr)[10], int row) {
    int sum = 0;
    int i;
    for (i = 0; i < 10; i++) {
        sum += arr[row][i];
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C167: Pointer to array should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C167: Output should not be empty");
    assert!(
        code.contains("fn sum_row"),
        "C167: Should contain sum_row function"
    );
}

#[test]
fn c168_void_pointer_casting_chain() {
    let c_code = r#"
void swap(void *a, void *b, int size) {
    char *ca = (char *)a;
    char *cb = (char *)b;
    int i;
    for (i = 0; i < size; i++) {
        char tmp = ca[i];
        ca[i] = cb[i];
        cb[i] = tmp;
    }
}
int swap_ints(int *x, int *y) {
    swap(x, y, sizeof(int));
    return *x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C168: Void pointer casting chain should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C168: Output should not be empty");
    assert!(
        code.contains("fn swap"),
        "C168: Should contain swap function"
    );
    assert!(
        code.contains("fn swap_ints"),
        "C168: Should contain swap_ints function"
    );
}

#[test]
fn c169_pointer_aliasing_violation() {
    let c_code = r#"
float type_pun(int i) {
    int *ip = &i;
    float *fp = (float *)ip;
    return *fp;
}
int alias_test(int *a, float *b) {
    *a = 1;
    *b = 2.0f;
    return *a;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C169: Pointer aliasing violation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C169: Output should not be empty");
    assert!(
        code.contains("fn type_pun"),
        "C169: Should contain type_pun function"
    );
    assert!(
        code.contains("fn alias_test"),
        "C169: Should contain alias_test function"
    );
}

#[test]
fn c170_dangling_pointer_creation() {
    let c_code = r#"
int *get_dangling() {
    int local = 42;
    return &local;
}
int use_dangling() {
    int *p = get_dangling();
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C170: Dangling pointer creation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C170: Output should not be empty");
    assert!(
        code.contains("fn get_dangling"),
        "C170: Should contain get_dangling function"
    );
    assert!(
        code.contains("fn use_dangling"),
        "C170: Should contain use_dangling function"
    );
}

// ============================================================================
// C171-C175: Memory and Lifetime Pathologies
// ============================================================================

#[test]
#[ignore = "FALSIFIED: setjmp.h jmp_buf type not recognized by parser"]
fn c171_setjmp_longjmp() {
    let c_code = r#"
#include <setjmp.h>
jmp_buf jump_buffer;
int safe_divide(int a, int b) {
    if (b == 0) {
        longjmp(jump_buffer, 1);
    }
    return a / b;
}
int try_divide(int a, int b) {
    if (setjmp(jump_buffer) == 0) {
        return safe_divide(a, b);
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C171: setjmp/longjmp should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C171: Output should not be empty");
}

#[test]
fn c172_signal_handler_with_global_state() {
    let c_code = r#"
#include <signal.h>
volatile int signal_received = 0;
int signal_count = 0;
void sig_handler(int sig) {
    signal_received = 1;
    signal_count++;
}
int setup_handler() {
    signal(SIGINT, sig_handler);
    return 0;
}
int check_signal() {
    if (signal_received) {
        signal_received = 0;
        return signal_count;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C172: Signal handler with global state should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C172: Output should not be empty");
    assert!(
        code.contains("fn sig_handler"),
        "C172: Should contain sig_handler function"
    );
    assert!(
        code.contains("fn setup_handler"),
        "C172: Should contain setup_handler function"
    );
    assert!(
        code.contains("fn check_signal"),
        "C172: Should contain check_signal function"
    );
}

#[test]
fn c173_thread_local_storage() {
    let c_code = r#"
__thread int tls_value = 0;
__thread int tls_initialized = 0;
int get_tls_value() {
    if (!tls_initialized) {
        tls_value = 42;
        tls_initialized = 1;
    }
    return tls_value;
}
void set_tls_value(int v) {
    tls_value = v;
    tls_initialized = 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C173: Thread-local storage should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C173: Output should not be empty");
    assert!(
        code.contains("fn get_tls_value"),
        "C173: Should contain get_tls_value function"
    );
    assert!(
        code.contains("fn set_tls_value"),
        "C173: Should contain set_tls_value function"
    );
}

#[test]
fn c174_alloca_stack_allocation() {
    let c_code = r#"
#include <alloca.h>
int sum_stack_array(int n) {
    int *arr = (int *)alloca(n * sizeof(int));
    int i;
    for (i = 0; i < n; i++) {
        arr[i] = i + 1;
    }
    int sum = 0;
    for (i = 0; i < n; i++) {
        sum += arr[i];
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C174: alloca stack allocation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C174: Output should not be empty");
    assert!(
        code.contains("fn sum_stack_array"),
        "C174: Should contain sum_stack_array function"
    );
}

#[test]
fn c175_mmap_memory_mapping() {
    let c_code = r#"
#include <sys/mman.h>
#include <stdlib.h>
void *allocate_page(int size) {
    void *ptr = mmap(0, size, 3, 0x22, -1, 0);
    if (ptr == (void *)-1) return 0;
    return ptr;
}
int free_page(void *ptr, int size) {
    return munmap(ptr, size);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C175: mmap memory mapping should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C175: Output should not be empty");
    assert!(
        code.contains("fn allocate_page"),
        "C175: Should contain allocate_page function"
    );
    assert!(
        code.contains("fn free_page"),
        "C175: Should contain free_page function"
    );
}
