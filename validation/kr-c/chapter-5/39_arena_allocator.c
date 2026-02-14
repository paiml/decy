/* K&R C Chapter 5: Memory Arena Allocator
 * K&R §5.10, §8.7: Fast linear allocator for short-lived data
 * Tests arena allocation pattern
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ARENA_SIZE (1024 * 1024)  /* 1 MB */

typedef struct {
    char *memory;
    size_t size;
    size_t used;
} Arena;

/* Create arena */
Arena *arena_create(size_t size) {
    Arena *arena = malloc(sizeof(Arena));
    arena->memory = malloc(size);
    arena->size = size;
    arena->used = 0;

    printf("Arena created: %zu bytes\n", size);
    return arena;
}

/* Allocate from arena */
void *arena_alloc(Arena *arena, size_t size) {
    /* Align to 8 bytes */
    size_t aligned_size = (size + 7) & ~7;

    if (arena->used + aligned_size > arena->size) {
        fprintf(stderr, "Arena out of memory\n");
        return NULL;
    }

    void *ptr = arena->memory + arena->used;
    arena->used += aligned_size;

    return ptr;
}

/* Reset arena (doesn't free, just resets counter) */
void arena_reset(Arena *arena) {
    arena->used = 0;
    printf("Arena reset: %zu bytes reclaimed\n", arena->size);
}

/* Get arena usage */
void arena_stats(Arena *arena) {
    printf("Arena stats:\n");
    printf("  Total:     %zu bytes\n", arena->size);
    printf("  Used:      %zu bytes\n", arena->used);
    printf("  Available: %zu bytes\n", arena->size - arena->used);
    printf("  Usage:     %.1f%%\n", (arena->used * 100.0) / arena->size);
}

/* Destroy arena */
void arena_destroy(Arena *arena) {
    free(arena->memory);
    free(arena);
}

/* Example: String pool */
typedef struct {
    char **strings;
    int count;
    int capacity;
} StringPool;

StringPool *string_pool_create(Arena *arena, int capacity) {
    StringPool *pool = arena_alloc(arena, sizeof(StringPool));
    pool->strings = arena_alloc(arena, capacity * sizeof(char*));
    pool->count = 0;
    pool->capacity = capacity;
    return pool;
}

void string_pool_add(Arena *arena, StringPool *pool, const char *str) {
    if (pool->count >= pool->capacity) {
        printf("String pool full\n");
        return;
    }

    size_t len = strlen(str) + 1;
    char *copy = arena_alloc(arena, len);
    strcpy(copy, str);

    pool->strings[pool->count++] = copy;
}

void string_pool_print(StringPool *pool) {
    printf("String pool (%d strings):\n", pool->count);
    for (int i = 0; i < pool->count; i++) {
        printf("  [%d] %s\n", i, pool->strings[i]);
    }
}

/* Example: Short-lived calculations */
typedef struct {
    int id;
    char name[50];
    double *scores;
    int score_count;
} Student;

Student *create_student(Arena *arena, int id, const char *name, int score_count) {
    Student *s = arena_alloc(arena, sizeof(Student));
    s->id = id;
    strncpy(s->name, name, sizeof(s->name) - 1);
    s->scores = arena_alloc(arena, score_count * sizeof(double));
    s->score_count = score_count;
    return s;
}

void demo_student_processing(Arena *arena) {
    printf("\n=== Student Processing (Ephemeral Data) ===\n");

    size_t start_used = arena->used;

    /* Create short-lived student records */
    Student *s1 = create_student(arena, 1, "Alice", 5);
    s1->scores[0] = 95.5;
    s1->scores[1] = 88.0;
    s1->scores[2] = 92.5;
    s1->scores[3] = 87.0;
    s1->scores[4] = 90.5;

    Student *s2 = create_student(arena, 2, "Bob", 5);
    s2->scores[0] = 78.0;
    s2->scores[1] = 85.5;
    s2->scores[2] = 82.0;
    s2->scores[3] = 88.5;
    s2->scores[4] = 79.0;

    /* Process students */
    printf("Student %d: %s\n", s1->id, s1->name);
    double avg1 = 0;
    for (int i = 0; i < s1->score_count; i++) {
        avg1 += s1->scores[i];
    }
    avg1 /= s1->score_count;
    printf("  Average: %.2f\n", avg1);

    printf("Student %d: %s\n", s2->id, s2->name);
    double avg2 = 0;
    for (int i = 0; i < s2->score_count; i++) {
        avg2 += s2->scores[i];
    }
    avg2 /= s2->score_count;
    printf("  Average: %.2f\n", avg2);

    size_t used_for_students = arena->used - start_used;
    printf("\nTemporary data used: %zu bytes\n", used_for_students);
    printf("(No individual free() calls needed!)\n");
}

int main() {
    printf("=== Memory Arena Allocator ===\n\n");

    /* Create arena */
    Arena *arena = arena_create(ARENA_SIZE);
    printf("\n");

    /* String pool example */
    printf("=== String Pool Example ===\n");
    StringPool *pool = string_pool_create(arena, 10);
    string_pool_add(arena, pool, "Hello");
    string_pool_add(arena, pool, "World");
    string_pool_add(arena, pool, "Arena");
    string_pool_add(arena, pool, "Allocator");
    string_pool_print(pool);

    printf("\n");
    arena_stats(arena);

    /* Student processing */
    demo_student_processing(arena);

    printf("\n");
    arena_stats(arena);

    /* Reset arena */
    printf("\n");
    arena_reset(arena);
    arena_stats(arena);

    /* Destroy */
    arena_destroy(arena);

    printf("\nArena allocator benefits:\n");
    printf("  - O(1) allocation (pointer bump)\n");
    printf("  - No fragmentation\n");
    printf("  - Bulk deallocation (reset/destroy)\n");
    printf("  - Cache-friendly (linear memory)\n");
    printf("  - Perfect for short-lived/scoped data\n");

    return 0;
}
