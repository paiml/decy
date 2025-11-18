/* K&R C Chapter 5: Pointer Arithmetic with Structs
 * Pointer arithmetic on structure arrays
 * Transpiled to safe Rust (using iterators and slices)
 */

#[derive(Debug)]
struct Student {
    name: String,
    id: i32,
    gpa: f32,
}

fn print_students(students: &[Student]) {
    println!("Using iterator:");
    for student in students {
        println!("  ID {}: {} (GPA: {:.2})", student.id, student.name, student.gpa);
    }
}

fn main() {
    let class = vec![
        Student { name: "Alice".to_string(), id: 1001, gpa: 3.8 },
        Student { name: "Bob".to_string(), id: 1002, gpa: 3.5 },
        Student { name: "Charlie".to_string(), id: 1003, gpa: 3.9 },
        Student { name: "Diana".to_string(), id: 1004, gpa: 3.7 },
        Student { name: "Eve".to_string(), id: 1005, gpa: 3.6 },
    ];

    // Slice access (safe, no pointer arithmetic)
    let first = &class[0];
    let last = &class[class.len() - 1];

    println!("First student: {}", first.name);
    println!("Last student: {}", last.name);
    println!("Number of students: {}", class.len());

    // Iterate with safe slice
    print_students(&class);

    // Access middle element
    let middle = &class[class.len() / 2];
    println!("\nMiddle student: {} (GPA: {:.2})", middle.name, middle.gpa);

    // Filter with iterator (safe, no pointer comparison)
    let count = class.iter().filter(|s| s.gpa >= 3.7).count();
    println!("Students with GPA >= 3.7: {}", count);
}

// Demonstrate iterator patterns (better than pointer arithmetic)
fn demonstrate_iterators() {
    let students = vec![
        Student { name: "Alice".to_string(), id: 1001, gpa: 3.8 },
        Student { name: "Bob".to_string(), id: 1002, gpa: 3.5 },
        Student { name: "Charlie".to_string(), id: 1003, gpa: 3.9 },
    ];

    // Filter
    let high_achievers: Vec<&Student> = students
        .iter()
        .filter(|s| s.gpa >= 3.7)
        .collect();

    println!("High achievers:");
    for student in high_achievers {
        println!("  {}", student.name);
    }

    // Map
    let names: Vec<&str> = students
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    println!("Names: {:?}", names);

    // Average GPA
    let avg_gpa: f32 = students
        .iter()
        .map(|s| s.gpa)
        .sum::<f32>() / students.len() as f32;

    println!("Average GPA: {:.2}", avg_gpa);
}

// Slicing students (safe alternative to pointer arithmetic)
fn demonstrate_slicing() {
    let students = vec![
        Student { name: "Alice".to_string(), id: 1001, gpa: 3.8 },
        Student { name: "Bob".to_string(), id: 1002, gpa: 3.5 },
        Student { name: "Charlie".to_string(), id: 1003, gpa: 3.9 },
        Student { name: "Diana".to_string(), id: 1004, gpa: 3.7 },
        Student { name: "Eve".to_string(), id: 1005, gpa: 3.6 },
    ];

    // First 3 students
    let first_three = &students[0..3];
    println!("First three: {:?}", first_three);

    // Last 2 students
    let last_two = &students[students.len() - 2..];
    println!("Last two: {:?}", last_two);

    // Middle students
    let middle = &students[1..4];
    println!("Middle: {:?}", middle);
}

// Key differences from C:
// 1. Slices (&[Student]) instead of struct student*
// 2. Iterators instead of pointer arithmetic
// 3. .filter(), .map(), .collect() for transformations
// 4. Bounds checking on slice access
// 5. No pointer comparison needed
// 6. Vec automatically manages memory
