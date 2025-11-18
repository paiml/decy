/* K&R C Chapter 6.1: Structure Initialization
 * Page 128-129
 * Transpiled to safe Rust (struct literals and initialization)
 */

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    pt1: Point,
    pt2: Point,
}

#[derive(Debug)]
struct Person {
    name: &'static str,
    age: i32,
    height: f32,
}

fn main() {
    // Basic initialization
    let origin = Point { x: 0, y: 0 };

    // Partial initialization (rest defaults)
    let pt1 = Point { x: 10, y: 0 };

    // Nested structure initialization
    let screen = Rect {
        pt1: Point { x: 0, y: 0 },
        pt2: Point { x: 100, y: 100 },
    };

    // Named field initialization (order doesn't matter)
    let pt2 = Point { y: 20, x: 10 };

    // Complex structure
    let p = Person {
        name: "Alice",
        age: 30,
        height: 5.6,
    };

    println!("Origin: ({}, {})", origin.x, origin.y);
    println!("pt1: ({}, {})", pt1.x, pt1.y);
    println!(
        "Screen: ({},{}) to ({},{})",
        screen.pt1.x, screen.pt1.y, screen.pt2.x, screen.pt2.y
    );
    println!("pt2: ({}, {})", pt2.x, pt2.y);
    println!("Person: {}, age {}, height {:.1}", p.name, p.age, p.height);
}

// Advanced initialization patterns
#[allow(dead_code)]
fn demonstrate_advanced_init() {
    // Default trait
    #[derive(Default)]
    struct Config {
        width: i32,
        height: i32,
        fullscreen: bool,
    }

    let config = Config::default();
    println!("Config: {}x{}", config.width, config.height);

    // Struct update syntax
    let config2 = Config {
        fullscreen: true,
        ..config
    };
    println!("Config2 fullscreen: {}", config2.fullscreen);

    // Builder pattern
    struct WindowBuilder {
        width: i32,
        height: i32,
        title: String,
    }

    impl WindowBuilder {
        fn new() -> Self {
            WindowBuilder {
                width: 800,
                height: 600,
                title: "Window".to_string(),
            }
        }

        fn width(mut self, width: i32) -> Self {
            self.width = width;
            self
        }

        fn height(mut self, height: i32) -> Self {
            self.height = height;
            self
        }

        fn title(mut self, title: &str) -> Self {
            self.title = title.to_string();
            self
        }
    }

    let window = WindowBuilder::new()
        .width(1920)
        .height(1080)
        .title("Game");

    println!("Window: {} ({}x{})", window.title, window.width, window.height);
}

// Key differences from C:
// 1. No partial initialization zeroing - all fields must be specified
// 2. Default trait for default values
// 3. Struct update syntax: { field: value, ..other }
// 4. Field order doesn't matter in initialization
// 5. No implicit conversions
// 6. Builder pattern for complex initialization
// 7. Type inference for nested structures
