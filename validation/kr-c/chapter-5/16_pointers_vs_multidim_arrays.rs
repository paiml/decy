/* K&R C Chapter 5.9: Pointers vs. Multi-dimensional Arrays
 * Page 110-113
 * Difference between array of pointers and 2D array
 * Transpiled to safe Rust
 */

fn main() {
    // Array of string slices - each can have different length
    let month_names = [
        "Illegal month",
        "January", "February", "March",
        "April", "May", "June",
        "July", "August", "September",
        "October", "November", "December"
    ];

    // Fixed-size 2D array
    let days = [
        "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
    ];

    println!("Array of string slices (variable length):");
    for (i, &month) in month_names.iter().enumerate() {
        println!("  month_names[{:2}] = \"{}\"", i, month);
    }

    println!("\nFixed array:");
    for (i, &day) in days.iter().enumerate() {
        println!("  days[{}] = \"{}\"", i, day);
    }

    // Memory layout
    println!("\nMemory layout:");
    println!("  size_of month_names = {} bytes (array of {} slices)",
             std::mem::size_of_val(&month_names),
             month_names.len());
    println!("  size_of days = {} bytes (7 string slices)",
             std::mem::size_of_val(&days));

    // Accessing elements
    println!("\nAccessing elements:");
    println!("  month_names[5] = \"{}\"", month_names[5]);
    println!("  days[3] = \"{}\"", days[3]);

    // Slice iteration (safe, no pointer arithmetic needed)
    println!("\nIterating with slice (skip first):");
    for (i, &month) in month_names[1..].iter().enumerate() {
        println!("  Month {:2}: {}", i + 1, month);
    }
}

// Demonstrate true 2D array vs array of slices
fn demonstrate_2d_arrays() {
    // 2D array with fixed dimensions
    let matrix: [[i32; 4]; 3] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
    ];

    println!("2D array:");
    for row in &matrix {
        for &val in row {
            print!("{:3} ", val);
        }
        println!();
    }

    // Array of slices (more flexible)
    let jagged: Vec<Vec<i32>> = vec![
        vec![1, 2],
        vec![3, 4, 5, 6],
        vec![7],
    ];

    println!("\nJagged array (Vec of Vec):");
    for row in &jagged {
        for &val in row {
            print!("{:3} ", val);
        }
        println!();
    }
}

// Key differences from C:
// 1. &[&str] instead of char**
// 2. [[T; N]; M] for 2D arrays
// 3. Vec<Vec<T>> for jagged arrays
// 4. Slice operations instead of pointer arithmetic
// 5. No manual pointer management
// 6. Size known at compile time or runtime checked
