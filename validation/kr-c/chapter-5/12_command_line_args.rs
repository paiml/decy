/* K&R C Chapter 5.10: Command-line Arguments
 * Page 114-118
 * Using argc and argv
 * Transpiled to safe Rust
 */

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    println!("Program name: {}", args[0]);
    println!("Number of arguments: {}\n", argc - 1);

    if argc == 1 {
        println!("No arguments provided.");
        println!("Usage: {} <arg1> <arg2> ...", args[0]);
        return;
    }

    println!("Arguments:");
    for i in 1..argc {
        println!("  argv[{}] = \"{}\" (length: {})",
                 i, args[i], args[i].len());
    }

    // Using iterator (safer than pointer arithmetic)
    println!("\nUsing iterator:");
    for arg in args.iter().skip(1) {
        println!("  *ptr = \"{}\"", arg);
    }

    // Search for specific argument
    println!("\nSearching for '-v' flag:");
    let mut found = false;
    for (i, arg) in args.iter().enumerate().skip(1) {
        if arg == "-v" {
            println!("  Found -v flag at position {}", i);
            found = true;
            break;
        }
    }
    if !found {
        println!("  -v flag not found");
    }
}
