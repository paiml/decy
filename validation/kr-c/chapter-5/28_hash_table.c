/* K&R C Chapter 5: Pointer-Based Hash Table
 * Simple hash table with chaining
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TABLE_SIZE 10

typedef struct entry {
    char *key;
    int value;
    struct entry *next;
} Entry;

typedef struct {
    Entry *buckets[TABLE_SIZE];
    int size;
} HashTable;

unsigned int hash(const char *key) {
    unsigned int hash = 0;
    while (*key) {
        hash = (hash * 31) + *key;
        key++;
    }
    return hash % TABLE_SIZE;
}

void ht_init(HashTable *ht) {
    for (int i = 0; i < TABLE_SIZE; i++)
        ht->buckets[i] = NULL;
    ht->size = 0;
}

void ht_insert(HashTable *ht, const char *key, int value) {
    unsigned int index = hash(key);
    Entry *entry = ht->buckets[index];

    /* Check if key exists */
    while (entry != NULL) {
        if (strcmp(entry->key, key) == 0) {
            entry->value = value;  /* Update */
            return;
        }
        entry = entry->next;
    }

    /* Create new entry */
    Entry *new_entry = (Entry*)malloc(sizeof(Entry));
    new_entry->key = strdup(key);
    new_entry->value = value;
    new_entry->next = ht->buckets[index];
    ht->buckets[index] = new_entry;
    ht->size++;
}

int ht_get(HashTable *ht, const char *key, int *value) {
    unsigned int index = hash(key);
    Entry *entry = ht->buckets[index];

    while (entry != NULL) {
        if (strcmp(entry->key, key) == 0) {
            *value = entry->value;
            return 1;  /* Found */
        }
        entry = entry->next;
    }

    return 0;  /* Not found */
}

void ht_print(HashTable *ht) {
    printf("Hash Table [%d entries]:\n", ht->size);
    for (int i = 0; i < TABLE_SIZE; i++) {
        if (ht->buckets[i] != NULL) {
            printf("  Bucket %d: ", i);
            Entry *entry = ht->buckets[i];
            while (entry != NULL) {
                printf("(%s: %d) ", entry->key, entry->value);
                entry = entry->next;
            }
            printf("\n");
        }
    }
}

void ht_free(HashTable *ht) {
    for (int i = 0; i < TABLE_SIZE; i++) {
        Entry *entry = ht->buckets[i];
        while (entry != NULL) {
            Entry *next = entry->next;
            free(entry->key);
            free(entry);
            entry = next;
        }
    }
    ht->size = 0;
}

int main() {
    HashTable ht;
    ht_init(&ht);

    printf("=== Hash Table Demo ===\n\n");

    /* Insert entries */
    printf("Inserting entries...\n");
    ht_insert(&ht, "apple", 100);
    ht_insert(&ht, "banana", 200);
    ht_insert(&ht, "cherry", 300);
    ht_insert(&ht, "date", 400);
    ht_insert(&ht, "elderberry", 500);
    ht_insert(&ht, "fig", 600);

    ht_print(&ht);

    /* Lookup entries */
    printf("\nLookup:\n");
    const char *keys[] = {"apple", "banana", "grape"};
    for (int i = 0; i < 3; i++) {
        int value;
        if (ht_get(&ht, keys[i], &value))
            printf("  %s = %d\n", keys[i], value);
        else
            printf("  %s not found\n", keys[i]);
    }

    /* Update entry */
    printf("\nUpdating 'apple' to 999...\n");
    ht_insert(&ht, "apple", 999);

    int value;
    if (ht_get(&ht, "apple", &value))
        printf("  apple = %d\n", value);

    /* Show collisions */
    printf("\nHash distribution:\n");
    for (int i = 0; i < TABLE_SIZE; i++) {
        int count = 0;
        Entry *entry = ht->buckets[i];
        while (entry != NULL) {
            count++;
            entry = entry->next;
        }
        if (count > 0)
            printf("  Bucket %d: %d entries\n", i, count);
    }

    /* Cleanup */
    ht_free(&ht);
    printf("\nHash table freed\n");

    return 0;
}
