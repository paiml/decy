/* K&R C Chapter 5: Pointer-Based Binary Search Tree
 * BST implementation with insert, search, traversal
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct tree_node {
    int value;
    struct tree_node *left;
    struct tree_node *right;
} TreeNode;

TreeNode *tree_create_node(int value) {
    TreeNode *node = (TreeNode*)malloc(sizeof(TreeNode));
    node->value = value;
    node->left = NULL;
    node->right = NULL;
    return node;
}

TreeNode *tree_insert(TreeNode *root, int value) {
    if (root == NULL)
        return tree_create_node(value);

    if (value < root->value)
        root->left = tree_insert(root->left, value);
    else if (value > root->value)
        root->right = tree_insert(root->right, value);

    return root;
}

TreeNode *tree_search(TreeNode *root, int value) {
    if (root == NULL || root->value == value)
        return root;

    if (value < root->value)
        return tree_search(root->left, value);
    else
        return tree_search(root->right, value);
}

void tree_inorder(TreeNode *root) {
    if (root == NULL)
        return;

    tree_inorder(root->left);
    printf("%d ", root->value);
    tree_inorder(root->right);
}

void tree_preorder(TreeNode *root) {
    if (root == NULL)
        return;

    printf("%d ", root->value);
    tree_preorder(root->left);
    tree_preorder(root->right);
}

void tree_postorder(TreeNode *root) {
    if (root == NULL)
        return;

    tree_postorder(root->left);
    tree_postorder(root->right);
    printf("%d ", root->value);
}

int tree_height(TreeNode *root) {
    if (root == NULL)
        return 0;

    int left_height = tree_height(root->left);
    int right_height = tree_height(root->right);

    return 1 + (left_height > right_height ? left_height : right_height);
}

int tree_count(TreeNode *root) {
    if (root == NULL)
        return 0;

    return 1 + tree_count(root->left) + tree_count(root->right);
}

void tree_free(TreeNode *root) {
    if (root == NULL)
        return;

    tree_free(root->left);
    tree_free(root->right);
    free(root);
}

int main() {
    TreeNode *root = NULL;

    printf("=== Binary Search Tree Demo ===\n\n");

    /* Build tree */
    int values[] = {50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 65};
    int n = sizeof(values) / sizeof(values[0]);

    printf("Inserting values: ");
    for (int i = 0; i < n; i++) {
        printf("%d ", values[i]);
        root = tree_insert(root, values[i]);
    }
    printf("\n");

    /* Tree statistics */
    printf("\nTree statistics:\n");
    printf("  Nodes: %d\n", tree_count(root));
    printf("  Height: %d\n", tree_height(root));

    /* Traversals */
    printf("\nTraversals:\n");
    printf("  Inorder:   ");
    tree_inorder(root);
    printf("\n");

    printf("  Preorder:  ");
    tree_preorder(root);
    printf("\n");

    printf("  Postorder: ");
    tree_postorder(root);
    printf("\n");

    /* Search */
    printf("\nSearching:\n");
    int search_vals[] = {35, 100};
    for (int i = 0; i < 2; i++) {
        TreeNode *found = tree_search(root, search_vals[i]);
        if (found)
            printf("  Found %d\n", search_vals[i]);
        else
            printf("  %d not found\n", search_vals[i]);
    }

    /* Cleanup */
    tree_free(root);
    printf("\nTree freed\n");

    return 0;
}
