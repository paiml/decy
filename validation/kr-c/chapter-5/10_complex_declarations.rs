/* K&R C Chapter 5.12: Complicated Declarations
 * Page 122-123
 * Examples of complex C declarations
 * Transpiled to safe Rust
 */

fn main() {
    // Rust equivalents of complex C declarations:

    // int *ap[10] - array of 10 pointers to int
    let ap: [&i32; 10];
    let x = 42;
    ap = [&x; 10];  // Array of references

    // int (*pa)[10] - pointer to array of 10 ints
    let arr: [i32; 10] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90];
    let pa: &[i32; 10] = &arr;

    println!("(*pa)[5] = {}", pa[5]);
    println!("*ap[0] = {}", ap[0]);

    // Rust is simpler: no complex pointer syntax needed
    // References are always safe and clear
}

// C: int *f() - function returning pointer
fn function_returning_ref() -> &'static i32 {
    static X: i32 = 42;
    &X
}

// C: int (*pf)() - pointer to function
fn demonstrate_function_pointers() {
    let pf: fn() -> i32 = || 42;
    println!("pf() = {}", pf());
}

// Key differences from C:
// 1. &[T; N] instead of (*ptr)[N]
// 2. fn() -> T instead of int (*pf)()
// 3. No ambiguous * and [] combinations
// 4. Lifetime annotations when needed
// 5. Much clearer and safer syntax
