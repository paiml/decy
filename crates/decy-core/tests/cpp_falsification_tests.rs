//! Popperian Falsification Test Suite: C++ Boundary Testing
//!
//! C201-C225: Systematic falsification of C++ constructs in C/C++ mixed codebases.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! The transpiler uses clang for parsing with filename "input.c", so clang
//! treats all input as C. C++ constructs that are valid C99 should transpile;
//! C++-only constructs should fail at parsing or HIR conversion.
//!
//! Organization:
//! - C201-C205: C++ features adopted into C99 (should pass)
//! - C206-C210: C++ features NOT in C (likely to fail)
//! - C211-C215: C++ templates and OOP (should fail)
//! - C216-C220: Modern C++ / STL (should fail)
//! - C221-C225: PyTorch-relevant C++ patterns (should fail)
//!
//! Results: 4 passing, 21 falsified (16.0% pass rate)
//! - C201-C203, C205: C99-adopted features transpile correctly
//! - C204: _Bool parses but HIR conversion drops functions (falsified)
//! - C206-C225: C++-only syntax rejected by clang in C mode (falsified)

// ============================================================================
// C201-C205: C++ features that are valid C99 (should pass)
// ============================================================================

#[test]
fn c201_line_comments() {
    // C99 adopted // line comments from C++
    let c_code = r#"
// This is a line comment (C99 feature, originally C++)
int add(int a, int b) {
    // another comment
    return a + b; // inline comment
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C201: Line comments (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C201: Output should not be empty");
    assert!(
        code.contains("fn add"),
        "C201: Should contain transpiled function add"
    );
}

#[test]
fn c202_mid_block_declarations() {
    // C99 adopted mid-block variable declarations from C++
    let c_code = r#"
int compute(int n) {
    int result = 0;
    for (int i = 0; i < n; i++) {
        int temp = i * 2;
        result += temp;
    }
    int final_val = result + 1;
    return final_val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C202: Mid-block declarations (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C202: Output should not be empty");
    assert!(
        code.contains("fn compute"),
        "C202: Should contain transpiled function compute"
    );
}

#[test]
fn c203_inline_functions() {
    // C99 adopted inline keyword from C++
    let c_code = r#"
inline int square(int x) {
    return x * x;
}

int use_square(int n) {
    return square(n) + square(n + 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C203: Inline functions (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C203: Output should not be empty");
    assert!(
        code.contains("fn square") || code.contains("fn use_square"),
        "C203: Should contain transpiled functions"
    );
}

#[test]
fn c204_bool_type() {
    // C99 has _Bool; <stdbool.h> provides bool/true/false macros
    // Previously falsified: clang parses _Bool correctly but the HIR conversion
    // did not handle _Bool return type. Fixed by adding Bool variant through pipeline.
    let c_code = r#"
_Bool is_positive(int x) {
    return x > 0;
}

_Bool both_positive(int a, int b) {
    _Bool a_pos = is_positive(a);
    _Bool b_pos = is_positive(b);
    return a_pos && b_pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C204: _Bool type (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C204: Output should not be empty");
    assert!(
        code.contains("fn is_positive") || code.contains("fn both_positive"),
        "C204: Should contain transpiled bool functions"
    );
}

#[test]
fn c205_designated_initializers_nested() {
    // C99 designated initializers with nested structs
    let c_code = r#"
typedef struct {
    int x;
    int y;
} Point;

typedef struct {
    Point origin;
    int width;
    int height;
} Rect;

Rect make_rect() {
    Rect r = { .origin = { .x = 10, .y = 20 }, .width = 100, .height = 50 };
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C205: Designated initializers (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C205: Output should not be empty");
    assert!(
        code.contains("Point") || code.contains("Rect") || code.contains("fn make_rect"),
        "C205: Should contain transpiled struct or function"
    );
}

// ============================================================================
// C206-C210: C++ features NOT in C (likely to fail/be falsified)
// ============================================================================

#[test]
#[ignore = "FALSIFIED: C++ references not supported by C transpiler - clang rejects & in type position when parsing as C"]
fn c206_cpp_references() {
    // C++ references (int&) are not valid C
    let c_code = r#"
int increment(int& ref) {
    ref += 1;
    return ref;
}

int main() {
    int x = 10;
    int result = increment(x);
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C206: C++ references should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C206: Output should not be empty");
    assert!(
        code.contains("fn increment"),
        "C206: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ default parameter values not valid C syntax - clang rejects when parsing as C"]
fn c207_default_parameter_values() {
    // C++ default parameter values are not valid C
    let c_code = r#"
int add(int a, int b = 10) {
    return a + b;
}

int main() {
    int x = add(5);
    int y = add(5, 20);
    return x + y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C207: C++ default params should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C207: Output should not be empty");
    assert!(
        code.contains("fn add"),
        "C207: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ function overloading not valid C - clang rejects duplicate function names with different signatures"]
fn c208_function_overloading() {
    // C++ function overloading is not valid C
    let c_code = r#"
int add(int a, int b) {
    return a + b;
}

double add(double a, double b) {
    return a + b;
}

int main() {
    int x = add(1, 2);
    double y = add(1.5, 2.5);
    return (int)(x + y);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C208: C++ function overloading should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C208: Output should not be empty");
    assert!(
        code.contains("fn add"),
        "C208: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ class keyword not valid C - clang rejects class declarations when parsing as C"]
fn c209_class_keyword() {
    // C++ class keyword is not valid C
    let c_code = r#"
class Point {
public:
    int x;
    int y;

    int sum() {
        return x + y;
    }
};

int main() {
    Point p;
    p.x = 10;
    p.y = 20;
    return p.sum();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C209: C++ class should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C209: Output should not be empty");
    assert!(
        code.contains("Point") || code.contains("struct"),
        "C209: Should contain transpiled class/struct"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ new/delete operators not valid C - clang rejects new/delete when parsing as C"]
fn c210_new_delete_operators() {
    // C++ new/delete are not valid C
    let c_code = r#"
int main() {
    int* p = new int(42);
    int* arr = new int[10];
    for (int i = 0; i < 10; i++) {
        arr[i] = i * 2;
    }
    int result = *p + arr[5];
    delete p;
    delete[] arr;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C210: C++ new/delete should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C210: Output should not be empty");
    assert!(
        code.contains("fn main"),
        "C210: Should contain transpiled main"
    );
}

// ============================================================================
// C211-C215: C++ templates and OOP (should fail)
// ============================================================================

#[test]
#[ignore = "FALSIFIED: C++ template functions not valid C - clang rejects template<> syntax when parsing as C"]
fn c211_simple_template_function() {
    // C++ template function is not valid C
    let c_code = r#"
template<typename T>
T max_val(T a, T b) {
    return (a > b) ? a : b;
}

int main() {
    int x = max_val(3, 5);
    double y = max_val(1.5, 2.5);
    return (int)(x + y);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C211: C++ template function should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C211: Output should not be empty");
    assert!(
        code.contains("fn max_val"),
        "C211: Should contain transpiled template function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ template classes not valid C - clang rejects template class syntax when parsing as C"]
fn c212_template_class() {
    // C++ template class is not valid C
    let c_code = r#"
template<typename T>
class Stack {
    T data[100];
    int top;
public:
    Stack() : top(-1) {}
    void push(T val) { data[++top] = val; }
    T pop() { return data[top--]; }
    bool empty() { return top < 0; }
};

int main() {
    Stack<int> s;
    s.push(10);
    s.push(20);
    int x = s.pop();
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C212: C++ template class should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C212: Output should not be empty");
    assert!(
        code.contains("Stack"),
        "C212: Should contain transpiled template class"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ inheritance not valid C - clang rejects class inheritance syntax when parsing as C"]
fn c213_class_inheritance() {
    // C++ class inheritance is not valid C
    let c_code = r#"
class Shape {
public:
    int x, y;
    int area() { return 0; }
};

class Rectangle : public Shape {
public:
    int width, height;
    int area() { return width * height; }
};

int main() {
    Rectangle r;
    r.width = 10;
    r.height = 5;
    return r.area();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C213: C++ inheritance should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C213: Output should not be empty");
    assert!(
        code.contains("Rectangle") || code.contains("Shape"),
        "C213: Should contain transpiled classes"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ virtual functions not valid C - clang rejects virtual keyword when parsing as C"]
fn c214_virtual_functions() {
    // C++ virtual functions and polymorphism are not valid C
    let c_code = r#"
class Animal {
public:
    virtual int sound() { return 0; }
    virtual int legs() { return 4; }
};

class Dog : public Animal {
public:
    int sound() override { return 1; }
};

class Bird : public Animal {
public:
    int sound() override { return 2; }
    int legs() override { return 2; }
};

int main() {
    Dog d;
    Bird b;
    Animal* a1 = &d;
    Animal* a2 = &b;
    return a1->sound() + a2->legs();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C214: C++ virtual functions should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C214: Output should not be empty");
    assert!(
        code.contains("Animal") || code.contains("Dog"),
        "C214: Should contain transpiled class hierarchy"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ operator overloading not valid C - clang rejects operator keyword when parsing as C"]
fn c215_operator_overloading() {
    // C++ operator overloading is not valid C
    let c_code = r#"
class Vec2 {
public:
    int x, y;
    Vec2(int x, int y) : x(x), y(y) {}
    Vec2 operator+(const Vec2& other) {
        return Vec2(x + other.x, y + other.y);
    }
    int dot(const Vec2& other) {
        return x * other.x + y * other.y;
    }
};

int main() {
    Vec2 a(1, 2);
    Vec2 b(3, 4);
    Vec2 c = a + b;
    return c.x + c.y + a.dot(b);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C215: C++ operator overloading should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C215: Output should not be empty");
    assert!(
        code.contains("Vec2"),
        "C215: Should contain transpiled class"
    );
}

// ============================================================================
// C216-C220: Modern C++ / STL (should fail)
// ============================================================================

#[test]
#[ignore = "FALSIFIED: C++ auto type deduction not valid C - clang rejects auto as type specifier when parsing as C"]
fn c216_auto_type_deduction() {
    // C++ auto type deduction is not valid C (auto in C means automatic storage, not type inference)
    let c_code = r#"
auto square(int x) {
    return x * x;
}

int main() {
    auto result = square(5);
    auto pi = 3.14159;
    return (int)(result + pi);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C216: C++ auto type deduction should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C216: Output should not be empty");
    assert!(
        code.contains("fn square"),
        "C216: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ range-based for not valid C - clang rejects range-for syntax when parsing as C"]
fn c217_range_based_for() {
    // C++ range-based for loop is not valid C
    let c_code = r#"
int sum_array() {
    int arr[] = {1, 2, 3, 4, 5};
    int sum = 0;
    for (auto& x : arr) {
        sum += x;
    }
    return sum;
}

int main() {
    return sum_array();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C217: C++ range-based for should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C217: Output should not be empty");
    assert!(
        code.contains("fn sum_array"),
        "C217: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ lambda expressions not valid C - clang rejects lambda syntax when parsing as C"]
fn c218_lambda_expressions() {
    // C++ lambda expressions are not valid C
    let c_code = r#"
int apply(int x) {
    auto doubler = [](int n) { return n * 2; };
    auto adder = [x](int n) { return n + x; };
    return doubler(x) + adder(10);
}

int main() {
    return apply(5);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C218: C++ lambda should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C218: Output should not be empty");
    assert!(
        code.contains("fn apply"),
        "C218: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ STL std::vector not valid C - clang rejects #include <vector> and std:: when parsing as C"]
fn c219_std_vector() {
    // C++ std::vector is not valid C
    let c_code = r#"
#include <vector>

int sum_vector() {
    std::vector<int> v;
    v.push_back(10);
    v.push_back(20);
    v.push_back(30);
    int sum = 0;
    for (int i = 0; i < v.size(); i++) {
        sum += v[i];
    }
    return sum;
}

int main() {
    return sum_vector();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C219: C++ std::vector should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C219: Output should not be empty");
    assert!(
        code.contains("fn sum_vector"),
        "C219: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ smart pointers not valid C - clang rejects #include <memory> and std::unique_ptr when parsing as C"]
fn c220_smart_pointers() {
    // C++ smart pointers (unique_ptr, shared_ptr) are not valid C
    let c_code = r#"
#include <memory>

int use_smart_ptr() {
    std::unique_ptr<int> p = std::make_unique<int>(42);
    std::shared_ptr<int> sp = std::make_shared<int>(100);
    return *p + *sp;
}

int main() {
    return use_smart_ptr();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C220: C++ smart pointers should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C220: Output should not be empty");
    assert!(
        code.contains("fn use_smart_ptr"),
        "C220: Should contain transpiled function"
    );
}

// ============================================================================
// C221-C225: PyTorch-relevant C++ patterns
// ============================================================================

#[test]
#[ignore = "FALSIFIED: C++ namespaces not valid C - clang rejects namespace keyword when parsing as C"]
fn c221_namespace_usage() {
    // C++ namespaces are not valid C (common in PyTorch: torch::, at::, c10::)
    let c_code = r#"
namespace math {
    int add(int a, int b) {
        return a + b;
    }

    namespace detail {
        int multiply(int a, int b) {
            return a * b;
        }
    }
}

int main() {
    int x = math::add(3, 4);
    int y = math::detail::multiply(5, 6);
    return x + y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C221: C++ namespaces should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C221: Output should not be empty");
    assert!(
        code.contains("fn add") || code.contains("math"),
        "C221: Should contain transpiled namespace functions"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ exception handling not valid C - clang rejects try/catch/throw when parsing as C"]
fn c222_exception_handling() {
    // C++ try/catch/throw are not valid C
    let c_code = r#"
int safe_divide(int a, int b) {
    if (b == 0) {
        throw -1;
    }
    return a / b;
}

int main() {
    try {
        int result = safe_divide(10, 0);
        return result;
    } catch (int e) {
        return e;
    } catch (...) {
        return -99;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C222: C++ exceptions should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C222: Output should not be empty");
    assert!(
        code.contains("fn safe_divide"),
        "C222: Should contain transpiled function"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ RAII constructors/destructors not valid C - clang rejects constructor/destructor syntax when parsing as C"]
fn c223_raii_pattern() {
    // C++ RAII (Resource Acquisition Is Initialization) is not valid C
    let c_code = r#"
class FileHandle {
    int fd;
public:
    FileHandle(int f) : fd(f) {}
    ~FileHandle() {
        if (fd >= 0) {
            fd = -1;
        }
    }
    int get() { return fd; }
};

int main() {
    FileHandle fh(42);
    return fh.get();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C223: C++ RAII should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C223: Output should not be empty");
    assert!(
        code.contains("FileHandle"),
        "C223: Should contain transpiled RAII class"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ const references not valid C - clang rejects const T& parameter syntax when parsing as C"]
fn c224_const_reference_params() {
    // C++ const references as function parameters are not valid C
    let c_code = r#"
struct Matrix {
    int data[4][4];
    int rows;
    int cols;
};

int trace(const Matrix& m) {
    int sum = 0;
    for (int i = 0; i < m.rows && i < m.cols; i++) {
        sum += m.data[i][i];
    }
    return sum;
}

int main() {
    Matrix m;
    m.rows = 2;
    m.cols = 2;
    m.data[0][0] = 1;
    m.data[1][1] = 4;
    return trace(m);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C224: C++ const references should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C224: Output should not be empty");
    assert!(
        code.contains("fn trace") || code.contains("Matrix"),
        "C224: Should contain transpiled function or struct"
    );
}

#[test]
#[ignore = "FALSIFIED: C++ static class methods not valid C - clang rejects static methods in class when parsing as C"]
fn c225_static_class_methods() {
    // C++ static class methods are not valid C
    let c_code = r#"
class Counter {
    static int count;
public:
    static int get_count() { return count; }
    static void increment() { count++; }
    static void reset() { count = 0; }
};

int Counter::count = 0;

int main() {
    Counter::increment();
    Counter::increment();
    Counter::increment();
    return Counter::get_count();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C225: C++ static methods should transpile (if clang allows): {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C225: Output should not be empty");
    assert!(
        code.contains("Counter"),
        "C225: Should contain transpiled class"
    );
}
