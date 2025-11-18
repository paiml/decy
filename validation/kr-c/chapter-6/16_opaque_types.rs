/* K&R C Chapter 6: Opaque Types (Information Hiding)
 * Forward declarations and opaque pointers
 * Transpiled to safe Rust (using modules and private fields)
 */

// Counter module with opaque implementation
mod counter {
    pub struct Counter {
        value: i32,
        step: i32,
        name: String,
    }

    impl Counter {
        pub fn new(name: &str, initial: i32, step: i32) -> Self {
            Counter {
                value: initial,
                step,
                name: name.to_string(),
            }
        }

        pub fn next(&mut self) -> i32 {
            let current = self.value;
            self.value += self.step;
            current
        }

        pub fn get(&self) -> i32 {
            self.value
        }

        pub fn reset(&mut self, value: i32) {
            self.value = value;
        }

        pub fn print(&self) {
            println!("{}: {}", self.name, self.value);
        }
    }
}

// Stack module with opaque implementation
mod stack {
    pub struct Stack {
        data: Vec<i32>,
        capacity: usize,
    }

    impl Stack {
        pub fn new(capacity: usize) -> Self {
            Stack {
                data: Vec::with_capacity(capacity),
                capacity,
            }
        }

        pub fn push(&mut self, value: i32) -> bool {
            if self.data.len() >= self.capacity {
                return false; // Full
            }
            self.data.push(value);
            true
        }

        pub fn pop(&mut self) -> Option<i32> {
            self.data.pop()
        }

        pub fn is_empty(&self) -> bool {
            self.data.is_empty()
        }

        pub fn len(&self) -> usize {
            self.data.len()
        }
    }
}

fn main() {
    println!("=== Opaque Types Demo ===\n");

    // Counter usage
    let mut c1 = counter::Counter::new("Main", 0, 1);
    let mut c2 = counter::Counter::new("Even", 0, 2);

    println!("Counters:");
    for i in 0..5 {
        print!("  {}: ", i);
        print!("Main={} ", c1.next());
        print!("Even={} ", c2.next());
        println!();
    }

    c1.reset(100);
    println!("\nAfter reset:");
    c1.print();
    c2.print();

    // Stack usage
    println!("\nStack:");
    let mut stack = stack::Stack::new(10);

    println!("Pushing: 10, 20, 30");
    stack.push(10);
    stack.push(20);
    stack.push(30);

    println!("Popping:");
    while let Some(value) = stack.pop() {
        println!("  {}", value);
    }

    if stack.is_empty() {
        println!("Stack is empty");
    }
}

// Advanced: Using newtype pattern for type safety
#[allow(dead_code)]
mod newtype {
    // UserId is opaque - implementation hidden
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UserId(u64);

    impl UserId {
        pub fn new(id: u64) -> Self {
            UserId(id)
        }

        // Public API - no direct access to inner value
        pub fn is_admin(&self) -> bool {
            self.0 < 100
        }
    }

    // ProductId is different type - prevents mix-ups
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProductId(u64);

    impl ProductId {
        pub fn new(id: u64) -> Self {
            ProductId(id)
        }
    }

    pub fn demo() {
        let user = UserId::new(42);
        let product = ProductId::new(100);

        // Type safety: cannot mix UserId and ProductId
        // let wrong = user == product; // ERROR: mismatched types

        println!("User {} is admin: {}", user.0, user.is_admin());
        println!("Product ID: {}", product.0);
    }
}

// Pimpl idiom (Pointer to Implementation)
#[allow(dead_code)]
mod pimpl {
    struct WidgetImpl {
        data: Vec<i32>,
        state: String,
    }

    pub struct Widget {
        pimpl: Box<WidgetImpl>,
    }

    impl Widget {
        pub fn new() -> Self {
            Widget {
                pimpl: Box::new(WidgetImpl {
                    data: Vec::new(),
                    state: "Init".to_string(),
                }),
            }
        }

        pub fn add_data(&mut self, value: i32) {
            self.pimpl.data.push(value);
        }

        pub fn get_state(&self) -> &str {
            &self.pimpl.state
        }
    }
}

// Key differences from C:
// 1. Modules for encapsulation (pub/private)
// 2. Private fields by default
// 3. No manual malloc/free
// 4. RAII: automatic cleanup
// 5. Newtype pattern for type safety
// 6. Option<T> instead of error codes
// 7. Traits for public interfaces
// 8. No need for forward declarations
