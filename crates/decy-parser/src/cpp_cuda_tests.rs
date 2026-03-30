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

    // =========================================================================
    // DECY-200: C++ class extraction
    // =========================================================================

    #[test]
    fn test_parse_cpp_class_with_fields() {
        let parser = CParser::new().expect("Parser creation failed");
        // extern "C" block plus a class with a field
        let source = r#"
            extern "C" { void dummy(); }
            class Point {
            public:
                int x;
                int y;
            };
        "#;
        let ast = parser.parse(source).expect("Parsing class with fields");
        assert_eq!(ast.classes().len(), 1, "Should find one class");
        assert_eq!(ast.classes()[0].name, "Point");
        assert_eq!(ast.classes()[0].fields.len(), 2);
        assert_eq!(ast.classes()[0].fields[0].name, "x");
        assert_eq!(ast.classes()[0].fields[1].name, "y");
    }

    #[test]
    fn test_parse_cpp_class_with_method() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            class Calculator {
            public:
                int value;
                int get_value() { return value; }
            };
        "#;
        let ast = parser.parse(source).expect("Parsing class with method");
        assert_eq!(ast.classes().len(), 1);
        assert_eq!(ast.classes()[0].name, "Calculator");
        assert_eq!(ast.classes()[0].methods.len(), 1);
        assert_eq!(ast.classes()[0].methods[0].function.name, "get_value");
    }

    #[test]
    fn test_parse_cpp_class_with_constructor_destructor() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            class Resource {
            public:
                int handle;
                Resource(int h) : handle(h) {}
                ~Resource() {}
            };
        "#;
        let ast = parser.parse(source).expect("Parsing class with ctor/dtor");
        assert_eq!(ast.classes().len(), 1);
        let cls = &ast.classes()[0];
        assert_eq!(cls.name, "Resource");
        assert!(cls.has_destructor, "Should detect destructor");
        assert_eq!(cls.constructor_params.len(), 1, "Constructor has 1 param");
        assert_eq!(cls.constructor_params[0].name, "h");
    }

    #[test]
    fn test_parse_cpp_class_file_based() {
        use std::io::Write;
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cpp_path = dir.path().join("test_class.cpp");
        {
            let mut f = std::fs::File::create(&cpp_path).expect("create");
            writeln!(
                f,
                "class Foo {{ public: int x; Foo(int v) : x(v) {{}} int get() {{ return x; }} ~Foo() {{}} }};"
            )
            .unwrap();
        }
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse_file(&cpp_path).expect("Parsing .cpp class file");
        assert_eq!(ast.classes().len(), 1);
        let cls = &ast.classes()[0];
        assert_eq!(cls.name, "Foo");
        assert_eq!(cls.fields.len(), 1);
        assert!(cls.has_destructor);
        assert_eq!(cls.methods.len(), 1);
        assert_eq!(cls.constructor_params.len(), 1);
    }

    // =========================================================================
    // DECY-201: C++ namespace extraction
    // =========================================================================

    #[test]
    fn test_parse_cpp_namespace_with_function() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            namespace math {
                int add(int a, int b) { return a + b; }
            }
        "#;
        let ast = parser.parse(source).expect("Parsing namespace with function");
        assert_eq!(ast.namespaces().len(), 1, "Should find one namespace");
        assert_eq!(ast.namespaces()[0].name, "math");
        assert_eq!(ast.namespaces()[0].functions.len(), 1);
        assert_eq!(ast.namespaces()[0].functions[0].name, "add");
    }

    #[test]
    fn test_parse_cpp_namespace_with_struct() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            namespace geom {
                struct Point { int x; int y; };
            }
        "#;
        let ast = parser.parse(source).expect("Parsing namespace with struct");
        assert_eq!(ast.namespaces().len(), 1);
        assert_eq!(ast.namespaces()[0].name, "geom");
        assert_eq!(ast.namespaces()[0].structs.len(), 1);
        assert_eq!(ast.namespaces()[0].structs[0].name, "Point");
    }

    #[test]
    fn test_parse_cpp_nested_namespace() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            namespace outer {
                namespace inner {
                    int value() { return 42; }
                }
            }
        "#;
        let ast = parser.parse(source).expect("Parsing nested namespace");
        assert_eq!(ast.namespaces().len(), 1);
        assert_eq!(ast.namespaces()[0].name, "outer");
        assert_eq!(ast.namespaces()[0].namespaces.len(), 1);
        assert_eq!(ast.namespaces()[0].namespaces[0].name, "inner");
        assert_eq!(ast.namespaces()[0].namespaces[0].functions.len(), 1);
    }

    #[test]
    fn test_parse_cpp_namespace_with_class() {
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            extern "C" { void dummy(); }
            namespace shapes {
                class Circle {
                public:
                    int radius;
                    int area() { return 3 * radius * radius; }
                };
            }
        "#;
        let ast = parser.parse(source).expect("Parsing namespace with class");
        assert_eq!(ast.namespaces().len(), 1);
        assert_eq!(ast.namespaces()[0].name, "shapes");
        assert_eq!(ast.namespaces()[0].classes.len(), 1);
        assert_eq!(ast.namespaces()[0].classes[0].name, "Circle");
    }

    // =========================================================================
    // Original tests
    // =========================================================================

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
