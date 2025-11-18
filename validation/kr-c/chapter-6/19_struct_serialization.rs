/* K&R C Chapter 6: Structure Serialization
 * Writing/reading structures to/from files
 * Transpiled to safe Rust (using serde for serialization)
 */

use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
struct Account {
    id: i32,
    name: String,
    balance: f32,
}

impl Account {
    fn new(id: i32, name: &str, balance: f32) -> Self {
        Account {
            id,
            name: name.to_string(),
            balance,
        }
    }

    fn print(&self) {
        println!("  ID:{} {:<30} Balance:${:.2}", self.id, self.name, self.balance);
    }
}

// Binary serialization (similar to fwrite)
fn save_accounts_binary(filename: &str, accounts: &[Account]) -> io::Result<()> {
    let mut file = File::create(filename)?;

    // Write count
    file.write_all(&(accounts.len() as i32).to_le_bytes())?;

    // Write each account (simple binary format)
    for account in accounts {
        file.write_all(&account.id.to_le_bytes())?;

        // Write name length and bytes
        let name_bytes = account.name.as_bytes();
        file.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
        file.write_all(name_bytes)?;

        file.write_all(&account.balance.to_le_bytes())?;
    }

    Ok(())
}

fn load_accounts_binary(filename: &str) -> io::Result<Vec<Account>> {
    let mut file = File::open(filename)?;
    let mut accounts = Vec::new();

    // Read count
    let mut count_bytes = [0u8; 4];
    file.read_exact(&mut count_bytes)?;
    let count = i32::from_le_bytes(count_bytes);

    // Read each account
    for _ in 0..count {
        let mut id_bytes = [0u8; 4];
        file.read_exact(&mut id_bytes)?;
        let id = i32::from_le_bytes(id_bytes);

        // Read name
        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)?;
        let name_len = u32::from_le_bytes(len_bytes) as usize;

        let mut name_bytes = vec![0u8; name_len];
        file.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes).unwrap_or_default();

        let mut balance_bytes = [0u8; 4];
        file.read_exact(&mut balance_bytes)?;
        let balance = f32::from_le_bytes(balance_bytes);

        accounts.push(Account { id, name, balance });
    }

    Ok(accounts)
}

fn main() -> io::Result<()> {
    let filename = "accounts.dat";

    println!("=== Structure Serialization ===\n");

    // Create accounts
    let mut accounts = vec![
        Account::new(1001, "Alice Johnson", 5000.00),
        Account::new(1002, "Bob Smith", 3500.50),
        Account::new(1003, "Charlie Brown", 12000.75),
    ];

    // Save to file
    println!("Saving {} accounts to '{}'...", accounts.len(), filename);
    save_accounts_binary(filename, &accounts)?;
    println!("Saved successfully\n");

    // Modify in-memory data
    accounts[0].balance = 9999.99;
    accounts[1].balance = 8888.88;
    accounts[2].balance = 7777.77;

    println!("Modified in-memory data:");
    for account in &accounts {
        account.print();
    }
    println!();

    // Load from file
    let loaded = load_accounts_binary(filename)?;

    println!("Loaded {} accounts from '{}':", loaded.len(), filename);
    for account in &loaded {
        account.print();
    }

    Ok(())
}

// Idiomatic Rust: use serde for serialization
#[allow(dead_code)]
fn demonstrate_serde() {
    // In Cargo.toml: serde = { version = "1.0", features = ["derive"] }
    //                serde_json = "1.0"
    //                bincode = "1.3"

    // use serde::{Serialize, Deserialize};
    //
    // #[derive(Serialize, Deserialize)]
    // struct Account {
    //     id: i32,
    //     name: String,
    //     balance: f32,
    // }
    //
    // // JSON serialization
    // let json = serde_json::to_string(&account)?;
    // let account: Account = serde_json::from_str(&json)?;
    //
    // // Binary serialization (more compact)
    // let bytes = bincode::serialize(&account)?;
    // let account: Account = bincode::deserialize(&bytes)?;
}

// Key differences from C:
// 1. Result<T, E> for error handling
// 2. ? operator for error propagation
// 3. Read/Write traits for I/O
// 4. to_le_bytes/from_le_bytes for endianness
// 5. Vec<u8> for byte buffers
// 6. serde for production serialization
// 7. No fwrite/fread (use Write/Read traits)
// 8. Automatic file closing (RAII)
