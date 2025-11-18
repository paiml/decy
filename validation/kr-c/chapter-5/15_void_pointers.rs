/* K&R C Chapter 5: Void Pointers
 * Generic pointers and type casting
 * Transpiled to safe Rust (using generics and enums)
 */

use std::mem;

// Generic swap using generics (type-safe!)
fn swap<T>(a: &mut T, b: &mut T) {
    mem::swap(a, b);  // Safe, built-in swap
}

// Type-safe print using enum (better than void*)
#[derive(Debug)]
enum Value {
    Int(i32),
    Float(f32),
    Char(char),
    Str(String),
}

fn print_value(val: &Value) {
    match val {
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{:.2}", f),
        Value::Char(c) => println!("{}", c),
        Value::Str(s) => println!("{}", s),
    }
}

fn main() {
    // Swap integers
    let mut x = 10;
    let mut y = 20;
    println!("Before swap: x = {}, y = {}", x, y);
    swap(&mut x, &mut y);
    println!("After swap:  x = {}, y = {}\n", x, y);

    // Swap floats
    let mut f1 = 3.14;
    let mut f2 = 2.71;
    println!("Before swap: f1 = {:.2}, f2 = {:.2}", f1, f2);
    swap(&mut f1, &mut f2);
    println!("After swap:  f1 = {:.2}, f2 = {:.2}\n", f1, f2);

    // Swap characters
    let mut c1 = 'A';
    let mut c2 = 'Z';
    println!("Before swap: c1 = {}, c2 = {}", c1, c2);
    swap(&mut c1, &mut c2);
    println!("After swap:  c1 = {}, c2 = {}\n", c1, c2);

    // Type-safe printing using enum
    println!("Type-safe print function:");
    let values = vec![
        Value::Int(42),
        Value::Float(3.14159),
        Value::Char('X'),
        Value::Str("Hello".to_string()),
    ];

    println!("  int:    ");
    print_value(&values[0]);
    println!("  float:  ");
    print_value(&values[1]);
    println!("  char:   ");
    print_value(&values[2]);
    println!("  string: ");
    print_value(&values[3]);

    // Generic container (Vec<Value> instead of void*[])
    println!("\nGeneric array:");
    for (i, val) in values.iter().enumerate() {
        print!("  [{}] ({:?}): ", i, val);
        print_value(val);
    }
}

// Alternative: using trait objects for dynamic dispatch
trait Printable {
    fn print(&self);
}

impl Printable for i32 {
    fn print(&self) {
        println!("{}", self);
    }
}

impl Printable for f32 {
    fn print(&self) {
        println!("{:.2}", self);
    }
}

fn demonstrate_trait_objects() {
    let values: Vec<Box<dyn Printable>> = vec![
        Box::new(42i32),
        Box::new(3.14f32),
    ];

    for val in values {
        val.print();
    }
}

// Key differences from C:
// 1. Generics instead of void*
// 2. Enums for sum types
// 3. Trait objects for dynamic dispatch
// 4. No unsafe casting needed
// 5. Type safety enforced at compile time
// 6. mem::swap() for safe generic swap
// 7. No runtime type information needed
