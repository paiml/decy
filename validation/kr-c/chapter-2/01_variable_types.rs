/* K&R C Chapter 2.2: Data Types and Sizes
 * Page 35-36
 * Basic data type examples
 * Transpiled to safe Rust
 */

fn main() {
    let c: char;
    let i: i32;
    let l: i64;
    let f: f32;
    let d: f64;

    c = 'A';
    i = 42;
    l = 1000000_i64;
    f = 3.14_f32;
    d = 2.71828;

    println!("char: {}", c);
    println!("int: {}", i);
    println!("long: {}", l);
    println!("float: {}", f);
    println!("double: {}", d);
}
