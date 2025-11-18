/* K&R C Chapter 7: Binary I/O Operations
 * K&R §7.5: fread and fwrite
 * Transpiled to safe Rust (using Read/Write traits with bincode or manual serialization)
 */

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

#[derive(Debug, Clone)]
struct Employee {
    id: i32,
    name: String,
    salary: f32,
    years: i32,
}

fn main() -> io::Result<()> {
    println!("=== Binary I/O Operations ===\n");

    let filename = "employees.dat";

    // Create sample data
    let employees = vec![
        Employee {
            id: 101,
            name: "Alice Johnson".to_string(),
            salary: 75000.0,
            years: 5,
        },
        Employee {
            id: 102,
            name: "Bob Smith".to_string(),
            salary: 82000.0,
            years: 8,
        },
        Employee {
            id: 103,
            name: "Carol Davis".to_string(),
            salary: 68000.0,
            years: 3,
        },
        Employee {
            id: 104,
            name: "David Wilson".to_string(),
            salary: 91000.0,
            years: 12,
        },
    ];

    // Write binary data
    println!("Writing {} employees to binary file...", employees.len());
    write_employees(filename, &employees)?;

    // Read binary data
    println!("Reading employees from binary file...");
    let read_employees = read_employees(filename)?;

    println!("\nEmployees read from file:");
    for (i, emp) in read_employees.iter().enumerate() {
        println!(
            "  [{}] {} - ID: {}, Salary: ${:.2}, Years: {}",
            i, emp.name, emp.id, emp.salary, emp.years
        );
    }

    // Append new employee
    println!("\nAppending new employee...");
    let new_emp = Employee {
        id: 105,
        name: "Eve Martinez".to_string(),
        salary: 77000.0,
        years: 6,
    };
    append_employee(filename, &new_emp)?;

    // Update specific employee
    println!("Updating employee at index 1...");
    let updated = Employee {
        id: 102,
        name: "Bob Smith".to_string(),
        salary: 85000.0,
        years: 9,
    };
    update_employee(filename, 1, &updated)?;

    // Read again to verify
    let final_employees = read_employees(filename)?;

    println!("\nEmployees after append and update:");
    for (i, emp) in final_employees.iter().enumerate() {
        println!(
            "  [{}] {} - ID: {}, Salary: ${:.2}, Years: {}",
            i, emp.name, emp.id, emp.salary, emp.years
        );
    }

    println!("\nBinary I/O advantages:");
    println!("  - Faster than text I/O");
    println!("  - Preserves exact data representation");
    println!("  - Random access via seek");
    println!("  - Fixed-size records for easy indexing");

    Ok(())
}

// Write employees to binary file
fn write_employees(filename: &str, employees: &[Employee]) -> io::Result<()> {
    let mut file = File::create(filename)?;

    for emp in employees {
        write_employee(&mut file, emp)?;
    }

    Ok(())
}

// Write single employee
fn write_employee<W: Write>(writer: &mut W, emp: &Employee) -> io::Result<()> {
    writer.write_all(&emp.id.to_le_bytes())?;

    // Write name length and name bytes
    let name_bytes = emp.name.as_bytes();
    writer.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
    writer.write_all(name_bytes)?;

    writer.write_all(&emp.salary.to_le_bytes())?;
    writer.write_all(&emp.years.to_le_bytes())?;

    Ok(())
}

// Read all employees from binary file
fn read_employees(filename: &str) -> io::Result<Vec<Employee>> {
    let mut file = File::open(filename)?;
    let mut employees = Vec::new();

    while let Ok(emp) = read_employee(&mut file) {
        employees.push(emp);
    }

    Ok(employees)
}

// Read single employee
fn read_employee<R: Read>(reader: &mut R) -> io::Result<Employee> {
    let mut id_bytes = [0u8; 4];
    reader.read_exact(&mut id_bytes)?;
    let id = i32::from_le_bytes(id_bytes);

    // Read name length and name
    let mut name_len_bytes = [0u8; 4];
    reader.read_exact(&mut name_len_bytes)?;
    let name_len = u32::from_le_bytes(name_len_bytes) as usize;

    let mut name_bytes = vec![0u8; name_len];
    reader.read_exact(&mut name_bytes)?;
    let name = String::from_utf8(name_bytes).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;

    let mut salary_bytes = [0u8; 4];
    reader.read_exact(&mut salary_bytes)?;
    let salary = f32::from_le_bytes(salary_bytes);

    let mut years_bytes = [0u8; 4];
    reader.read_exact(&mut years_bytes)?;
    let years = i32::from_le_bytes(years_bytes);

    Ok(Employee {
        id,
        name,
        salary,
        years,
    })
}

// Append employee to file
fn append_employee(filename: &str, emp: &Employee) -> io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(filename)?;

    write_employee(&mut file, emp)?;
    Ok(())
}

// Update specific employee by index
fn update_employee(filename: &str, index: usize, emp: &Employee) -> io::Result<()> {
    // Read all employees
    let mut employees = read_employees(filename)?;

    if index >= employees.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Index out of bounds",
        ));
    }

    // Update the employee
    employees[index] = emp.clone();

    // Write all employees back
    write_employees(filename, &employees)?;

    Ok(())
}

// Key differences from C:
// 1. Read/Write traits instead of fread/fwrite
// 2. read_exact() for reading exact bytes
// 3. to_le_bytes/from_le_bytes for endianness
// 4. Vec<u8> for variable-length data
// 5. Result<T, E> for error handling
// 6. Type-safe serialization
// 7. No sizeof operator needed
// 8. RAII: files auto-close
// 9. bincode crate for production (serde)
