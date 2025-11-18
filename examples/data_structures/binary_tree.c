// Binary search tree implementation
// Tests: recursive structures, malloc/free, pointer manipulation

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
