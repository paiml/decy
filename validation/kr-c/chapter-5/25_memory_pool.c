/* K&R C Chapter 5: Memory Pool Allocation
 * Custom memory allocator using pointers
 */

#include <stdio.h>
#include <string.h>

#define POOL_SIZE 1024

typedef struct {
    char memory[POOL_SIZE];
    char *next_free;
    size_t used;
    size_t allocated_count;
} MemoryPool;

void pool_init(MemoryPool *pool) {
    pool->next_free = pool->memory;
    pool->used = 0;
    pool->allocated_count = 0;
}

void *pool_alloc(MemoryPool *pool, size_t size) {
    if (pool->used + size > POOL_SIZE) {
        printf("  Pool exhausted! (used: %zu, requested: %zu)\n",
               pool->used, size);
        return NULL;
    }

    void *ptr = pool->next_free;
    pool->next_free += size;
    pool->used += size;
    pool->allocated_count++;

    return ptr;
}

void pool_reset(MemoryPool *pool) {
    pool->next_free = pool->memory;
    pool->used = 0;
    pool->allocated_count = 0;
}

void pool_stats(MemoryPool *pool) {
    printf("Pool statistics:\n");
    printf("  Total size: %d bytes\n", POOL_SIZE);
    printf("  Used: %zu bytes (%.1f%%)\n",
           pool->used, (pool->used * 100.0) / POOL_SIZE);
    printf("  Free: %zu bytes\n", POOL_SIZE - pool->used);
    printf("  Allocations: %zu\n", pool->allocated_count);
}

int main() {
    MemoryPool pool;
    pool_init(&pool);

    printf("=== Memory Pool Demo ===\n\n");

    /* Allocate some integers */
    int *nums = (int*)pool_alloc(&pool, 10 * sizeof(int));
    if (nums) {
        printf("Allocated array of 10 ints at %p\n", (void*)nums);
        for (int i = 0; i < 10; i++)
            nums[i] = i * 10;

        printf("Values: ");
        for (int i = 0; i < 10; i++)
            printf("%d ", nums[i]);
        printf("\n\n");
    }

    /* Allocate a string */
    char *str = (char*)pool_alloc(&pool, 50);
    if (str) {
        strcpy(str, "Hello from memory pool!");
        printf("Allocated string at %p: \"%s\"\n\n", (void*)str, str);
    }

    /* Allocate a struct */
    struct point {
        int x, y;
    };

    struct point *pt = (struct point*)pool_alloc(&pool, sizeof(struct point));
    if (pt) {
        pt->x = 100;
        pt->y = 200;
        printf("Allocated struct at %p: {%d, %d}\n\n", (void*)pt, pt->x, pt->y);
    }

    pool_stats(&pool);

    /* Try to over-allocate */
    printf("\nTrying to allocate 2000 bytes (exceeds pool):\n");
    void *big = pool_alloc(&pool, 2000);
    if (!big)
        printf("Allocation failed as expected\n");

    /* Reset and reuse */
    printf("\nResetting pool...\n");
    pool_reset(&pool);
    pool_stats(&pool);

    /* Allocate again after reset */
    int *new_nums = (int*)pool_alloc(&pool, 5 * sizeof(int));
    if (new_nums) {
        printf("\nAllocated after reset at %p (same as first allocation)\n",
               (void*)new_nums);
        for (int i = 0; i < 5; i++)
            new_nums[i] = i + 1;

        printf("New values: ");
        for (int i = 0; i < 5; i++)
            printf("%d ", new_nums[i]);
        printf("\n");
    }

    return 0;
}
