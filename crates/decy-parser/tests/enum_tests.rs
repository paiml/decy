/// Enum parsing tests
/// RED phase for DECY-062: Enum with explicit values
///
/// These tests validate parsing of C enum declarations with explicit discriminant values.

use decy_parser::CParser;

#[test]
fn test_parse_simple_enum() {
    let source = r#"
        enum Status {
            OK,
            ERROR,
            PENDING
        };

        int main() {
            enum Status s = OK;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    // Once enum parsing is added, we'll check result.enums().len()
}

#[test]
fn test_parse_enum_with_explicit_values() {
    let source = r#"
        enum Status {
            OK = 0,
            ERROR = 1,
            PENDING = 2
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    // Should capture explicit values: OK=0, ERROR=1, PENDING=2
}

#[test]
fn test_parse_enum_with_hex_values() {
    let source = r#"
        enum Flags {
            READ = 0x01,
            WRITE = 0x02,
            EXECUTE = 0x04
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    // Should capture hex values: READ=0x01, WRITE=0x02, EXECUTE=0x04
}

#[test]
fn test_parse_enum_bitmask_pattern() {
    let source = r#"
        enum FileMode {
            READ = 0x01,
            WRITE = 0x02,
            EXECUTE = 0x04,
            READ_WRITE = 0x03,
            ALL = 0x07
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_enum_mixed_explicit_implicit() {
    let source = r#"
        enum Mixed {
            FIRST = 10,
            SECOND,
            THIRD,
            FOURTH = 100,
            FIFTH
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    // SECOND should be 11, THIRD should be 12, FIFTH should be 101
}

#[test]
fn test_parse_enum_http_codes() {
    let source = r#"
        enum HTTP {
            OK = 200,
            CREATED = 201,
            BAD_REQUEST = 400,
            NOT_FOUND = 404,
            INTERNAL_ERROR = 500
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_enum_in_struct() {
    let source = r#"
        enum Status {
            OK = 0,
            ERROR = 1
        };

        struct Response {
            enum Status status;
            int code;
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_typedef_enum() {
    let source = r#"
        typedef enum {
            RED = 0,
            GREEN = 1,
            BLUE = 2
        } Color;

        int main() {
            Color c = RED;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    // Should have typedef Color
}

#[test]
fn test_parse_enum_negative_values() {
    let source = r#"
        enum Temperature {
            FREEZING = -10,
            COLD = 0,
            WARM = 20,
            HOT = 40
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_enum_large_values() {
    let source = r#"
        enum Large {
            BIG = 1000000,
            HUGE = 2000000
        };

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}
