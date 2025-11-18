/* K&R C Chapter 4.4: Scope Rules
 * Page 80-82
 * Variable scope and shadowing
 * Transpiled to safe Rust
 */

static GLOBAL_VAR: i32 = 100;  // Module scope

fn function1() {
    let local_var = 10;  // Function scope
    println!("function1: local_var = {}, global_var = {}",
             local_var, GLOBAL_VAR);
}

fn function2() {
    let global_var = 200;  // Shadows the static global
    println!("function2: global_var (shadowed) = {}", global_var);
}

fn nested_scopes() {
    let x = 1;
    println!("Outer: x = {}", x);

    {
        let x = 2;  // Inner scope shadows outer
        println!("Inner: x = {}", x);

        {
            let x = 3;  // Even more inner
            println!("Innermost: x = {}", x);
        }

        println!("Back to inner: x = {}", x);
    }

    println!("Back to outer: x = {}", x);
}

fn main() {
    println!("Global variable: {}\n", GLOBAL_VAR);

    function1();
    function2();
    println!("After function2, global_var still = {}\n", GLOBAL_VAR);

    nested_scopes();
}

// Rust shadowing is safer than C because:
// 1. Shadowed variable is truly a new binding
// 2. Can change type when shadowing
// 3. Inner scope cannot modify outer scope (without mut ref)
// 4. Compiler warns about unused variables

#[allow(dead_code)]
fn demonstrate_shadowing_features() {
    let x = 5;           // i32
    let x = "hello";     // Can change type!
    let x = x.len();     // And again

    println!("Final x = {}", x);
}
