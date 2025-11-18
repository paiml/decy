/* K&R C Chapter 5: Trie (Prefix Tree)
 * K&R §5.10: String search tree
 * Tests trie for efficient string operations
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#define ALPHABET_SIZE 26

typedef struct trie_node {
    struct trie_node *children[ALPHABET_SIZE];
    bool is_end_of_word;
} TrieNode;

/* Create node */
TrieNode *trie_node_create(void) {
    TrieNode *node = malloc(sizeof(TrieNode));
    node->is_end_of_word = false;
    for (int i = 0; i < ALPHABET_SIZE; i++) {
        node->children[i] = NULL;
    }
    return node;
}

/* Insert word */
void trie_insert(TrieNode *root, const char *word) {
    TrieNode *current = root;

    for (int i = 0; word[i] != '\0'; i++) {
        int index = word[i] - 'a';
        if (current->children[index] == NULL) {
            current->children[index] = trie_node_create();
        }
        current = current->children[index];
    }

    current->is_end_of_word = true;
}

/* Search word */
bool trie_search(TrieNode *root, const char *word) {
    TrieNode *current = root;

    for (int i = 0; word[i] != '\0'; i++) {
        int index = word[i] - 'a';
        if (current->children[index] == NULL) {
            return false;
        }
        current = current->children[index];
    }

    return current->is_end_of_word;
}

/* Check prefix */
bool trie_starts_with(TrieNode *root, const char *prefix) {
    TrieNode *current = root;

    for (int i = 0; prefix[i] != '\0'; i++) {
        int index = prefix[i] - 'a';
        if (current->children[index] == NULL) {
            return false;
        }
        current = current->children[index];
    }

    return true;
}

/* Print all words with prefix */
void trie_print_helper(TrieNode *node, char *prefix, int length) {
    if (node->is_end_of_word) {
        prefix[length] = '\0';
        printf("  %s\n", prefix);
    }

    for (int i = 0; i < ALPHABET_SIZE; i++) {
        if (node->children[i] != NULL) {
            prefix[length] = 'a' + i;
            trie_print_helper(node->children[i], prefix, length + 1);
        }
    }
}

void trie_print_all(TrieNode *root) {
    char prefix[100];
    trie_print_helper(root, prefix, 0);
}

int main() {
    printf("=== Trie (Prefix Tree) ===\n\n");

    TrieNode *root = trie_node_create();

    printf("Insert: cat, car, card, care, careful, dog, dodge, door\n");
    trie_insert(root, "cat");
    trie_insert(root, "car");
    trie_insert(root, "card");
    trie_insert(root, "care");
    trie_insert(root, "careful");
    trie_insert(root, "dog");
    trie_insert(root, "dodge");
    trie_insert(root, "door");

    printf("\nAll words:\n");
    trie_print_all(root);

    printf("\nSearch 'car': %s\n", trie_search(root, "car") ? "Found" : "Not found");
    printf("Search 'can': %s\n", trie_search(root, "can") ? "Found" : "Not found");

    printf("\nStarts with 'car': %s\n", trie_starts_with(root, "car") ? "Yes" : "No");
    printf("Starts with 'do': %s\n", trie_starts_with(root, "do") ? "Yes" : "No");

    return 0;
}
