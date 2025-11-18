/**
 * Sprint 19 Feature Validation Test File
 * Transpiled to safe Rust
 *
 * This file demonstrates how Sprint 19 C features map to Rust:
 * - Global variables → static/const
 * - Enums with explicit values → Rust enums with discriminants
 * - Structs → Rust structs
 * - Cast expressions → 'as' keyword
 * - Compound literals → Struct literals
 * - Designated initializers → Named field initialization
 */

// ============================================================================
// DECY-058: Global Variables with Storage Class Specifiers
// ============================================================================

// Regular global variable (static mut in Rust)
static mut GLOBAL_COUNTER: i32 = 0;

// Static global (file-local) - static mut
static mut FILE_LOCAL_STATE: i32 = 100;

// Const global
const MAX_BUFFER_SIZE: i32 = 4096;

// Combined: static const
const VERSION: &str = "1.0.0";

// ============================================================================
// DECY-062: Enums with Explicit Values
// ============================================================================

// Enum with explicit values
#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
enum Status {
    StatusOk = 0,
    StatusError = 1,
    StatusPending = 2,
}

// Enum with hex values (bitmask pattern)
#[repr(i32)]
#[derive(Copy, Clone)]
enum FileMode {
    ModeRead = 0x01,
    ModeWrite = 0x02,
    ModeExecute = 0x04,
    ModeReadWrite = 0x03,
    ModeAll = 0x07,
}

// Enum with mixed explicit/implicit
#[repr(i32)]
#[derive(Copy, Clone)]
enum Priority {
    PriorityLow = 0,
    PriorityNormal = 1,
    PriorityHigh = 2,
    PriorityCritical = 100,
}

// ============================================================================
// DECY-058 + DECY-062: Struct Definition
// ============================================================================

#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone)]
struct Color {
    r: i32,
    g: i32,
    b: i32,
    a: i32,
}

#[derive(Copy, Clone)]
struct Config {
    width: i32,
    height: i32,
    status: Status,
}

// ============================================================================
// Functions using Sprint 19 features
// ============================================================================

// Function with cast expressions (DECY-059)
fn test_cast_expressions() -> i32 {
    // Float to int cast
    let pi: f64 = 3.14159;
    let rounded: i32 = pi as i32;

    // Pointer cast (using Box for heap allocation)
    let generic_ptr: Box<[u8; 100]> = Box::new([0u8; 100]);
    // In Rust, we don't need explicit pointer casts

    // Size cast
    let size: usize = 1024;
    let size_int: i32 = size as i32;

    // Box automatically freed when it goes out of scope
    return rounded;
}

// Function with compound literals (DECY-060)
fn create_point() -> Point {
    // Compound literal → struct literal
    Point { x: 10, y: 20 }
}

// Function with designated initializers (DECY-061)
fn create_color() -> Color {
    // Designated initializer → named fields
    Color { r: 255, g: 128, b: 64, a: 255 }
}

// Function using global variables (DECY-058)
fn increment_global() {
    unsafe {
        GLOBAL_COUNTER += 1;
        FILE_LOCAL_STATE -= 1;
    }
}

// Function with compound literal in call
fn draw_point(p: Point) {
    println!("Point: ({}, {})", p.x, p.y);
}

fn test_compound_literal_in_call() {
    // Compound literal passed as argument
    draw_point(Point { x: 100, y: 200 });
}

// Function with array compound literal
fn sum_array() -> i32 {
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let mut sum: i32 = 0;
    for i in 0..5 {
        sum += arr[i];
    }
    return sum;
}

// Function with designated initializers (DECY-061)
fn create_config() -> Config {
    // Partial initialization with designated initializers
    let cfg = Config {
        width: 800,
        height: 600,
        status: Status::StatusOk,
    };
    return cfg;
}

// Function with out-of-order designated initializers
fn create_point_designated() -> Point {
    // Out of order is valid in Rust too
    let p = Point { y: 50, x: 25 };
    return p;
}

// Function with nested compound literals
struct NestedStruct {
    position: Point,
    color: Color,
}

fn create_nested() -> NestedStruct {
    NestedStruct {
        position: Point { x: 0, y: 0 },
        color: Color { r: 255, g: 255, b: 255, a: 255 },
    }
}

// Function with enum usage
fn check_status(code: i32) -> Status {
    if code == 0 {
        return Status::StatusOk;
    } else if code < 0 {
        return Status::StatusError;
    } else {
        return Status::StatusPending;
    }
}

// Function with bitmask enum
fn check_permissions(mode: i32) -> i32 {
    // Bitmask operations with enum values
    let mode_read = FileMode::ModeRead as i32;
    let mode_write = FileMode::ModeWrite as i32;

    if (mode & mode_read != 0) && (mode & mode_write != 0) {
        return 1;  // Has both read and write
    }
    return 0;
}

// Main function demonstrating all features
fn main() {
    println!("Sprint 19 Feature Validation");
    println!("============================\n");

    // Test global variables
    unsafe {
        println!("Global counter: {}", GLOBAL_COUNTER);
    }
    increment_global();
    unsafe {
        println!("After increment: {}", GLOBAL_COUNTER);
    }

    // Test cast expressions
    let result = test_cast_expressions();
    println!("Cast test result: {}", result);

    // Test compound literals
    let p1 = create_point();
    println!("Point: ({}, {})", p1.x, p1.y);

    // Test designated initializers
    let c1 = create_color();
    println!("Color: RGBA({}, {}, {}, {})", c1.r, c1.g, c1.b, c1.a);

    // Test compound literal in call
    test_compound_literal_in_call();

    // Test array compound literal
    let sum = sum_array();
    println!("Array sum: {}", sum);

    // Test config creation
    let cfg = create_config();
    println!("Config: {}x{}, status={}", cfg.width, cfg.height, cfg.status as i32);

    // Test enum functions
    let status = check_status(0);
    println!("Status check: {}", status as i32);

    // Test bitmask enum
    let has_rw = check_permissions(FileMode::ModeReadWrite as i32);
    println!("Has read/write: {}", has_rw);

    println!("\n✅ All Sprint 19 features tested!");
}
