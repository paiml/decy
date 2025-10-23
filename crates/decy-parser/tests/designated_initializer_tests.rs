/// Designated initializer parsing tests
/// RED phase for DECY-061: Designated initializer support
///
/// These tests validate parsing of C99 designated initializers: {.field = value} or {[index] = value}
/// Designated initializers enable explicit field/element initialization.

use decy_parser::CParser;

#[test]
fn test_parse_struct_designated_initializer() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p = {.x = 10, .y = 20};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);

    // Designated initializers should parse without errors
    // The exact representation will be checked once we enhance the AST
}

#[test]
fn test_parse_partial_designated_initializer() {
    let source = r#"
        struct Color {
            int r;
            int g;
            int b;
            int a;
        };

        int main() {
            struct Color c = {.r = 255};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_array_designated_initializer() {
    let source = r#"
        int main() {
            int arr[10] = {[0] = 1, [5] = 6, [9] = 10};
            return arr[0];
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_mixed_designated_positional() {
    let source = r#"
        struct Point {
            int x;
            int y;
            int z;
        };

        int main() {
            struct Point p = {10, .y = 20, .z = 30};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_nested_designated_initializer() {
    let source = r#"
        struct Inner {
            int value;
        };

        struct Outer {
            int x;
            struct Inner inner;
        };

        int main() {
            struct Outer o = {
                .x = 10,
                .inner = {.value = 42}
            };
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 2);
}

#[test]
fn test_parse_designated_initializer_out_of_order() {
    let source = r#"
        struct Point {
            int x;
            int y;
            int z;
        };

        int main() {
            struct Point p = {.z = 30, .x = 10, .y = 20};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_designated_in_compound_literal() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        void draw(struct Point p) {}

        int main() {
            draw((struct Point){.x = 10, .y = 20});
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 2);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_array_designated_with_size() {
    let source = r#"
        int main() {
            int arr[5] = {[1] = 10, [3] = 30};
            return arr[1];
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_designated_string_in_struct() {
    let source = r#"
        struct Config {
            int width;
            int height;
        };

        int main() {
            struct Config cfg = {.width = 800, .height = 600};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_global_with_designated_initializer() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        struct Point origin = {.x = 0, .y = 0};

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
    assert_eq!(result.variables().len(), 1);
}
