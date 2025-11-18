/* K&R C Chapter 5: Complicated Pointer Declarations
 * Advanced pointer declaration reading
 * Transpiled to safe Rust
 */

// Function returning reference to static int
fn func1() -> &'static i32 {
    static X: i32 = 42;
    &X
}

// Regular function returning int
fn func2() -> i32 {
    100
}

// Simple functions for function pointer array
fn add() -> i32 { 1 }
fn sub() -> i32 { 2 }
fn mul() -> i32 { 3 }

fn main() {
    // Simple reference
    let x = 10;
    let p = &x;
    println!("let p: &i32 - p points to int, *p = {}", p);

    // Reference to reference
    let pp = &p;
    println!("let pp: &&i32 - reference to reference, **pp = {}", pp);

    // Array of references
    let (a, b, c) = (1, 2, 3);
    let arr: [&i32; 3] = [&a, &b, &c];
    println!("let arr: [&i32; 3] - array of 3 references, arr[1] = {}", arr[1]);

    // Reference to array
    let nums = [10, 20, 30, 40, 50];
    let pa: &[i32; 5] = &nums;
    println!("let pa: &[i32; 5] - reference to array, pa[2] = {}", pa[2]);

    // Function returning reference
    let result = func1();
    println!("fn func1() -> &i32 - function returning reference, *result = {}", result);

    // Function pointer
    let pf: fn() -> i32 = func2;
    println!("let pf: fn() -> i32 - function pointer, pf() = {}", pf());

    // Array of function pointers
    let func_arr: [fn() -> i32; 3] = [add, sub, mul];
    println!("let func_arr: [fn() -> i32; 3] - array of function pointers");
    for (i, f) in func_arr.iter().enumerate() {
        println!("  func_arr[{}]() = {}", i, f());
    }

    // Reference variations
    let _r1: &i32 = &x;           // Immutable reference
    let mut y = x;
    let _r2: &mut i32 = &mut y;   // Mutable reference

    println!("\nReference variations:");
    println!("  let r1: &i32 - immutable reference");
    println!("  let r2: &mut i32 - mutable reference");

    // Complex type (using type alias for clarity)
    type FuncPtr = fn(&i32) -> &'static str;
    let _complex: [FuncPtr; 10] = [|_| "test"; 10];

    println!("\nComplex type:");
    println!("  type FuncPtr = fn(&i32) -> &str");
    println!("  let complex: [FuncPtr; 10]");
    println!("  Array of 10 function pointers");
}

// Demonstrate type aliases for clarity
type IntRef = &'static i32;
type IntArray = [i32; 10];
type FnReturningInt = fn() -> i32;

fn demonstrate_type_aliases() {
    static VALUE: i32 = 42;
    let _r: IntRef = &VALUE;
    let _arr: IntArray = [0; 10];
    let _f: FnReturningInt = || 100;
}

// Key differences from C:
// 1. &T instead of T*
// 2. &mut T for mutable pointers
// 3. fn(A) -> B instead of B (*f)(A)
// 4. [&T; N] instead of T* arr[N]
// 5. Type aliases for complex types
// 6. Lifetime annotations when needed
// 7. Much clearer syntax overall
