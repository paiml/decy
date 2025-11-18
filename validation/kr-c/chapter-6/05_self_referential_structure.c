/* K&R C Chapter 6.5: Self-referential Structures
 * Page 139-140
 * Binary tree with self-referential structure
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct tnode {
    char *word;
    int count;
    struct tnode *left;
    struct tnode *right;
};

/* Simple tree node creation */
struct tnode *talloc(void) {
    return (struct tnode *) malloc(sizeof(struct tnode));
}

int main() {
    struct tnode *root;

    /* Create simple tree: root with two children */
    root = talloc();
    root->word = "hello";
    root->count = 1;

    root->left = talloc();
    root->left->word = "apple";
    root->left->count = 1;
    root->left->left = NULL;
    root->left->right = NULL;

    root->right = talloc();
    root->right->word = "world";
    root->right->count = 1;
    root->right->left = NULL;
    root->right->right = NULL;

    /* Print tree */
    printf("Root: %s (%d)\n", root->word, root->count);
    printf("Left: %s (%d)\n", root->left->word, root->left->count);
    printf("Right: %s (%d)\n", root->right->word, root->right->count);

    /* Memory cleanup */
    free(root->left);
    free(root->right);
    free(root);

    return 0;
}
