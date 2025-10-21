//! Unit tests for struct type parameters in function signatures (DECY-037 RED phase)
//!
//! These tests verify that struct types in function parameters are correctly parsed:
//! - struct TypeName* param (pointer to struct)
//! - struct TypeName param (by value)
//! - Multiple struct parameters
//! - Mixed primitive and struct parameters
//!
//! References:
//! - K&R §6.2: Structures and Functions
//! - ISO C99 §6.7.2.1: Structure and union specifiers

use decy_parser::CParser;

#[test]
fn test_parse_real_world_linked_list_parameters() {
    // Real-world example from linked_list.c
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int data;
            struct Node* next;
        };

        int list_length(struct Node* head) {
            int count;
            count = 0;
            while (head != 0) {
                count = count + 1;
                head = head->next;
            }
            return count;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "list_length");

    println!("Function: {}", func.name);
    println!("Parameters: {}", func.parameters.len());
    for (i, param) in func.parameters.iter().enumerate() {
        println!(
            "  Param {}: {} (type: {:?})",
            i, param.name, param.param_type
        );
    }

    assert_eq!(
        func.parameters.len(),
        1,
        "Function should have 1 parameter (struct Node* head), found {}",
        func.parameters.len()
    );

    let param = &func.parameters[0];
    assert_eq!(param.name, "head", "Parameter name should be 'head'");

    match &param.param_type {
        decy_parser::parser::Type::Pointer(base_type) => match **base_type {
            decy_parser::parser::Type::Struct(ref name) => {
                assert_eq!(name, "Node", "Struct name should be 'Node', got '{}'", name);
            }
            _ => panic!("Expected Struct type, got {:?}", **base_type),
        },
        _ => panic!("Expected Pointer type, got {:?}", param.param_type),
    }
}

#[test]
fn test_parse_struct_pointer_parameter() {
    // Test: void traverse(struct Node* head)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        void traverse(struct Node* head) {
            // body not important for this test
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "traverse", "Function name should be 'traverse'");

    assert_eq!(
        func.parameters.len(),
        1,
        "Function should have 1 parameter, found {}",
        func.parameters.len()
    );

    let param = &func.parameters[0];
    assert_eq!(param.name, "head", "Parameter name should be 'head'");

    // Parameter type should be pointer to struct
    match &param.param_type {
        decy_parser::parser::Type::Pointer(base_type) => {
            assert!(
                matches!(**base_type, decy_parser::parser::Type::Struct(_)),
                "Base type should be struct, got {:?}",
                **base_type
            );
        }
        _ => panic!("Expected pointer type, got {:?}", param.param_type),
    }
}

#[test]
fn test_parse_struct_value_parameter() {
    // Test: void process(struct Point p)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        void process(struct Point p) {
            // body not important
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "process");

    assert_eq!(func.parameters.len(), 1, "Function should have 1 parameter");

    let param = &func.parameters[0];
    assert_eq!(param.name, "p", "Parameter name should be 'p'");

    // Parameter type should be struct (by value)
    assert!(
        matches!(param.param_type, decy_parser::parser::Type::Struct(_)),
        "Parameter type should be struct, got {:?}",
        param.param_type
    );
}

#[test]
fn test_parse_multiple_struct_parameters() {
    // Test: void connect(struct Node* a, struct Node* b)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
        };

        void connect(struct Node* a, struct Node* b) {
            // body not important
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "connect");

    assert_eq!(
        func.parameters.len(),
        2,
        "Function should have 2 parameters, found {}",
        func.parameters.len()
    );

    // First parameter
    assert_eq!(
        func.parameters[0].name, "a",
        "First parameter should be 'a'"
    );
    assert!(
        matches!(
            func.parameters[0].param_type,
            decy_parser::parser::Type::Pointer(_)
        ),
        "First parameter should be pointer, got {:?}",
        func.parameters[0].param_type
    );

    // Second parameter
    assert_eq!(
        func.parameters[1].name, "b",
        "Second parameter should be 'b'"
    );
    assert!(
        matches!(
            func.parameters[1].param_type,
            decy_parser::parser::Type::Pointer(_)
        ),
        "Second parameter should be pointer, got {:?}",
        func.parameters[1].param_type
    );
}

#[test]
fn test_parse_mixed_primitive_struct_parameters() {
    // Test: void update(int id, struct Node* node, float value)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int data;
        };

        void update(int id, struct Node* node, float value) {
            // body not important
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "update");

    assert_eq!(
        func.parameters.len(),
        3,
        "Function should have 3 parameters, found {}",
        func.parameters.len()
    );

    // First parameter: int id
    assert_eq!(func.parameters[0].name, "id");
    assert!(
        matches!(
            func.parameters[0].param_type,
            decy_parser::parser::Type::Int
        ),
        "First parameter should be int"
    );

    // Second parameter: struct Node* node
    assert_eq!(func.parameters[1].name, "node");
    assert!(
        matches!(
            func.parameters[1].param_type,
            decy_parser::parser::Type::Pointer(_)
        ),
        "Second parameter should be pointer to struct, got {:?}",
        func.parameters[1].param_type
    );

    // Third parameter: float value
    assert_eq!(func.parameters[2].name, "value");
    assert!(
        matches!(
            func.parameters[2].param_type,
            decy_parser::parser::Type::Float
        ),
        "Third parameter should be float"
    );
}

#[test]
fn test_parse_const_struct_pointer_parameter() {
    // Test: void display(const struct Node* node)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
        };

        void display(const struct Node* node) {
            // body not important
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "display");

    assert_eq!(func.parameters.len(), 1, "Function should have 1 parameter");

    let param = &func.parameters[0];
    assert_eq!(param.name, "node", "Parameter name should be 'node'");
    assert!(
        matches!(param.param_type, decy_parser::parser::Type::Pointer(_)),
        "Parameter should be pointer, got {:?}",
        param.param_type
    );
}
