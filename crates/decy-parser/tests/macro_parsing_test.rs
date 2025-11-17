//! RED phase tests for parsing #define directives (DECY-098b)
//!
//! These tests verify that the parser can extract macro definitions from C code
//! and convert them to HIR representation.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3
//!
//! Status: GREEN phase - parser implementation in progress

use decy_parser::parser::CParser;

#[test]
fn test_parse_object_like_macro_simple() {
    // #define MAX 100
    let c_code = r#"
        #define MAX 100

        int main() {
            return MAX;
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    // Verify macro was parsed (currently returns 0 macros)
    assert_eq!(ast.macros().len(), 1, "Should parse 1 macro definition");

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "MAX");
    assert_eq!(macro_def.body(), "100");
    assert!(macro_def.is_object_like());
}

#[test]
#[ignore = "Parser limitation: Calls undeclared function printf. Need built-in function prototypes."]
fn test_parse_object_like_macro_string() {
    // #define GREETING "Hello, World!"
    let c_code = r#"
        #define GREETING "Hello, World!"

        void print_greeting() {
            printf(GREETING);
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "GREETING");
    assert_eq!(macro_def.body(), r#""Hello, World!""#);
    assert!(macro_def.is_object_like());
}

#[test]

fn test_parse_function_like_macro_single_param() {
    // #define SQR(x) ((x) * (x))
    let c_code = r#"
        #define SQR(x) ((x) * (x))

        int main() {
            int n = SQR(5);
            return n;
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "SQR");
    assert_eq!(macro_def.parameters(), &["x"]);
    // Clang tokenizes without spaces
    assert_eq!(macro_def.body(), "((x)*(x))");
    assert!(macro_def.is_function_like());
}

#[test]

fn test_parse_function_like_macro_multiple_params() {
    // #define MAX(a, b) ((a) > (b) ? (a) : (b))
    let c_code = r#"
        #define MAX(a, b) ((a) > (b) ? (a) : (b))

        int get_max(int x, int y) {
            return MAX(x, y);
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "MAX");
    assert_eq!(macro_def.parameters(), &["a", "b"]);
    // Clang tokenizes without spaces
    assert_eq!(macro_def.body(), "((a)>(b)?(a):(b))");
    assert!(macro_def.is_function_like());
}

#[test]

fn test_parse_multiple_macros() {
    let c_code = r#"
        #define PI 3.14159
        #define MAX(a, b) ((a) > (b) ? (a) : (b))
        #define DEBUG 1

        double area(double r) {
            return PI * r * r;
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 3, "Should parse 3 macro definitions");

    // Check first macro (PI)
    let pi_macro = &ast.macros()[0];
    assert_eq!(pi_macro.name(), "PI");
    assert_eq!(pi_macro.body(), "3.14159");
    assert!(pi_macro.is_object_like());

    // Check second macro (MAX)
    let max_macro = &ast.macros()[1];
    assert_eq!(max_macro.name(), "MAX");
    assert_eq!(max_macro.parameters(), &["a", "b"]);
    assert!(max_macro.is_function_like());

    // Check third macro (DEBUG)
    let debug_macro = &ast.macros()[2];
    assert_eq!(debug_macro.name(), "DEBUG");
    assert_eq!(debug_macro.body(), "1");
    assert!(debug_macro.is_object_like());
}

#[test]

fn test_parse_macro_with_no_body() {
    // #define EMPTY
    let c_code = r#"
        #define EMPTY

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "EMPTY");
    assert_eq!(macro_def.body(), "");
    assert!(macro_def.is_object_like());
}

#[test]

fn test_parse_macro_with_multiline_body() {
    // Multi-line macros use \ for line continuation
    let c_code = r#"
        #define SWAP(a, b) \
            { \
                typeof(a) tmp = a; \
                a = b; \
                b = tmp; \
            }

        void test() {
            int x = 1, y = 2;
            SWAP(x, y);
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "SWAP");
    assert_eq!(macro_def.parameters(), &["a", "b"]);
    // Body should have line continuations removed and be tokenized
    assert!(macro_def.body().contains("typeof"));
    assert!(macro_def.body().contains("tmp"));
    assert!(macro_def.is_function_like());
}

#[test]

fn test_parse_macro_and_function() {
    // Verify parsing both macros and functions together
    let c_code = r#"
        #define BUFFER_SIZE 1024

        int allocate_buffer() {
            return BUFFER_SIZE;
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1, "Should parse 1 macro");
    assert_eq!(ast.functions().len(), 1, "Should parse 1 function");

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "BUFFER_SIZE");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "allocate_buffer");
}

#[test]

fn test_parse_macro_with_parentheses_in_body() {
    // #define ABS(x) ((x) < 0 ? -(x) : (x))
    let c_code = r#"
        #define ABS(x) ((x) < 0 ? -(x) : (x))

        int abs_value(int n) {
            return ABS(n);
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 1);

    let macro_def = &ast.macros()[0];
    assert_eq!(macro_def.name(), "ABS");
    assert_eq!(macro_def.parameters(), &["x"]);
    // Clang tokenizes without spaces
    assert_eq!(macro_def.body(), "((x)<0?-(x):(x))");
}

#[test]

fn test_parse_macro_with_arithmetic() {
    // #define DOUBLE(x) ((x) * 2)
    let c_code = r#"
        #define DOUBLE(x) ((x) * 2)
        #define TRIPLE(x) ((x) * 3)

        int main() {
            return DOUBLE(5) + TRIPLE(3);
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    assert_eq!(ast.macros().len(), 2);

    let double_macro = &ast.macros()[0];
    assert_eq!(double_macro.name(), "DOUBLE");
    // Clang tokenizes without spaces
    assert_eq!(double_macro.body(), "((x)*2)");

    let triple_macro = &ast.macros()[1];
    assert_eq!(triple_macro.name(), "TRIPLE");
    // Clang tokenizes without spaces
    assert_eq!(triple_macro.body(), "((x)*3)");
}
