/* K&R C Chapter 6: Structure Array Operations
 * Sorting, searching, filtering arrays of structures
 * Transpiled to safe Rust (using slices and iterators)
 */

#[derive(Debug, Clone)]
struct Student {
    id: i32,
    name: String,
    score: f32,
}

impl Student {
    fn new(id: i32, name: &str, score: f32) -> Self {
        Student {
            id,
            name: name.to_string(),
            score,
        }
    }
}

fn print_students(arr: &[Student], title: &str) {
    println!("{}:", title);
    for student in arr {
        println!(
            "  ID:{} {:<20} Score:{:.1}",
            student.id, student.name, student.score
        );
    }
    println!();
}

fn find_by_id(arr: &[Student], id: i32) -> Option<&Student> {
    arr.iter().find(|s| s.id == id)
}

fn filter_by_score(arr: &[Student], min_score: f32) -> Vec<Student> {
    arr.iter()
        .filter(|s| s.score >= min_score)
        .cloned()
        .collect()
}

fn main() {
    let mut students = vec![
        Student::new(1003, "Charlie", 85.5),
        Student::new(1001, "Alice", 92.0),
        Student::new(1005, "Eve", 78.5),
        Student::new(1002, "Bob", 88.0),
        Student::new(1004, "Diana", 95.5),
    ];

    print_students(&students, "Original");

    // Sort by ID
    students.sort_by_key(|s| s.id);
    print_students(&students, "Sorted by ID");

    // Sort by name
    students.sort_by(|a, b| a.name.cmp(&b.name));
    print_students(&students, "Sorted by Name");

    // Sort by score (descending)
    students.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    print_students(&students, "Sorted by Score (desc)");

    // Find student
    if let Some(found) = find_by_id(&students, 1002) {
        println!(
            "Found student ID 1002: {} (Score: {:.1})\n",
            found.name, found.score
        );
    }

    // Filter by score
    let high_scorers = filter_by_score(&students, 90.0);
    print_students(&high_scorers, "Students with score >= 90");
}

// Advanced operations using iterators
#[allow(dead_code)]
fn demonstrate_advanced_operations() {
    let students = vec![
        Student::new(1, "Alice", 92.0),
        Student::new(2, "Bob", 85.0),
        Student::new(3, "Charlie", 78.0),
        Student::new(4, "Diana", 95.0),
    ];

    // Map: transform each element
    let names: Vec<String> = students.iter().map(|s| s.name.clone()).collect();
    println!("Names: {:?}", names);

    // Average score
    let avg = students.iter().map(|s| s.score).sum::<f32>() / students.len() as f32;
    println!("Average score: {:.2}", avg);

    // Max/min score
    if let Some(max) = students.iter().map(|s| s.score).max_by(|a, b| a.partial_cmp(b).unwrap())
    {
        println!("Max score: {:.1}", max);
    }

    // Partition: split into two groups
    let (pass, fail): (Vec<_>, Vec<_>) = students.iter().partition(|s| s.score >= 80.0);
    println!("Pass: {} students", pass.len());
    println!("Fail: {} students", fail.len());

    // Group by score ranges
    let mut excellent = Vec::new();
    let mut good = Vec::new();
    let mut average = Vec::new();

    for s in &students {
        if s.score >= 90.0 {
            excellent.push(s);
        } else if s.score >= 80.0 {
            good.push(s);
        } else {
            average.push(s);
        }
    }

    println!("Excellent: {} students", excellent.len());
    println!("Good: {} students", good.len());
    println!("Average: {} students", average.len());

    // Binary search (requires sorted array)
    let mut sorted = students.clone();
    sorted.sort_by_key(|s| s.id);

    let result = sorted.binary_search_by_key(&2, |s| s.id);
    match result {
        Ok(idx) => println!("Found student at index {}", idx),
        Err(_) => println!("Student not found"),
    }
}

// Key differences from C:
// 1. Vec<T> instead of array + size
// 2. Slices (&[T]) for array views
// 3. sort_by, sort_by_key for sorting
// 4. Iterator methods (filter, map, find)
// 5. No manual qsort comparator
// 6. Closures instead of function pointers
// 7. Option<&T> instead of NULL pointer
// 8. collect() to build new collections
