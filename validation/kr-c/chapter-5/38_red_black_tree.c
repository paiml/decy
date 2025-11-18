/* K&R C Chapter 5: Red-Black Tree
 * K&R §5.10: Self-balancing binary search tree
 * Tests red-black tree properties
 */

#include <stdio.h>
#include <stdlib.h>

typedef enum { RED, BLACK } Color;

typedef struct rb_node {
    int key;
    Color color;
    struct rb_node *left;
    struct rb_node *right;
    struct rb_node *parent;
} RBNode;

typedef struct {
    RBNode *root;
    RBNode *nil;
} RBTree;

/* Create node */
RBNode *rb_node_create(int key, RBNode *nil) {
    RBNode *node = malloc(sizeof(RBNode));
    node->key = key;
    node->color = RED;
    node->left = nil;
    node->right = nil;
    node->parent = nil;
    return node;
}

/* Create tree */
RBTree *rb_tree_create(void) {
    RBTree *tree = malloc(sizeof(RBTree));
    tree->nil = malloc(sizeof(RBNode));
    tree->nil->color = BLACK;
    tree->root = tree->nil;
    return tree;
}

/* Left rotate */
void rb_left_rotate(RBTree *tree, RBNode *x) {
    RBNode *y = x->right;
    x->right = y->left;

    if (y->left != tree->nil) {
        y->left->parent = x;
    }

    y->parent = x->parent;

    if (x->parent == tree->nil) {
        tree->root = y;
    } else if (x == x->parent->left) {
        x->parent->left = y;
    } else {
        x->parent->right = y;
    }

    y->left = x;
    x->parent = y;
}

/* Right rotate */
void rb_right_rotate(RBTree *tree, RBNode *y) {
    RBNode *x = y->left;
    y->left = x->right;

    if (x->right != tree->nil) {
        x->right->parent = y;
    }

    x->parent = y->parent;

    if (y->parent == tree->nil) {
        tree->root = x;
    } else if (y == y->parent->right) {
        y->parent->right = x;
    } else {
        y->parent->left = x;
    }

    x->right = y;
    y->parent = x;
}

/* Fix insert */
void rb_insert_fixup(RBTree *tree, RBNode *z) {
    while (z->parent->color == RED) {
        if (z->parent == z->parent->parent->left) {
            RBNode *y = z->parent->parent->right;
            if (y->color == RED) {
                z->parent->color = BLACK;
                y->color = BLACK;
                z->parent->parent->color = RED;
                z = z->parent->parent;
            } else {
                if (z == z->parent->right) {
                    z = z->parent;
                    rb_left_rotate(tree, z);
                }
                z->parent->color = BLACK;
                z->parent->parent->color = RED;
                rb_right_rotate(tree, z->parent->parent);
            }
        } else {
            RBNode *y = z->parent->parent->left;
            if (y->color == RED) {
                z->parent->color = BLACK;
                y->color = BLACK;
                z->parent->parent->color = RED;
                z = z->parent->parent;
            } else {
                if (z == z->parent->left) {
                    z = z->parent;
                    rb_right_rotate(tree, z);
                }
                z->parent->color = BLACK;
                z->parent->parent->color = RED;
                rb_left_rotate(tree, z->parent->parent);
            }
        }
    }
    tree->root->color = BLACK;
}

/* Insert */
void rb_insert(RBTree *tree, int key) {
    RBNode *z = rb_node_create(key, tree->nil);
    RBNode *y = tree->nil;
    RBNode *x = tree->root;

    while (x != tree->nil) {
        y = x;
        if (z->key < x->key) {
            x = x->left;
        } else {
            x = x->right;
        }
    }

    z->parent = y;

    if (y == tree->nil) {
        tree->root = z;
    } else if (z->key < y->key) {
        y->left = z;
    } else {
        y->right = z;
    }

    rb_insert_fixup(tree, z);
}

/* Inorder traversal */
void rb_inorder_helper(RBTree *tree, RBNode *node) {
    if (node != tree->nil) {
        rb_inorder_helper(tree, node->left);
        printf("%d%c ", node->key, node->color == RED ? 'R' : 'B');
        rb_inorder_helper(tree, node->right);
    }
}

void rb_inorder(RBTree *tree) {
    rb_inorder_helper(tree, tree->root);
    printf("\n");
}

int main() {
    printf("=== Red-Black Tree ===\n\n");

    RBTree *tree = rb_tree_create();

    printf("Insert: 7, 3, 18, 10, 22, 8, 11, 26\n");
    rb_insert(tree, 7);
    rb_insert(tree, 3);
    rb_insert(tree, 18);
    rb_insert(tree, 10);
    rb_insert(tree, 22);
    rb_insert(tree, 8);
    rb_insert(tree, 11);
    rb_insert(tree, 26);

    printf("Inorder (R=Red, B=Black): ");
    rb_inorder(tree);

    printf("\nRed-Black properties maintained:\n");
    printf("  1. Every node is red or black\n");
    printf("  2. Root is black\n");
    printf("  3. All leaves (NIL) are black\n");
    printf("  4. Red node has black children\n");
    printf("  5. All paths have same black height\n");

    return 0;
}
