/// Compound literal parsing tests
/// RED phase for DECY-060: Compound literal support
///
/// These tests validate parsing of C99 compound literals: (type){initializers}
/// Compound literals create temporary objects inline.
use decy_parser::{CParser, Expression};

#[test]
fn test_parse_simple_struct_compound_literal() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p = (struct Point){10, 20};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);

    // Just verify it parses without errors for now
    // Will add detailed checks once CompoundLiteral is implemented
}

#[test]
fn test_parse_struct_compound_literal_with_designated_init() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p = (struct Point){.x = 10, .y = 20};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_array_compound_literal() {
    let source = r#"
        int main() {
            int* arr = (int[]){1, 2, 3, 4, 5};
            return arr[0];
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_compound_literal_in_function_call() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        void draw(struct Point p) {}

        int main() {
            draw((struct Point){10, 20});
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 2);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_nested_compound_literal() {
    let source = r#"
        struct Inner {
            int value;
        };

        struct Outer {
            struct Inner inner;
        };

        int main() {
            struct Outer o = (struct Outer){
                .inner = (struct Inner){42}
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
fn test_parse_compound_literal_with_multiple_fields() {
    let source = r#"
        struct Color {
            int r;
            int g;
            int b;
            int a;
        };

        int main() {
            struct Color c = (struct Color){255, 128, 64, 255};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_compound_literal_return_value() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        struct Point get_origin() {
            return (struct Point){0, 0};
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_array_compound_literal_with_size() {
    let source = r#"
        int main() {
            int* arr = (int[3]){10, 20, 30};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_compound_literal_in_assignment() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p;
            p = (struct Point){100, 200};
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_compound_literal_pointer() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point* p = &(struct Point){10, 20};
            return p->x;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}
