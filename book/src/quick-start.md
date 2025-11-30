# Quick Start

Get started with Decy in minutes. This guide covers installation, basic usage, and running examples.

## Installation

### From crates.io

```bash
cargo install decy
```

### From Source

```bash
git clone https://github.com/paiml/decy.git
cd decy
make install
cargo build --release
```

### Prerequisites

Decy requires LLVM/Clang for parsing C code:

```bash
# Ubuntu/Debian
sudo apt install llvm-14-dev libclang-14-dev

# macOS
brew install llvm@14

# Set environment variables
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
```

## Basic Usage

### Transpile a Single File

```bash
# Output to stdout
decy transpile hello.c

# Output to file
decy transpile hello.c -o hello.rs
```

### Example: Hello World

Create `hello.c`:

```c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
```

Transpile it:

```bash
decy transpile hello.c -o hello.rs
```

Result (`hello.rs`):

```rust
fn main() -> i32 {
    println!("Hello, World!");
    0
}
```

### Transpile a Project

```bash
decy transpile-project ./my-c-project -o ./rust-output
```

## Running Examples

Decy includes comprehensive examples demonstrating safety transformations. Run them with `cargo run --example`:

### Safety Demonstration Examples

```bash
# Buffer overflow prevention
cargo run -p decy-core --example buffer_overflow_safety_demo

# Double-free prevention
cargo run -p decy-core --example double_free_safety_demo

# Dynamic memory safety
cargo run -p decy-core --example dynamic_memory_safety_demo

# Format string safety
cargo run -p decy-core --example format_string_safety_demo

# Integer overflow safety
cargo run -p decy-core --example integer_overflow_safety_demo

# Loop and array safety
cargo run -p decy-core --example loop_array_safety_demo

# Null pointer safety
cargo run -p decy-core --example null_pointer_safety_demo

# Pointer arithmetic safety
cargo run -p decy-core --example pointer_arithmetic_safety_demo

# Race condition safety
cargo run -p decy-core --example race_condition_safety_demo

# String safety
cargo run -p decy-core --example string_safety_demo

# Type casting safety
cargo run -p decy-core --example type_casting_safety_demo

# Uninitialized memory safety
cargo run -p decy-core --example uninitialized_memory_safety_demo

# Use-after-free safety
cargo run -p decy-core --example use_after_free_safety_demo
```

### Macro Expansion Examples

```bash
# Macro constant expansion
cargo run --example macro_expansion_constants

# Macro function expansion
cargo run --example macro_expansion_functions

# Macro naming conventions
cargo run --example macro_expansion_naming

# Macro parentheses handling
cargo run --example macro_expansion_parens

# Macro spacing normalization
cargo run --example macro_expansion_spacing

# Macro ternary operator handling
cargo run --example macro_expansion_ternary

# Macro type inference
cargo run --example macro_expansion_type_inference
```

### Code Generation Examples

```bash
# malloc to Box transformation
cargo run -p decy-codegen --example malloc_to_box
```

### Validation Runner

```bash
# Run comprehensive validation
cargo run --example validation_runner
```

## Using Decy Programmatically

```rust
use decy_core::transpile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
        int add(int a, int b) {
            return a + b;
        }
    "#;

    let rust_code = transpile(c_code)?;
    println!("{}", rust_code);
    Ok(())
}
```

Output:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## Interactive REPL

Experiment with transpilation interactively:

```bash
decy repl
```

```
Decy REPL v1.0.2
Type C code to transpile. Type 'exit' to quit.

> int square(int x) { return x * x; }

fn square(x: i32) -> i32 {
    x * x
}

> exit
```

## What's Next?

- [Your First Transpilation](./first-transpilation.md) - Detailed walkthrough
- [How Decy Works](./how-it-works.md) - Architecture overview
- [C-to-Rust Patterns](./patterns/pointers.md) - Pattern reference

---

**Tip**: Run `decy --help` for all available commands and options.
