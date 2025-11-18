/* K&R C Chapter 6: Anonymous Struct/Union Members (C11)
 * Nested anonymous structs and unions
 * Transpiled to safe Rust (using enum and structs)
 */

// Rust doesn't have anonymous unions, but we can use enums for type-safe variants

#[derive(Debug, Clone, Copy)]
struct Value {
    type_tag: ValueType,
    data: ValueData,
}

#[derive(Debug, Clone, Copy)]
enum ValueType {
    Int,
    Float,
}

#[derive(Clone, Copy)]
union ValueData {
    i: i32,
    f: f32,
}

impl Value {
    fn new_int(value: i32) -> Self {
        Value {
            type_tag: ValueType::Int,
            data: ValueData { i: value },
        }
    }

    fn new_float(value: f32) -> Self {
        Value {
            type_tag: ValueType::Float,
            data: ValueData { f: value },
        }
    }

    fn print(&self) {
        unsafe {
            match self.type_tag {
                ValueType::Int => println!("int: {}", self.data.i),
                ValueType::Float => println!("float: {:.2}", self.data.f),
            }
        }
    }
}

// Better: use enum (type-safe, no unsafe)
#[derive(Debug, Clone, Copy)]
enum ValueSafe {
    Int(i32),
    Float(f32),
}

impl ValueSafe {
    fn print(&self) {
        match self {
            ValueSafe::Int(i) => println!("int: {}", i),
            ValueSafe::Float(f) => println!("float: {:.2}", f),
        }
    }
}

// Point with named fields (no anonymous struct)
#[derive(Debug, Clone, Copy)]
struct Point2D {
    x: f32,
    y: f32,
}

impl Point2D {
    fn as_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    fn from_array(coords: [f32; 2]) -> Self {
        Point2D {
            x: coords[0],
            y: coords[1],
        }
    }
}

// TimeStamp with separate fields
#[derive(Debug, Clone, Copy)]
struct TimeStamp {
    name: [u8; 30],
    hours: i32,
    minutes: i32,
    seconds: i32,
}

impl TimeStamp {
    fn new(name: &str) -> Self {
        let mut name_bytes = [0u8; 30];
        let bytes = name.as_bytes();
        let len = bytes.len().min(29);
        name_bytes[..len].copy_from_slice(&bytes[..len]);

        TimeStamp {
            name: name_bytes,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }

    fn name_str(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap_or("")
            .trim_end_matches('\0')
    }

    fn as_array(&self) -> [i32; 3] {
        [self.hours, self.minutes, self.seconds]
    }

    fn from_array(&mut self, components: [i32; 3]) {
        self.hours = components[0];
        self.minutes = components[1];
        self.seconds = components[2];
    }
}

fn main() {
    println!("=== Anonymous Members (Rust Alternative) ===\n");

    // Value with enum (better than union)
    let v1 = ValueSafe::Int(42);
    let v2 = ValueSafe::Float(3.14);

    print!("Value 1: ");
    v1.print();
    print!("Value 2: ");
    v2.print();

    // Point with explicit fields
    let mut pt = Point2D { x: 10.0, y: 20.0 };

    println!("\nPoint: ({:.1}, {:.1})", pt.x, pt.y);
    let coords = pt.as_array();
    println!("Using array: ({:.1}, {:.1})", coords[0], coords[1]);

    // Modify via array conversion
    pt = Point2D::from_array([30.0, 40.0]);
    println!("After array modification: ({:.1}, {:.1})", pt.x, pt.y);

    // TimeStamp
    let mut ts = TimeStamp::new("Event1");
    ts.hours = 14;
    ts.minutes = 30;
    ts.seconds = 45;

    println!("\nTimeStamp: {}", ts.name_str());
    println!("  Time: {:02}:{:02}:{:02}", ts.hours, ts.minutes, ts.seconds);

    let components = ts.as_array();
    println!("  Array: [{}, {}, {}]", components[0], components[1], components[2]);

    // Modify via array
    ts.from_array([23, 45, 59]);
    println!("  After array modification: {:02}:{:02}:{:02}", ts.hours, ts.minutes, ts.seconds);

    println!("\nSizes:");
    println!("  sizeof(ValueSafe) = {} bytes", std::mem::size_of::<ValueSafe>());
    println!("  sizeof(Point2D) = {} bytes", std::mem::size_of::<Point2D>());
    println!("  sizeof(TimeStamp) = {} bytes", std::mem::size_of::<TimeStamp>());
}

// Key differences from C:
// 1. No anonymous unions in Rust
// 2. enum for type-safe tagged unions
// 3. Named fields required
// 4. Conversion methods (as_array, from_array)
// 5. Pattern matching instead of field access
// 6. unsafe required for union access
// 7. Prefer enum over union for type safety
