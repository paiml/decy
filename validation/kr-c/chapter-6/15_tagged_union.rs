/* K&R C Chapter 6: Tagged Unions
 * Variant types with type tag
 * Transpiled to safe Rust (using enum)
 */

// Idiomatic Rust: use enum instead of tagged union
#[derive(Debug, Clone)]
enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Pointer(*const ()),
}

impl Value {
    fn make_int(value: i32) -> Self {
        Value::Int(value)
    }

    fn make_float(value: f32) -> Self {
        Value::Float(value)
    }

    fn make_string(value: &str) -> Self {
        Value::String(value.to_string())
    }

    fn print(&self) {
        match self {
            Value::Int(i) => print!("int: {}", i),
            Value::Float(f) => print!("float: {:.2}", f),
            Value::String(s) => print!("string: \"{}\"", s),
            Value::Pointer(p) => print!("pointer: {:p}", p),
        }
    }

    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Pointer(a), Value::Pointer(b)) => a == b,
            _ => false,
        }
    }
}

fn main() {
    println!("=== Tagged Union Demo ===\n");

    let values = vec![
        Value::make_int(42),
        Value::make_float(3.14),
        Value::make_string("Hello"),
        Value::make_int(100),
        Value::make_float(2.71),
    ];

    println!("Values:");
    for (i, value) in values.iter().enumerate() {
        print!("  [{}] ", i);
        value.print();
        println!();
    }

    println!("\nComparisons:");
    println!("  values[0] == values[3]: {}", values[0].equals(&values[3]));
    println!("  values[1] == values[4]: {}", values[1].equals(&values[4]));

    let v1 = Value::make_int(42);
    let v2 = Value::make_int(42);
    println!("  make_int(42) == make_int(42): {}", v1.equals(&v2));

    println!("\nSize of Value enum: {} bytes", std::mem::size_of::<Value>());
}

// More complex enum with associated data
#[allow(dead_code)]
fn demonstrate_complex_enum() {
    #[derive(Debug)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(u8, u8, u8),
    }

    fn process_message(msg: Message) {
        match msg {
            Message::Quit => println!("Quit"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: RGB({}, {}, {})", r, g, b),
        }
    }

    let messages = vec![
        Message::Move { x: 10, y: 20 },
        Message::Write("Hello".to_string()),
        Message::ChangeColor(255, 0, 0),
        Message::Quit,
    ];

    for msg in messages {
        process_message(msg);
    }
}

// Option and Result (standard library enums)
#[allow(dead_code)]
fn demonstrate_option_result() {
    // Option<T> for nullable values
    fn divide(a: i32, b: i32) -> Option<i32> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }

    match divide(10, 2) {
        Some(result) => println!("10 / 2 = {}", result),
        None => println!("Division by zero"),
    }

    // Result<T, E> for error handling
    fn parse_number(s: &str) -> Result<i32, std::num::ParseIntError> {
        s.parse()
    }

    match parse_number("42") {
        Ok(n) => println!("Parsed: {}", n),
        Err(e) => println!("Error: {}", e),
    }

    // Chaining with ? operator
    fn add_numbers(a: &str, b: &str) -> Result<i32, std::num::ParseIntError> {
        let x = a.parse::<i32>()?;
        let y = b.parse::<i32>()?;
        Ok(x + y)
    }

    if let Ok(sum) = add_numbers("10", "20") {
        println!("Sum: {}", sum);
    }
}

// Key differences from C:
// 1. enum instead of tagged union
// 2. Type-safe variants (no manual tag checking)
// 3. match for exhaustive pattern matching
// 4. Compiler ensures all variants handled
// 5. No undefined behavior from wrong variant access
// 6. Option<T> instead of NULL
// 7. Result<T, E> for error handling
// 8. Enums can have different data per variant
