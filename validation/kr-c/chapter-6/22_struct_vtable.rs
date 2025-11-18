/* K&R C Chapter 6: Virtual Function Tables (VTables)
 * Simulating polymorphism with function pointers
 * Transpiled to safe Rust (using trait objects)
 */

// Trait for polymorphic shapes
trait Shape {
    fn area(&self) -> f32;
    fn perimeter(&self) -> f32;
    fn draw(&self);
    fn name(&self) -> &str;
    fn position(&self) -> (f32, f32);
}

// Circle implementation
struct Circle {
    name: String,
    x: f32,
    y: f32,
    radius: f32,
}

impl Circle {
    fn new(name: &str, x: f32, y: f32, radius: f32) -> Self {
        Circle {
            name: name.to_string(),
            x,
            y,
            radius,
        }
    }
}

impl Shape for Circle {
    fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f32 {
        2.0 * std::f32::consts::PI * self.radius
    }

    fn draw(&self) {
        println!(
            "  Drawing circle '{}' at ({:.1},{:.1}) radius={:.1}",
            self.name, self.x, self.y, self.radius
        );
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

// Rectangle implementation
struct Rectangle {
    name: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rectangle {
    fn new(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            name: name.to_string(),
            x,
            y,
            width,
            height,
        }
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f32 {
        self.width * self.height
    }

    fn perimeter(&self) -> f32 {
        2.0 * (self.width + self.height)
    }

    fn draw(&self) {
        println!(
            "  Drawing rectangle '{}' at ({:.1},{:.1}) size={:.1}x{:.1}",
            self.name, self.x, self.y, self.width, self.height
        );
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

// Triangle implementation
struct Triangle {
    name: String,
    x: f32,
    y: f32,
    base: f32,
    height: f32,
}

impl Triangle {
    fn new(name: &str, x: f32, y: f32, base: f32, height: f32) -> Self {
        Triangle {
            name: name.to_string(),
            x,
            y,
            base,
            height,
        }
    }
}

impl Shape for Triangle {
    fn area(&self) -> f32 {
        0.5 * self.base * self.height
    }

    fn perimeter(&self) -> f32 {
        // Simplified: assuming equilateral
        self.base * 3.0
    }

    fn draw(&self) {
        println!(
            "  Drawing triangle '{}' at ({:.1},{:.1}) base={:.1} height={:.1}",
            self.name, self.x, self.y, self.base, self.height
        );
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

fn main() {
    println!("=== VTable Polymorphism (Trait Objects) ===\n");

    // Create shapes (boxed for trait objects)
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle::new("Circle1", 0.0, 0.0, 5.0)),
        Box::new(Circle::new("Circle2", 10.0, 10.0, 3.0)),
        Box::new(Rectangle::new("Rect1", 5.0, 5.0, 10.0, 20.0)),
        Box::new(Rectangle::new("Rect2", 15.0, 15.0, 8.0, 12.0)),
        Box::new(Triangle::new("Triangle1", 20.0, 20.0, 6.0, 8.0)),
    ];

    // Call virtual functions polymorphically
    println!("Shape operations:");
    for shape in &shapes {
        shape.draw();
        println!("    Area: {:.2}, Perimeter: {:.2}", shape.area(), shape.perimeter());
    }

    // Calculate total area
    let total_area: f32 = shapes.iter().map(|s| s.area()).sum();
    println!("\nTotal area: {:.2}", total_area);
}

// Alternative: enum dispatch (faster, but closed set)
#[allow(dead_code)]
mod enum_dispatch {
    pub enum Shape {
        Circle { x: f32, y: f32, radius: f32 },
        Rectangle { x: f32, y: f32, width: f32, height: f32 },
    }

    impl Shape {
        pub fn area(&self) -> f32 {
            match self {
                Shape::Circle { radius, .. } => std::f32::consts::PI * radius * radius,
                Shape::Rectangle { width, height, .. } => width * height,
            }
        }

        pub fn draw(&self) {
            match self {
                Shape::Circle { x, y, radius } => {
                    println!("Circle at ({}, {}) radius {}", x, y, radius);
                }
                Shape::Rectangle { x, y, width, height } => {
                    println!("Rectangle at ({}, {}) size {}x{}", x, y, width, height);
                }
            }
        }
    }
}

// Key differences from C:
// 1. Traits instead of manual vtables
// 2. Compiler generates vtable automatically
// 3. trait objects (dyn Trait) for polymorphism
// 4. Box<dyn Trait> for heap-allocated trait objects
// 5. No manual casting or unsafe code
// 6. Type safety enforced
// 7. enum dispatch as alternative (faster)
// 8. No memory leaks (RAII)
