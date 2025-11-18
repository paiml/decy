/* K&R C Chapter 6: Structure Design Patterns
 * K&R §6: Advanced structure usage patterns
 * Transpiled to safe Rust (using traits and enums)
 */

// ========== Factory Pattern ==========

trait Shape {
    fn area(&self) -> f32;
}

struct Circle {
    radius: f32,
}

impl Shape for Circle {
    fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }
}

struct Rectangle {
    width: f32,
    height: f32,
}

impl Shape for Rectangle {
    fn area(&self) -> f32 {
        self.width * self.height
    }
}

// Factory function
fn create_shape(shape_type: &str, params: &[f32]) -> Option<Box<dyn Shape>> {
    match shape_type {
        "circle" if params.len() >= 1 => Some(Box::new(Circle { radius: params[0] })),
        "rectangle" if params.len() >= 2 => Some(Box::new(Rectangle {
            width: params[0],
            height: params[1],
        })),
        _ => None,
    }
}

fn factory_pattern_demo() {
    println!("=== Factory Pattern ===");

    let circle = create_shape("circle", &[5.0]).unwrap();
    let rectangle = create_shape("rectangle", &[4.0, 6.0]).unwrap();

    println!("Circle area:    {:.2}", circle.area());
    println!("Rectangle area: {:.2}\n", rectangle.area());
}

// ========== Builder Pattern ==========

#[derive(Debug, Default)]
struct Person {
    name: String,
    age: i32,
    email: String,
    phone: String,
}

struct PersonBuilder {
    person: Person,
}

impl PersonBuilder {
    fn new() -> Self {
        PersonBuilder {
            person: Person::default(),
        }
    }

    fn name(mut self, name: &str) -> Self {
        self.person.name = name.to_string();
        self
    }

    fn age(mut self, age: i32) -> Self {
        self.person.age = age;
        self
    }

    fn email(mut self, email: &str) -> Self {
        self.person.email = email.to_string();
        self
    }

    fn build(self) -> Person {
        self.person
    }
}

fn builder_pattern_demo() {
    println!("=== Builder Pattern ===");

    let person = PersonBuilder::new()
        .name("Alice")
        .age(30)
        .email("alice@example.com")
        .build();

    println!("Person:");
    println!("  Name:  {}", person.name);
    println!("  Age:   {}", person.age);
    println!("  Email: {}\n", person.email);
}

// ========== Singleton Pattern ==========

use std::sync::{Arc, Mutex, Once};

struct Config {
    value: i32,
    message: String,
}

static mut SINGLETON: Option<Arc<Mutex<Config>>> = None;
static INIT: Once = Once::new();

fn get_config() -> Arc<Mutex<Config>> {
    unsafe {
        INIT.call_once(|| {
            let config = Config {
                value: 42,
                message: "Default config".to_string(),
            };
            SINGLETON = Some(Arc::new(Mutex::new(config)));
        });
        SINGLETON.clone().unwrap()
    }
}

fn singleton_pattern_demo() {
    println!("=== Singleton Pattern ===");

    let config1 = get_config();
    let config2 = get_config();

    println!(
        "Same instance: {}",
        Arc::ptr_eq(&config1, &config2)
    );

    if let Ok(cfg) = config1.lock() {
        println!("Config value: {}\n", cfg.value);
    }
}

// ========== Observer Pattern ==========

trait Observer {
    fn notify(&self, value: i32);
}

struct PrintObserver;

impl Observer for PrintObserver {
    fn notify(&self, value: i32) {
        println!("  Observer notified: value = {}", value);
    }
}

struct Subject {
    observers: Vec<Box<dyn Observer>>,
    value: i32,
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: Vec::new(),
            value: 0,
        }
    }

    fn attach(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    fn set_value(&mut self, value: i32) {
        self.value = value;
        self.notify();
    }

    fn notify(&self) {
        for observer in &self.observers {
            observer.notify(self.value);
        }
    }
}

fn observer_pattern_demo() {
    println!("=== Observer Pattern ===");

    let mut subject = Subject::new();

    subject.attach(Box::new(PrintObserver));
    subject.attach(Box::new(PrintObserver));

    println!("Setting value to 10:");
    subject.set_value(10);

    println!("Setting value to 20:");
    subject.set_value(20);
    println!();
}

// ========== Iterator Pattern ==========

struct ArrayIterator {
    data: Vec<i32>,
    current: usize,
}

impl ArrayIterator {
    fn new(data: Vec<i32>) -> Self {
        ArrayIterator { data, current: 0 }
    }
}

impl Iterator for ArrayIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.data.len() {
            let value = self.data[self.current];
            self.current += 1;
            Some(value)
        } else {
            None
        }
    }
}

fn iterator_pattern_demo() {
    println!("=== Iterator Pattern ===");

    let data = vec![10, 20, 30, 40, 50];
    let mut iter = ArrayIterator::new(data);

    println!("Iterating through data:");
    while let Some(value) = iter.next() {
        println!("  {}", value);
    }
    println!();
}

// ========== Strategy Pattern ==========

trait SortStrategy {
    fn compare(&self, a: i32, b: i32) -> std::cmp::Ordering;
}

struct Ascending;
impl SortStrategy for Ascending {
    fn compare(&self, a: i32, b: i32) -> std::cmp::Ordering {
        a.cmp(&b)
    }
}

struct Descending;
impl SortStrategy for Descending {
    fn compare(&self, a: i32, b: i32) -> std::cmp::Ordering {
        b.cmp(&a)
    }
}

fn sort_with_strategy(arr: &mut [i32], strategy: &dyn SortStrategy) {
    arr.sort_by(|a, b| strategy.compare(*a, *b));
}

fn strategy_pattern_demo() {
    println!("=== Strategy Pattern ===");

    let mut data = vec![5, 2, 8, 1, 9];

    println!("Original: {:?}", data);

    sort_with_strategy(&mut data, &Ascending);
    println!("Ascending: {:?}", data);

    sort_with_strategy(&mut data, &Descending);
    println!("Descending: {:?}\n", data);
}

// ========== Composite Pattern ==========

trait Component {
    fn operation(&self) -> i32;
}

struct Leaf;

impl Component for Leaf {
    fn operation(&self) -> i32 {
        1
    }
}

struct Composite {
    children: Vec<Box<dyn Component>>,
}

impl Component for Composite {
    fn operation(&self) -> i32 {
        self.children.iter().map(|c| c.operation()).sum()
    }
}

fn composite_pattern_demo() {
    println!("=== Composite Pattern ===");

    let composite = Composite {
        children: vec![Box::new(Leaf), Box::new(Leaf), Box::new(Leaf)],
    };

    println!("Leaf operation:      {}", Leaf.operation());
    println!("Composite operation: {}\n", composite.operation());
}

fn main() {
    println!("=== Structure Design Patterns ===\n");

    factory_pattern_demo();
    builder_pattern_demo();
    singleton_pattern_demo();
    observer_pattern_demo();
    iterator_pattern_demo();
    strategy_pattern_demo();
    composite_pattern_demo();

    println!("Design patterns in Rust:");
    println!("  - Factory:    Trait objects for abstraction");
    println!("  - Builder:    Method chaining with self");
    println!("  - Singleton:  Arc + Mutex + Once");
    println!("  - Observer:   Vec<Box<dyn Trait>>");
    println!("  - Iterator:   Iterator trait");
    println!("  - Strategy:   Trait for interchangeable algorithms");
    println!("  - Composite:  Recursive trait objects");
}

// Key differences from C:
// 1. Traits instead of function pointers
// 2. trait objects (dyn Trait) for polymorphism
// 3. Arc + Mutex for thread-safe singleton
// 4. Iterator trait (built-in)
// 5. Method chaining for builders
// 6. No manual memory management
// 7. RAII for cleanup
// 8. Type safety enforced at compile time
