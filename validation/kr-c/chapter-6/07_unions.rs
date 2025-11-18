/* K&R C Chapter 6.8: Unions
 * Page 147-149
 * Union for variant types
 * Transpiled to safe Rust using enums (safer than unions)
 */

// Rust enum is safer and more idiomatic than C unions
enum Variant {
    Int(i32),
    Float(f32),
    String(&'static str),
}

fn main() {
    // Using enum instead of raw union
    let mut uval: Variant;

    // Store int
    uval = Variant::Int(42);
    if let Variant::Int(i) = uval {
        println!("Integer: {}", i);
    }

    // Store float (replaces previous value)
    uval = Variant::Float(3.14);
    if let Variant::Float(f) = uval {
        println!("Float: {:.2}", f);
    }

    // Store string (replaces previous value)
    uval = Variant::String("hello");
    if let Variant::String(s) = uval {
        println!("String: {}", s);
    }

    // Pattern matching on tagged union
    let v = Variant::Int(100);

    match v {
        Variant::Int(i) => println!("Variant int: {}", i),
        Variant::Float(f) => println!("Variant float: {}", f),
        Variant::String(s) => println!("Variant string: {}", s),
    }
}
