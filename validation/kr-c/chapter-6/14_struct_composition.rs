/* K&R C Chapter 6: Structure Composition
 * Building complex structures from simpler ones
 * Transpiled to safe Rust (composition and methods)
 */

#[derive(Debug, Clone, Copy)]
struct Point2D {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy)]
struct Rectangle {
    min: Point2D,
    max: Point2D,
}

impl Rectangle {
    fn area(&self) -> f64 {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        width * height
    }

    fn center(&self) -> Point2D {
        Point2D {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Particle {
    position: Point3D,
    velocity: Point3D,
    acceleration: Point3D,
}

impl Particle {
    fn update(&mut self, dt: f64) {
        // Update velocity
        self.velocity.x += self.acceleration.x * dt;
        self.velocity.y += self.acceleration.y * dt;
        self.velocity.z += self.acceleration.z * dt;

        // Update position
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.position.z += self.velocity.z * dt;
    }
}

#[derive(Debug, Clone)]
struct Shape {
    name: String,
    bounds: Rectangle,
    center: Point2D,
}

fn main() {
    // Rectangle example
    let rect = Rectangle {
        min: Point2D { x: 0.0, y: 0.0 },
        max: Point2D { x: 10.0, y: 5.0 },
    };

    println!(
        "Rectangle: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        rect.min.x, rect.min.y, rect.max.x, rect.max.y
    );
    println!("Area: {:.2}", rect.area());

    let center = rect.center();
    println!("Center: ({:.1}, {:.1})\n", center.x, center.y);

    // Particle simulation
    let mut p = Particle {
        position: Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        velocity: Point3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        acceleration: Point3D {
            x: 0.0,
            y: -9.8,
            z: 0.0,
        },
    };

    println!("Particle simulation (dt=0.1):");
    for i in 0..5 {
        println!(
            "  t={:.1}: pos=({:.2}, {:.2}, {:.2}) vel=({:.2}, {:.2}, {:.2})",
            i as f64 * 0.1,
            p.position.x,
            p.position.y,
            p.position.z,
            p.velocity.x,
            p.velocity.y,
            p.velocity.z
        );
        p.update(0.1);
    }

    // Composite shape
    let shape = Shape {
        name: "Box".to_string(),
        bounds: Rectangle {
            min: Point2D { x: 0.0, y: 0.0 },
            max: Point2D { x: 100.0, y: 50.0 },
        },
        center: Point2D { x: 50.0, y: 25.0 },
    };

    println!("\nShape: {}", shape.name);
    println!(
        "  Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        shape.bounds.min.x,
        shape.bounds.min.y,
        shape.bounds.max.x,
        shape.bounds.max.y
    );
    println!("  Center: ({:.1}, {:.1})", shape.center.x, shape.center.y);
}

// Advanced composition patterns
#[allow(dead_code)]
fn demonstrate_traits() {
    trait Area {
        fn area(&self) -> f64;
    }

    trait Centroid {
        fn centroid(&self) -> Point2D;
    }

    struct Circle {
        center: Point2D,
        radius: f64,
    }

    impl Area for Circle {
        fn area(&self) -> f64 {
            std::f64::consts::PI * self.radius * self.radius
        }
    }

    impl Centroid for Circle {
        fn centroid(&self) -> Point2D {
            self.center
        }
    }

    impl Area for Rectangle {
        fn area(&self) -> f64 {
            Rectangle::area(self)
        }
    }

    impl Centroid for Rectangle {
        fn centroid(&self) -> Point2D {
            Rectangle::center(self)
        }
    }

    // Polymorphism via trait objects
    let shapes: Vec<Box<dyn Area>> = vec![
        Box::new(Circle {
            center: Point2D { x: 0.0, y: 0.0 },
            radius: 5.0,
        }),
        Box::new(Rectangle {
            min: Point2D { x: 0.0, y: 0.0 },
            max: Point2D { x: 10.0, y: 5.0 },
        }),
    ];

    for (i, shape) in shapes.iter().enumerate() {
        println!("Shape {} area: {:.2}", i, shape.area());
    }
}

// Key differences from C:
// 1. impl blocks for methods
// 2. &self and &mut self for method receivers
// 3. No manual pointer passing
// 4. Traits for polymorphism
// 5. Trait objects (dyn Trait) for runtime polymorphism
// 6. Copy trait for value semantics
// 7. No typedef needed (type aliases optional)
// 8. Method chaining with builder pattern
