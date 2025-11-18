/* K&R C Chapter 5: LRU Cache
 * K&R §5.10: Least Recently Used cache with hash map + doubly-linked list
 * Tests cache eviction policy
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct lru_node {
    int key;
    int value;
    struct lru_node *prev;
    struct lru_node *next;
} LRUNode;

typedef struct {
    LRUNode *head;
    LRUNode *tail;
    LRUNode **hash_map;
    int capacity;
    int size;
} LRUCache;

/* Create cache */
LRUCache *lru_create(int capacity) {
    LRUCache *cache = malloc(sizeof(LRUCache));
    cache->head = malloc(sizeof(LRUNode));
    cache->tail = malloc(sizeof(LRUNode));
    cache->head->next = cache->tail;
    cache->tail->prev = cache->head;
    cache->hash_map = calloc(1000, sizeof(LRUNode*));
    cache->capacity = capacity;
    cache->size = 0;
    return cache;
}

/* Remove node from list */
void lru_remove_node(LRUNode *node) {
    node->prev->next = node->next;
    node->next->prev = node->prev;
}

/* Add node to front */
void lru_add_to_front(LRUCache *cache, LRUNode *node) {
    node->next = cache->head->next;
    node->prev = cache->head;
    cache->head->next->prev = node;
    cache->head->next = node;
}

/* Move to front */
void lru_move_to_front(LRUCache *cache, LRUNode *node) {
    lru_remove_node(node);
    lru_add_to_front(cache, node);
}

/* Get value */
int lru_get(LRUCache *cache, int key) {
    LRUNode *node = cache->hash_map[key];

    if (node == NULL) {
        return -1;
    }

    lru_move_to_front(cache, node);
    return node->value;
}

/* Put value */
void lru_put(LRUCache *cache, int key, int value) {
    LRUNode *node = cache->hash_map[key];

    if (node != NULL) {
        node->value = value;
        lru_move_to_front(cache, node);
        return;
    }

    LRUNode *new_node = malloc(sizeof(LRUNode));
    new_node->key = key;
    new_node->value = value;

    cache->hash_map[key] = new_node;
    lru_add_to_front(cache, new_node);
    cache->size++;

    if (cache->size > cache->capacity) {
        LRUNode *lru = cache->tail->prev;
        lru_remove_node(lru);
        cache->hash_map[lru->key] = NULL;
        printf("  Evicted key %d\n", lru->key);
        free(lru);
        cache->size--;
    }
}

/* Print cache */
void lru_print(LRUCache *cache) {
    printf("Cache (MRU -> LRU): [");
    LRUNode *node = cache->head->next;
    while (node != cache->tail) {
        printf("(%d:%d)", node->key, node->value);
        if (node->next != cache->tail) printf(", ");
        node = node->next;
    }
    printf("]\n");
}

int main() {
    printf("=== LRU Cache ===\n\n");

    LRUCache *cache = lru_create(3);

    printf("Put(1, 10):\n");
    lru_put(cache, 1, 10);
    lru_print(cache);

    printf("\nPut(2, 20):\n");
    lru_put(cache, 2, 20);
    lru_print(cache);

    printf("\nPut(3, 30):\n");
    lru_put(cache, 3, 30);
    lru_print(cache);

    printf("\nGet(1): %d\n", lru_get(cache, 1));
    lru_print(cache);

    printf("\nPut(4, 40) - should evict key 2:\n");
    lru_put(cache, 4, 40);
    lru_print(cache);

    printf("\nGet(2): %d (should be -1, evicted)\n", lru_get(cache, 2));

    return 0;
}
