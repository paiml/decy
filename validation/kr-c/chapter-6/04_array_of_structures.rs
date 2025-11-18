/* K&R C Chapter 6.3: Arrays of Structures
 * Page 130-131
 * Array of structures for data records
 * Transpiled to safe Rust
 */

const MAXKEYS: usize = 10;

struct Key {
    word: &'static str,
    count: i32,
}

fn main() {
    let mut keytab: [Key; MAXKEYS] = [
        Key { word: "auto", count: 0 },
        Key { word: "break", count: 0 },
        Key { word: "case", count: 0 },
        Key { word: "char", count: 0 },
        Key { word: "const", count: 0 },
        Key { word: "continue", count: 0 },
        Key { word: "default", count: 0 },
        Key { word: "do", count: 0 },
        Key { word: "double", count: 0 },
        Key { word: "else", count: 0 },
    ];

    // Increment counts for demonstration
    keytab[0].count = 5;
    keytab[1].count = 3;
    keytab[4].count = 7;

    println!("Keyword counts:");
    for i in 0..MAXKEYS {
        if keytab[i].count > 0 {
            println!("{:<10} {}", keytab[i].word, keytab[i].count);
        }
    }
}
