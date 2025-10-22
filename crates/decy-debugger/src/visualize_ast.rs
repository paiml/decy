//! C AST visualization for debugging
//!
//! Provides formatted visualization of C ASTs parsed by clang-sys.
//! Inspired by spydecy's Python AST visualizer.

use anyhow::{Context, Result};
use colored::Colorize;
use decy_parser::{CParser, Function, Statement, Expression};
use std::fs;
use std::path::Path;

/// Visualize C source as AST
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed
pub fn visualize_c_ast(file_path: &Path, use_colors: bool) -> Result<String> {
    // Read the source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Parse to AST
    let parser = CParser::new();
    let functions = parser
        .parse(&source)
        .context("Failed to parse C source")?;

    // Format the output
    let mut output = String::new();

    // Header
    if use_colors {
        output.push_str(&format!(
            "{}",
            "╔══════════════════════════════════════════════════════════╗\n".cyan()
        ));
        output.push_str(&format!(
            "{}",
            "║  Decy Debugger: C AST Visualization                     ║\n".cyan()
        ));
        output.push_str(&format!(
            "{}",
            "╚══════════════════════════════════════════════════════════╝\n".cyan()
        ));
    } else {
        output.push_str("╔══════════════════════════════════════════════════════════╗\n");
        output.push_str("║  Decy Debugger: C AST Visualization                     ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    }
    output.push('\n');

    // File info
    let file_label = if use_colors { "File:".bold().to_string() } else { "File:".to_string() };
    output.push_str(&format!("{} {}\n", file_label, file_path.display()));

    let size_label = if use_colors { "Size:".bold().to_string() } else { "Size:".to_string() };
    output.push_str(&format!(
        "{} {} lines\n",
        size_label,
        source.lines().count()
    ));
    output.push('\n');

    // Source code preview
    let source_header = if use_colors {
        "═══ Source Code ═══".yellow().bold().to_string()
    } else {
        "═══ Source Code ═══".to_string()
    };
    output.push_str(&format!("{}\n", source_header));

    for (i, line) in source.lines().enumerate() {
        if use_colors {
            output.push_str(&format!("{:3} │ {}\n", (i + 1).to_string().dimmed(), line));
        } else {
            output.push_str(&format!("{:3} │ {}\n", i + 1, line));
        }
    }
    output.push('\n');

    // AST tree
    let ast_header = if use_colors {
        "═══ Abstract Syntax Tree ═══".green().bold().to_string()
    } else {
        "═══ Abstract Syntax Tree ═══".to_string()
    };
    output.push_str(&format!("{}\n", ast_header));

    for function in &functions {
        format_function(function, 0, &mut output, use_colors);
    }
    output.push('\n');

    // Statistics
    let stats_header = if use_colors {
        "═══ Statistics ═══".blue().bold().to_string()
    } else {
        "═══ Statistics ═══".to_string()
    };
    output.push_str(&format!("{}\n", stats_header));

    let func_count_label = if use_colors { "Functions:".bold().to_string() } else { "Functions:".to_string() };
    output.push_str(&format!("  {} {}\n", func_count_label, functions.len()));

    let total_statements: usize = functions.iter().map(|f| f.body.len()).sum();
    let stmt_count_label = if use_colors { "Total statements:".bold().to_string() } else { "Total statements:".to_string() };
    output.push_str(&format!("  {} {}\n", stmt_count_label, total_statements));

    Ok(output)
}

/// Format a function node with indentation
fn format_function(function: &Function, depth: usize, output: &mut String, use_colors: bool) {
    let indent = "  ".repeat(depth);

    let node_type = if use_colors {
        format!("Function: {}", function.name).green().bold().to_string()
    } else {
        format!("Function: {}", function.name)
    };

    output.push_str(&format!("{}├─ {}", indent, node_type));

    // Return type
    if use_colors {
        output.push_str(&format!(" → {}", function.return_type.dimmed()));
    } else {
        output.push_str(&format!(" → {}", function.return_type));
    }
    output.push('\n');

    // Parameters
    if !function.parameters.is_empty() {
        let params_label = if use_colors { "Parameters:".blue().to_string() } else { "Parameters:".to_string() };
        output.push_str(&format!("{}  {} ", indent, params_label));

        for (i, param) in function.parameters.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: {}", param.name, param.param_type));
        }
        output.push('\n');
    }

    // Body statements
    if !function.body.is_empty() {
        let body_label = if use_colors { "Body:".cyan().to_string() } else { "Body:".to_string() };
        output.push_str(&format!("{}  {}\n", indent, body_label));

        for stmt in &function.body {
            format_statement(stmt, depth + 2, output, use_colors);
        }
    }
}

/// Format a statement node
fn format_statement(stmt: &Statement, depth: usize, output: &mut String, use_colors: bool) {
    let indent = "  ".repeat(depth);

    let stmt_str = match stmt {
        Statement::Return(Some(expr)) => {
            let label = if use_colors { "Return".red().to_string() } else { "Return".to_string() };
            format!("{}: {}", label, format_expression(expr, use_colors))
        }
        Statement::Return(None) => {
            if use_colors { "Return (void)".red().to_string() } else { "Return (void)".to_string() }
        }
        Statement::Assignment { target, value } => {
            let label = if use_colors { "Assignment".yellow().to_string() } else { "Assignment".to_string() };
            format!("{}: {} = {}", label, target, format_expression(value, use_colors))
        }
        Statement::If { condition, then_block, else_block } => {
            let mut s = if use_colors {
                format!("{}: {}", "If".magenta(), format_expression(condition, use_colors))
            } else {
                format!("If: {}", format_expression(condition, use_colors))
            };

            s.push_str(&format!(" (then: {} stmts", then_block.len()));
            if let Some(else_b) = else_block {
                s.push_str(&format!(", else: {} stmts)", else_b.len()));
            } else {
                s.push(')');
            }
            s
        }
        Statement::While { condition, body } => {
            let label = if use_colors { "While".magenta().to_string() } else { "While".to_string() };
            format!("{}: {} ({} stmts)", label, format_expression(condition, use_colors), body.len())
        }
        Statement::For { init, condition, increment, body } => {
            let label = if use_colors { "For".magenta().to_string() } else { "For".to_string() };
            format!("{}: init={:?}, cond={:?}, inc={:?} ({} stmts)",
                label, init, condition, increment, body.len())
        }
        Statement::VariableDeclaration { name, var_type, initializer } => {
            let label = if use_colors { "VarDecl".cyan().to_string() } else { "VarDecl".to_string() };
            let mut s = format!("{}: {} {}: {}", label, var_type, name, name);
            if let Some(init) = initializer {
                s.push_str(&format!(" = {}", format_expression(init, use_colors)));
            }
            s
        }
        _ => format!("{:?}", stmt),
    };

    output.push_str(&format!("{}├─ {}\n", indent, stmt_str));
}

/// Format an expression for display
fn format_expression(expr: &Expression, use_colors: bool) -> String {
    match expr {
        Expression::IntLiteral(n) => {
            if use_colors {
                n.to_string().bright_yellow().to_string()
            } else {
                n.to_string()
            }
        }
        Expression::Variable(name) => {
            if use_colors {
                name.blue().to_string()
            } else {
                name.clone()
            }
        }
        Expression::BinaryOp { left, op, right } => {
            format!(
                "({} {} {})",
                format_expression(left, use_colors),
                op,
                format_expression(right, use_colors)
            )
        }
        Expression::FunctionCall { function, arguments } => {
            let args_str = arguments
                .iter()
                .map(|arg| format_expression(arg, use_colors))
                .collect::<Vec<_>>()
                .join(", ");

            if use_colors {
                format!("{}({})", function.magenta(), args_str)
            } else {
                format!("{}({})", function, args_str)
            }
        }
        _ => format!("{:?}", expr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_visualize_simple_function() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int add(int a, int b) {{ return a + b; }}").unwrap();

        let result = visualize_c_ast(temp_file.path(), false);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Function: add"));
        assert!(output.contains("Return"));
    }

    #[test]
    fn test_visualize_with_colors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int main() {{ return 0; }}").unwrap();

        let result = visualize_c_ast(temp_file.path(), true);
        assert!(result.is_ok());

        // Just verify it produces output (color codes make exact matching hard)
        let output = result.unwrap();
        assert!(!output.is_empty());
        assert!(output.contains("main"));
    }
}
