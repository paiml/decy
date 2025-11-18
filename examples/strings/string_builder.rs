// Dynamic string builder
// Transpiled to safe Rust using String (automatic memory management)

struct StringBuilder {
    data: String,
}

impl StringBuilder {
    fn create() -> StringBuilder {
        StringBuilder {
            data: String::with_capacity(16),
        }
    }

    fn append(&mut self, s: &str) {
        self.data.push_str(s);
    }

    fn append_char(&mut self, c: char) {
        self.data.push(c);
    }

    fn to_string(&self) -> String {
        self.data.clone()
    }

    fn length(&self) -> usize {
        self.data.len()
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

fn main() {
    let mut sb = StringBuilder::create();

    sb.append("Hello");
    sb.append_char(' ');
    sb.append("World");
    sb.append_char('!');

    println!("{}", sb.data);
    println!("Length: {}, Capacity: {}", sb.length(), sb.capacity());

    let result = sb.to_string();
    println!("Result: {}", result);

    // No manual freeing needed - Rust handles cleanup automatically
}
