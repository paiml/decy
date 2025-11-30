//! DECY-160: Tests for malloc(sizeof(struct)) to raw pointer transformation.
//!
//! When malloc is used with sizeof(struct T) and assigned to *mut T,
//! it should generate Box::into_raw(Box::<T>::default()), not Vec::<u8>.

use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a function that allocates a struct with malloc.
/// C equivalent:
/// ```c
/// struct Node { int data; struct Node* next; };
/// struct Node* create_node(int value) {
///     struct Node* node = malloc(sizeof(struct Node));
///     node->data = value;
///     node->next = 0;
///     return node;
/// }
/// ```
fn create_malloc_struct_function() -> HirFunction {
    let node_type = HirType::Struct("Node".to_string());
    let node_ptr_type = HirType::Pointer(Box::new(node_type.clone()));

    HirFunction::new_with_body(
        "create_node".to_string(),
        node_ptr_type.clone(),
        vec![HirParameter::new("value".to_string(), HirType::Int)],
        vec![
            // struct Node* node = malloc(sizeof(struct Node));
            HirStatement::VariableDeclaration {
                name: "node".to_string(),
                var_type: node_ptr_type.clone(),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::Sizeof {
                        type_name: "Node".to_string(),
                    }],
                }),
            },
            // node->data = value;
            HirStatement::Assignment {
                target: "(*node).data".to_string(),
                value: HirExpression::Variable("value".to_string()),
            },
            // node->next = 0;
            HirStatement::Assignment {
                target: "(*node).next".to_string(),
                value: HirExpression::IntLiteral(0),
            },
            // return node;
            HirStatement::Return(Some(HirExpression::Variable("node".to_string()))),
        ],
    )
}

#[test]
fn test_malloc_struct_generates_box_into_raw() {
    // DECY-160: malloc(sizeof(struct Node)) assigned to *mut Node
    // should generate Box::into_raw(Box::<Node>::default())
    let func = create_malloc_struct_function();

    let generator = decy_codegen::CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    println!("Generated code:\n{}", rust_code);

    // Should NOT contain Vec::<u8>::with_capacity - that's wrong for struct allocation
    assert!(
        !rust_code.contains("Vec::<u8>"),
        "Should not generate Vec::<u8> for struct malloc. Got:\n{}",
        rust_code
    );

    // Should contain Box::into_raw for proper struct allocation
    assert!(
        rust_code.contains("Box::into_raw") || rust_code.contains("Box::<Node>::default"),
        "Should use Box::into_raw for struct malloc. Got:\n{}",
        rust_code
    );
}

#[test]
fn test_malloc_struct_generates_correct_type() {
    // The generated code should be type-correct:
    // let mut node: *mut Node = Box::into_raw(Box::<Node>::default());
    let func = create_malloc_struct_function();

    let generator = decy_codegen::CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    println!("Generated code:\n{}", rust_code);

    // Should have consistent types: *mut Node on both sides
    assert!(
        rust_code.contains("*mut Node"),
        "Should declare node as *mut Node. Got:\n{}",
        rust_code
    );
}

#[test]
fn test_malloc_struct_in_assignment() {
    // Test case: malloc assigned via separate statement (not initialization)
    // struct Node* node;
    // node = malloc(sizeof(struct Node));
    let node_type = HirType::Struct("Node".to_string());
    let node_ptr_type = HirType::Pointer(Box::new(node_type.clone()));

    let func = HirFunction::new_with_body(
        "create_node2".to_string(),
        node_ptr_type.clone(),
        vec![],
        vec![
            // struct Node* node;
            HirStatement::VariableDeclaration {
                name: "node".to_string(),
                var_type: node_ptr_type.clone(),
                initializer: None,
            },
            // node = malloc(sizeof(struct Node));
            HirStatement::Assignment {
                target: "node".to_string(),
                value: HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::Sizeof {
                        type_name: "Node".to_string(),
                    }],
                },
            },
            HirStatement::Return(Some(HirExpression::Variable("node".to_string()))),
        ],
    );

    let generator = decy_codegen::CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    println!("Generated code:\n{}", rust_code);

    // Should NOT contain Vec::<u8>
    assert!(
        !rust_code.contains("Vec::<u8>"),
        "Assignment case should not use Vec::<u8>. Got:\n{}",
        rust_code
    );
}
