//! Interactive REPL for C-to-Rust transpilation.
//!
//! This module provides a Read-Eval-Print Loop for interactively transpiling
//! C code snippets to Rust. Users can enter C code and see the transpiled
//! Rust output immediately.
//!
//! # Features
//!
//! - Interactive line editing with history
//! - Instant transpilation feedback
//! - Built-in help and commands
//! - Clear error messages
//!
//! # Example Usage
//!
//! ```bash
//! $ decy repl
//! Decy REPL v0.1.0
//! C-to-Rust Transpiler - Interactive Mode
//! Type :help for commands, :quit to exit
//!
//! decy> int add(int a, int b) { return a + b; }
//! => Transpiled Rust code:
//! fn add(a: i32, b: i32) -> i32 {
//!     return a + b;
//! }
//! ```

#![warn(clippy::all)]

use anyhow::Result;

/// REPL command types
#[derive(Debug, PartialEq, Eq)]
pub enum ReplCommand {
    /// Quit the REPL
    Quit,
    /// Show help message
    Help,
    /// Clear the screen
    Clear,
    /// C code to transpile
    Code(String),
}

/// Parse a REPL input line into a command
pub fn parse_command(input: &str) -> ReplCommand {
    let trimmed = input.trim();

    match trimmed {
        ":quit" | ":q" | ":exit" => ReplCommand::Quit,
        ":help" | ":h" | ":?" => ReplCommand::Help,
        ":clear" | ":c" => ReplCommand::Clear,
        _ => ReplCommand::Code(input.to_string()),
    }
}

/// Run the interactive REPL
pub fn run() -> Result<()> {
    use rustyline::error::ReadlineError;
    use rustyline::{DefaultEditor, Result as RustyResult};

    println!("Decy REPL v0.1.0");
    println!("C-to-Rust Transpiler - Interactive Mode");
    println!("Type :help for commands, :quit to exit");
    println!();

    let rl: RustyResult<DefaultEditor> = DefaultEditor::new();
    if rl.is_err() {
        anyhow::bail!("Failed to initialize REPL");
    }
    let mut rl = rl.unwrap();

    loop {
        let readline = rl.readline("decy> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                let command = parse_command(&line);
                match command {
                    ReplCommand::Quit => {
                        println!("Goodbye!");
                        break;
                    }
                    ReplCommand::Help => {
                        print_help();
                    }
                    ReplCommand::Clear => {
                        // Clear screen
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    ReplCommand::Code(c_code) => match transpile_snippet(&c_code) {
                        Ok(rust_code) => {
                            println!("=> Transpiled Rust code:");
                            println!("{}", rust_code);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Decy REPL Commands:");
    println!("  :help, :h, :?     Show this help message");
    println!("  :quit, :q, :exit  Exit the REPL");
    println!("  :clear, :c        Clear the screen");
    println!();
    println!("Enter C code to transpile it to Rust.");
    println!("Example:");
    println!("  decy> int add(int a, int b) {{ return a + b; }}");
}

/// Transpile a C code snippet to Rust
///
/// This is a helper function for the REPL that handles snippet transpilation.
fn transpile_snippet(c_code: &str) -> Result<String> {
    decy_core::transpile(c_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // REPL COMMAND PARSING TESTS
    // ============================================================================

    #[test]
    fn test_parse_quit_command() {
        assert_eq!(parse_command(":quit"), ReplCommand::Quit);
        assert_eq!(parse_command(":q"), ReplCommand::Quit);
        assert_eq!(parse_command(":exit"), ReplCommand::Quit);
        assert_eq!(parse_command("  :quit  "), ReplCommand::Quit);
    }

    #[test]
    fn test_parse_help_command() {
        assert_eq!(parse_command(":help"), ReplCommand::Help);
        assert_eq!(parse_command(":h"), ReplCommand::Help);
        assert_eq!(parse_command(":?"), ReplCommand::Help);
    }

    #[test]
    fn test_parse_clear_command() {
        assert_eq!(parse_command(":clear"), ReplCommand::Clear);
        assert_eq!(parse_command(":c"), ReplCommand::Clear);
    }

    #[test]
    fn test_parse_c_code() {
        let code = "int main() { return 0; }";
        assert_eq!(parse_command(code), ReplCommand::Code(code.to_string()));
    }

    #[test]
    fn test_parse_multiline_c_code() {
        let code = "int add(int a, int b) {\n    return a + b;\n}";
        assert_eq!(parse_command(code), ReplCommand::Code(code.to_string()));
    }

    #[test]
    fn test_parse_empty_input() {
        assert_eq!(parse_command(""), ReplCommand::Code("".to_string()));
        assert_eq!(parse_command("  "), ReplCommand::Code("  ".to_string()));
    }

    #[test]
    fn test_parse_colon_prefix_not_command() {
        // Colon-prefixed strings that aren't valid commands should be Code
        assert_eq!(
            parse_command(":unknown"),
            ReplCommand::Code(":unknown".to_string())
        );
        assert_eq!(
            parse_command(":foo"),
            ReplCommand::Code(":foo".to_string())
        );
    }

    #[test]
    fn test_parse_whitespace_handling() {
        // Whitespace should be trimmed for command detection
        assert_eq!(parse_command("  :help  "), ReplCommand::Help);
        assert_eq!(parse_command("\t:quit\t"), ReplCommand::Quit);
        assert_eq!(parse_command("\n:clear\n"), ReplCommand::Clear);
    }

    // ============================================================================
    // REPL COMMAND ENUM TESTS
    // ============================================================================

    #[test]
    fn test_repl_command_debug() {
        let cmd = ReplCommand::Quit;
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Quit"));

        let cmd = ReplCommand::Code("test".into());
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Code"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_repl_command_eq() {
        assert_eq!(ReplCommand::Quit, ReplCommand::Quit);
        assert_eq!(ReplCommand::Help, ReplCommand::Help);
        assert_eq!(ReplCommand::Clear, ReplCommand::Clear);
        assert_eq!(
            ReplCommand::Code("test".into()),
            ReplCommand::Code("test".into())
        );

        assert_ne!(ReplCommand::Quit, ReplCommand::Help);
        assert_ne!(
            ReplCommand::Code("a".into()),
            ReplCommand::Code("b".into())
        );
    }

    // ============================================================================
    // TRANSPILE SNIPPET TESTS
    // ============================================================================

    #[test]
    fn test_transpile_simple_function() {
        let c_code = "int main() { return 0; }";
        let result = transpile_snippet(c_code);

        assert!(result.is_ok(), "Transpilation should succeed");
        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn main()"), "Should contain fn main()");
    }

    #[test]
    fn test_transpile_with_error() {
        let c_code = "int incomplete(";
        let result = transpile_snippet(c_code);

        assert!(result.is_err(), "Should return error for invalid C code");
    }

    #[test]
    fn test_transpile_expression_only() {
        let c_code = "x + y";
        let result = transpile_snippet(c_code);

        assert!(
            result.is_err(),
            "Expression-only code should fail (not yet supported)"
        );
    }

    #[test]
    fn test_transpile_void_function() {
        let c_code = "void hello() { }";
        let result = transpile_snippet(c_code);
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn hello()"));
    }

    #[test]
    fn test_transpile_function_with_params() {
        let c_code = "int add(int a, int b) { return a + b; }";
        let result = transpile_snippet(c_code);
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn add"));
        assert!(rust_code.contains("i32"));
    }

    #[test]
    fn test_transpile_empty_input() {
        let c_code = "";
        let result = transpile_snippet(c_code);
        // Empty input may succeed with empty output or fail
        // Either behavior is acceptable
        let _ = result;
    }

    #[test]
    fn test_transpile_whitespace_only() {
        let c_code = "   \n\t  ";
        let result = transpile_snippet(c_code);
        // Whitespace-only input may succeed with empty output or fail
        let _ = result;
    }

    // ============================================================================
    // PRINT_HELP TESTS
    // ============================================================================

    #[test]
    fn test_print_help_does_not_panic() {
        // print_help just prints to stdout, verify it doesn't panic
        print_help();
    }
}
