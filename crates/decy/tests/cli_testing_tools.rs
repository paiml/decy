//! CLI Testing Tools
//!
//! Shared helpers for CLI contract testing.
//! Provides common utilities for creating test files, running commands, and assertions.
//!
//! **Pattern**: Following ruchy's proven CLI testing approach
//! **Reference**: CLAUDE.md CLI Contract Testing section

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create decy command
#[allow(deprecated)]
pub fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp file with content
pub fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

/// Helper: Create temp directory with files
pub fn create_temp_dir_with_files(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    for (name, content) in files {
        create_temp_file(&dir, name, content);
    }
    dir
}

/// Sample valid C code for testing
#[allow(dead_code)]
pub const VALID_C_CODE: &str = r#"
int main() {
    return 0;
}
"#;

/// Sample C code with syntax error
#[allow(dead_code)]
pub const INVALID_C_CODE: &str = r#"
int main( {
    return 0;
}
"#;

/// Sample C code with functions
#[allow(dead_code)]
pub const C_WITH_FUNCTION: &str = r#"
int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(2, 3);
    return result;
}
"#;

/// Sample C code with pointers
#[allow(dead_code)]
pub const C_WITH_POINTERS: &str = r#"
void increment(int* ptr) {
    (*ptr)++;
}

int main() {
    int x = 5;
    increment(&x);
    return x;
}
"#;

/// Sample C code with malloc/free
#[allow(dead_code)]
pub const C_WITH_MALLOC: &str = r#"
#include <stdlib.h>

int main() {
    int* ptr = (int*)malloc(sizeof(int));
    *ptr = 42;
    free(ptr);
    return 0;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decy_cmd_exists() {
        // Should be able to create command
        let _cmd = decy_cmd();
    }

    #[test]
    fn test_create_temp_file() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_file(&dir, "test.c", "int main() { return 0; }");

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("int main"));
    }

    #[test]
    fn test_create_temp_dir_with_files() {
        let files = vec![("file1.c", "int x = 1;"), ("file2.c", "int y = 2;")];
        let dir = create_temp_dir_with_files(&files);

        assert!(dir.path().join("file1.c").exists());
        assert!(dir.path().join("file2.c").exists());
    }
}
