/* K&R C Chapter 5: Skip List
 * K&R §5.10: Probabilistic data structure
 * Tests skip list for efficient search
 */

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define MAX_LEVEL 4

typedef struct node {
    int key;
    int value;
    struct node **forward;
} SkipNode;

typedef struct {
    int level;
    SkipNode *header;
} SkipList;

/* Create node */
SkipNode *skip_node_create(int key, int value, int level) {
    SkipNode *node = malloc(sizeof(SkipNode));
    node->key = key;
    node->value = value;
    node->forward = malloc(sizeof(SkipNode*) * (level + 1));
    for (int i = 0; i <= level; i++) {
        node->forward[i] = NULL;
    }
    return node;
}

/* Create skip list */
SkipList *skip_list_create(void) {
    SkipList *list = malloc(sizeof(SkipList));
    list->level = 0;
    list->header = skip_node_create(-1, 0, MAX_LEVEL);
    return list;
}

/* Random level */
int random_level(void) {
    int level = 0;
    while (rand() % 2 && level < MAX_LEVEL) {
        level++;
    }
    return level;
}

/* Insert */
void skip_list_insert(SkipList *list, int key, int value) {
    SkipNode *update[MAX_LEVEL + 1];
    SkipNode *current = list->header;

    for (int i = list->level; i >= 0; i--) {
        while (current->forward[i] && current->forward[i]->key < key) {
            current = current->forward[i];
        }
        update[i] = current;
    }

    int level = random_level();
    if (level > list->level) {
        for (int i = list->level + 1; i <= level; i++) {
            update[i] = list->header;
        }
        list->level = level;
    }

    SkipNode *node = skip_node_create(key, value, level);
    for (int i = 0; i <= level; i++) {
        node->forward[i] = update[i]->forward[i];
        update[i]->forward[i] = node;
    }
}

/* Search */
int skip_list_search(SkipList *list, int key, int *value) {
    SkipNode *current = list->header;

    for (int i = list->level; i >= 0; i--) {
        while (current->forward[i] && current->forward[i]->key < key) {
            current = current->forward[i];
        }
    }

    current = current->forward[0];
    if (current && current->key == key) {
        *value = current->value;
        return 1;
    }
    return 0;
}

/* Print */
void skip_list_print(SkipList *list) {
    printf("Skip List:\n");
    for (int i = list->level; i >= 0; i--) {
        printf("Level %d: ", i);
        SkipNode *node = list->header->forward[i];
        while (node) {
            printf("%d->", node->key);
            node = node->forward[i];
        }
        printf("NULL\n");
    }
}

int main() {
    printf("=== Skip List ===\n\n");

    srand(time(NULL));
    SkipList *list = skip_list_create();

    printf("Insert: 3, 6, 7, 9, 12, 19, 17\n");
    skip_list_insert(list, 3, 30);
    skip_list_insert(list, 6, 60);
    skip_list_insert(list, 7, 70);
    skip_list_insert(list, 9, 90);
    skip_list_insert(list, 12, 120);
    skip_list_insert(list, 19, 190);
    skip_list_insert(list, 17, 170);

    skip_list_print(list);

    int value;
    printf("\nSearch 7: %s (value=%d)\n",
           skip_list_search(list, 7, &value) ? "Found" : "Not found", value);
    printf("Search 15: %s\n",
           skip_list_search(list, 15, &value) ? "Found" : "Not found");

    return 0;
}
