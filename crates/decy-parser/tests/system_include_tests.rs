//! Tests for system include path discovery (DECY-117).
//!
//! Verifies that C files using standard library headers parse correctly.

use decy_parser::parser::CParser;

// ============================================================================
// TEST 1: Parse C code with <stdlib.h>
// ============================================================================

#[test]
fn test_parse_with_stdlib_h() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <stdlib.h>

int main() {
    int* ptr = malloc(sizeof(int));
    free(ptr);
    return 0;
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse C code with <stdlib.h>: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    // stdlib.h brings in many function declarations, just verify main exists
    let has_main = ast.functions().iter().any(|f| f.name == "main");
    assert!(
        has_main,
        "Should find main function among {} functions",
        ast.functions().len()
    );
}

// ============================================================================
// TEST 2: Parse C code with <stdio.h>
// ============================================================================

#[test]
fn test_parse_with_stdio_h() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse C code with <stdio.h>: {:?}",
        result.err()
    );
}

// ============================================================================
// TEST 3: Parse C code with <string.h>
// ============================================================================

#[test]
fn test_parse_with_string_h() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <string.h>

int main() {
    char dest[20];
    strcpy(dest, "hello");
    return strlen(dest);
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse C code with <string.h>: {:?}",
        result.err()
    );
}

// ============================================================================
// TEST 4: Parse C code with multiple system headers
// ============================================================================

#[test]
fn test_parse_with_multiple_headers() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

int main() {
    char* str = malloc(100);
    strcpy(str, "test");
    printf("%s\n", str);
    free(str);
    return 0;
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse C code with multiple headers: {:?}",
        result.err()
    );
}

// ============================================================================
// TEST 5: Parse struct with stdlib types
// ============================================================================

#[test]
fn test_parse_struct_with_stdlib_types() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <stdlib.h>

struct Node {
    int value;
    struct Node* next;
};

struct Node* create_node(int value) {
    struct Node* node = malloc(sizeof(struct Node));
    node->value = value;
    node->next = NULL;
    return node;
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse struct with stdlib types: {:?}",
        result.err()
    );
}

// ============================================================================
// TEST 6: Parse with size_t from stddef.h
// ============================================================================

#[test]
fn test_parse_with_size_t() {
    let parser = CParser::new().expect("Failed to create parser");

    let source = r#"
#include <stddef.h>

size_t get_size(void) {
    return sizeof(int);
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse C code with size_t: {:?}",
        result.err()
    );
}

// ============================================================================
// TEST 7: Parse binary_tree.c example (QA audit acceptance criterion)
// ============================================================================

#[test]
fn test_parse_binary_tree_example() {
    let parser = CParser::new().expect("Failed to create parser");

    // This is the binary_tree.c file that QA audit identified as failing
    let source = r#"
#include <stdlib.h>
#include <stdio.h>

typedef struct TreeNode {
    int value;
    struct TreeNode* left;
    struct TreeNode* right;
} TreeNode;

TreeNode* create_node(int value) {
    TreeNode* node = (TreeNode*)malloc(sizeof(TreeNode));
    if (node != NULL) {
        node->value = value;
        node->left = NULL;
        node->right = NULL;
    }
    return node;
}

TreeNode* insert(TreeNode* root, int value) {
    if (root == NULL) {
        return create_node(value);
    }

    if (value < root->value) {
        root->left = insert(root->left, value);
    } else if (value > root->value) {
        root->right = insert(root->right, value);
    }

    return root;
}

int search(TreeNode* root, int value) {
    if (root == NULL) {
        return 0;
    }

    if (value == root->value) {
        return 1;
    } else if (value < root->value) {
        return search(root->left, value);
    } else {
        return search(root->right, value);
    }
}

void free_tree(TreeNode* root) {
    if (root == NULL) {
        return;
    }

    free_tree(root->left);
    free_tree(root->right);
    free(root);
}

int main(void) {
    TreeNode* root = NULL;

    root = insert(root, 50);
    root = insert(root, 30);
    root = insert(root, 70);
    root = insert(root, 20);
    root = insert(root, 40);

    int found = search(root, 40);
    printf("Found 40: %d\n", found);

    found = search(root, 100);
    printf("Found 100: %d\n", found);

    free_tree(root);
    return 0;
}
"#;

    let result = parser.parse(source);
    assert!(
        result.is_ok(),
        "Should parse binary_tree.c (QA acceptance criterion): {:?}",
        result.err()
    );

    let ast = result.unwrap();
    // Verify expected functions exist
    let function_names: Vec<&str> = ast.functions().iter().map(|f| f.name.as_str()).collect();
    assert!(
        function_names.contains(&"create_node"),
        "Should have create_node"
    );
    assert!(function_names.contains(&"insert"), "Should have insert");
    assert!(function_names.contains(&"search"), "Should have search");
    assert!(
        function_names.contains(&"free_tree"),
        "Should have free_tree"
    );
    assert!(function_names.contains(&"main"), "Should have main");
}
