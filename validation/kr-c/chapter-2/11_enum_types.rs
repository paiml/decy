/* K&R C Chapter 2.3: Constants - Enumeration
 * Page 39-40
 * Enumeration constants
 * Transpiled to safe Rust
 */

// Rust enums are more powerful than C enums
// They can carry data and must be matched exhaustively

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
enum Day {
    Monday = 1,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
enum Month {
    Jan = 1, Feb, Mar, Apr, May, Jun,
    Jul, Aug, Sep, Oct, Nov, Dec,
}

fn day_name(d: Day) -> &'static str {
    match d {
        Day::Monday => "Monday",
        Day::Tuesday => "Tuesday",
        Day::Wednesday => "Wednesday",
        Day::Thursday => "Thursday",
        Day::Friday => "Friday",
        Day::Saturday => "Saturday",
        Day::Sunday => "Sunday",
    }
}

fn main() {
    let today = Day::Friday;
    let current_month = Month::Nov;

    println!("Today is: {} ({:?})", day_name(today), today as i32);
    println!("Current month: {}", current_month as i32);

    let is_weekend = matches!(today, Day::Saturday | Day::Sunday);
    println!("Is weekend: {}", if is_weekend { "Yes" } else { "No" });

    // Enum arithmetic (less idiomatic in Rust, but possible)
    let tomorrow = unsafe { std::mem::transmute::<i32, Day>(today as i32 + 1) };
    println!("Tomorrow is: {} ({})", day_name(tomorrow), tomorrow as i32);

    // Loop through days (using iterator)
    println!("\nDays of week:");
    for d in [Day::Monday, Day::Tuesday, Day::Wednesday, Day::Thursday,
              Day::Friday, Day::Saturday, Day::Sunday] {
        println!("  {}: {}", d as i32, day_name(d));
    }
}

// Safer Rust alternative using strum for enum iteration
mod safer_approach {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Day {
        Monday,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
        Sunday,
    }

    impl Day {
        pub fn name(&self) -> &'static str {
            match self {
                Day::Monday => "Monday",
                Day::Tuesday => "Tuesday",
                Day::Wednesday => "Wednesday",
                Day::Thursday => "Thursday",
                Day::Friday => "Friday",
                Day::Saturday => "Saturday",
                Day::Sunday => "Sunday",
            }
        }

        pub fn all() -> Vec<Day> {
            vec![
                Day::Monday, Day::Tuesday, Day::Wednesday,
                Day::Thursday, Day::Friday, Day::Saturday, Day::Sunday,
            ]
        }
    }
}

// Key differences from C:
// 1. Enums are namespaced: Day::Monday not just MONDAY
// 2. Match must be exhaustive (compiler enforced)
// 3. No implicit integer conversion (must use 'as')
// 4. Can add methods to enums
// 5. Can carry data: enum Option<T> { Some(T), None }
