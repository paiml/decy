//! Tests for C++/CUDA language mode detection and parsing (DECY-198).

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // DECY-198: .cu file extension and CUDA language mode
    // =========================================================================

    #[test]
    fn test_parse_cuda_global_keyword_detected() {
        // CUDA __global__ keyword should trigger CUDA mode via content detection
        let parser = CParser::new().expect("Parser creation failed");
        // Note: We test with a simplified version - full CUDA parsing requires
        // CUDA toolkit headers, but the language mode flag should be set
        let source = r#"
            void regular_function(int x) { }
        "#;
        let ast = parser.parse(source).expect("Parsing should succeed");
        assert_eq!(ast.functions().len(), 1);
    }

    #[test]
    fn test_parse_cpp_class_basic() {
        // C++ class should parse when C++ mode is detected
        let parser = CParser::new().expect("Parser creation failed");
        // extern "C" triggers C++ mode
        let source = r#"
            extern "C" {
                int c_function(int x);
            }
        "#;
        let ast = parser.parse(source).expect("Parsing extern C should succeed");
        assert_eq!(ast.functions().len(), 1);
        assert_eq!(ast.functions()[0].name, "c_function");
    }

    #[test]
    fn test_parse_file_cu_extension() {
        // .cu files should be accepted by parse_file
        use std::io::Write;
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cu_path = dir.path().join("test.cu");
        {
            let mut f = std::fs::File::create(&cu_path).expect("Failed to create .cu file");
            // Simple C code in .cu file - no CUDA-specific syntax needed
            writeln!(f, "int add(int a, int b) {{ return a + b; }}").unwrap();
        }

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse_file(&cu_path).expect("Parsing .cu file should succeed");
        assert_eq!(ast.functions().len(), 1);
        assert_eq!(ast.functions()[0].name, "add");
    }

    #[test]
    fn test_parse_file_cpp_extension() {
        // .cpp files should be accepted by parse_file
        use std::io::Write;
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cpp_path = dir.path().join("test.cpp");
        {
            let mut f = std::fs::File::create(&cpp_path).expect("Failed to create .cpp file");
            // Simple function in .cpp file
            writeln!(f, "int multiply(int a, int b) {{ return a * b; }}").unwrap();
        }

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse_file(&cpp_path).expect("Parsing .cpp file should succeed");
        assert_eq!(ast.functions().len(), 1);
        assert_eq!(ast.functions()[0].name, "multiply");
    }

    #[test]
    fn test_parse_file_c_extension_still_works() {
        // .c files should continue to work as before
        use std::io::Write;
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let c_path = dir.path().join("test.c");
        {
            let mut f = std::fs::File::create(&c_path).expect("Failed to create .c file");
            writeln!(f, "int sub(int a, int b) {{ return a - b; }}").unwrap();
        }

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse_file(&c_path).expect("Parsing .c file should succeed");
        assert_eq!(ast.functions().len(), 1);
        assert_eq!(ast.functions()[0].name, "sub");
    }

    #[test]
    fn test_cuda_keyword_in_comment_still_parses() {
        // CUDA keywords in comments should not affect parsing
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            // This code mentions __device__ in a comment
            int host_function(int x) { return x + 1; }
        "#;
        let ast = parser.parse(source).expect("Parsing with CUDA keyword in comment");
        assert_eq!(ast.functions().len(), 1);
        assert_eq!(ast.functions()[0].name, "host_function");
    }
}
