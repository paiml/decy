/* K&R C Chapter 6: Nested Structures
 * Complex nested structure definitions
 * Transpiled to safe Rust (nested structs)
 */

#[derive(Debug, Clone)]
struct Date {
    day: i32,
    month: i32,
    year: i32,
}

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
    state: String,
    zip: i32,
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    birthdate: Date,
    home_address: Address,
    work_address: Address,
}

#[derive(Debug, Clone)]
struct Company {
    name: String,
    headquarters: Address,
    ceo: Person,
    employee_count: i32,
}

fn print_date(d: &Date) {
    print!("{:02}/{:02}/{:04}", d.month, d.day, d.year);
}

fn print_address(addr: &Address) {
    print!("{}, {}, {} {:05}", addr.street, addr.city, addr.state, addr.zip);
}

fn print_person(p: &Person) {
    println!("Name: {}", p.name);
    print!("  Birth date: ");
    print_date(&p.birthdate);
    println!();
    print!("  Home: ");
    print_address(&p.home_address);
    println!();
    print!("  Work: ");
    print_address(&p.work_address);
    println!();
}

fn main() {
    // Initialize nested structure
    let employee = Person {
        name: "Alice Johnson".to_string(),
        birthdate: Date {
            day: 15,
            month: 6,
            year: 1990,
        },
        home_address: Address {
            street: "123 Maple St".to_string(),
            city: "Springfield".to_string(),
            state: "IL".to_string(),
            zip: 62701,
        },
        work_address: Address {
            street: "456 Oak Ave".to_string(),
            city: "Springfield".to_string(),
            state: "IL".to_string(),
            zip: 62702,
        },
    };

    println!("=== Employee Information ===");
    print_person(&employee);

    // Deeply nested structure
    let mut tech_corp = Company {
        name: "TechCorp Inc.".to_string(),
        headquarters: Address {
            street: "789 Tech Blvd".to_string(),
            city: "San Jose".to_string(),
            state: "CA".to_string(),
            zip: 95110,
        },
        ceo: Person {
            name: "Bob Smith".to_string(),
            birthdate: Date {
                day: 20,
                month: 3,
                year: 1975,
            },
            home_address: Address {
                street: "999 Executive Dr".to_string(),
                city: "Palo Alto".to_string(),
                state: "CA".to_string(),
                zip: 94301,
            },
            work_address: Address {
                street: "789 Tech Blvd".to_string(),
                city: "San Jose".to_string(),
                state: "CA".to_string(),
                zip: 95110,
            },
        },
        employee_count: 5000,
    };

    println!("\n=== Company Information ===");
    println!("Company: {}", tech_corp.name);
    print!("Headquarters: ");
    print_address(&tech_corp.headquarters);
    println!();
    println!("Employees: {}", tech_corp.employee_count);
    println!("\nCEO Information:");
    print_person(&tech_corp.ceo);

    // Accessing deeply nested members
    println!("\n=== Accessing Nested Members ===");
    println!("CEO name: {}", tech_corp.ceo.name);
    println!("CEO birth year: {}", tech_corp.ceo.birthdate.year);
    println!("CEO home city: {}", tech_corp.ceo.home_address.city);

    // Modifying nested members
    tech_corp.ceo.birthdate.year = 1976;
    tech_corp.ceo.home_address.city = "Mountain View".to_string();

    println!("\nAfter modification:");
    println!("CEO birth year: {}", tech_corp.ceo.birthdate.year);
    println!("CEO home city: {}", tech_corp.ceo.home_address.city);
}

// Builder pattern for complex nested structures
#[allow(dead_code)]
fn demonstrate_builder_pattern() {
    struct PersonBuilder {
        name: String,
        birthdate: Option<Date>,
        home_address: Option<Address>,
        work_address: Option<Address>,
    }

    impl PersonBuilder {
        fn new(name: &str) -> Self {
            PersonBuilder {
                name: name.to_string(),
                birthdate: None,
                home_address: None,
                work_address: None,
            }
        }

        fn birthdate(mut self, date: Date) -> Self {
            self.birthdate = Some(date);
            self
        }

        fn home(mut self, addr: Address) -> Self {
            self.home_address = Some(addr);
            self
        }

        fn build(self) -> Result<Person, &'static str> {
            Ok(Person {
                name: self.name,
                birthdate: self.birthdate.ok_or("Missing birthdate")?,
                home_address: self.home_address.ok_or("Missing home address")?,
                work_address: self.work_address.unwrap_or_else(|| Address {
                    street: "".to_string(),
                    city: "".to_string(),
                    state: "".to_string(),
                    zip: 0,
                }),
            })
        }
    }
}

// Key differences from C:
// 1. String instead of char[]
// 2. Clone trait for nested cloning
// 3. References (&) to avoid copies
// 4. No manual strcpy needed
// 5. Builder pattern for complex initialization
// 6. Option for optional fields
// 7. Debug trait for easy printing
// 8. No null pointers in nested structures
