// Simple hash table with chaining
// Tests: arrays of pointers, linked lists, string hashing

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#define TABLE_SIZE 100

typedef struct Entry {
    char* key;
    int value;
    struct Entry* next;
} Entry;

typedef struct HashTable {
    Entry* buckets[TABLE_SIZE];
} HashTable;

unsigned int hash(const char* key) {
    unsigned int hash = 0;
    while (*key) {
        hash = (hash << 5) + *key++;
    }
    return hash % TABLE_SIZE;
}

HashTable* create_table(void) {
    HashTable* table = (HashTable*)malloc(sizeof(HashTable));
    if (table != NULL) {
        for (int i = 0; i < TABLE_SIZE; i++) {
            table->buckets[i] = NULL;
        }
    }
    return table;
}

void insert(HashTable* table, const char* key, int value) {
    unsigned int index = hash(key);
    Entry* entry = (Entry*)malloc(sizeof(Entry));

    entry->key = (char*)malloc(strlen(key) + 1);
    strcpy(entry->key, key);
    entry->value = value;
    entry->next = table->buckets[index];

    table->buckets[index] = entry;
}

int get(HashTable* table, const char* key, int* value) {
    unsigned int index = hash(key);
    Entry* entry = table->buckets[index];

    while (entry != NULL) {
        if (strcmp(entry->key, key) == 0) {
            *value = entry->value;
            return 1;
        }
        entry = entry->next;
    }

    return 0;
}

void free_table(HashTable* table) {
    for (int i = 0; i < TABLE_SIZE; i++) {
        Entry* entry = table->buckets[i];
        while (entry != NULL) {
            Entry* next = entry->next;
            free(entry->key);
            free(entry);
            entry = next;
        }
    }
    free(table);
}

int main(void) {
    HashTable* table = create_table();

    insert(table, "apple", 5);
    insert(table, "banana", 7);
    insert(table, "cherry", 3);

    int value;
    if (get(table, "banana", &value)) {
        printf("banana: %d\n", value);
    }

    if (!get(table, "grape", &value)) {
        printf("grape not found\n");
    }

    free_table(table);
    return 0;
}
