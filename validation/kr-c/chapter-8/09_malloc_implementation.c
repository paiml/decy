/* K&R C Chapter 8.7: Example - A Storage Allocator
 * Page 185-189
 * Simple malloc implementation concept
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define ALLOCSIZE 10000

static char allocbuf[ALLOCSIZE];
static char *allocp = allocbuf;

/* Simple allocator - allocates from static buffer */
char *simple_alloc(int n) {
    if (allocbuf + ALLOCSIZE - allocp >= n) {
        allocp += n;
        return allocp - n;
    } else {
        return NULL;
    }
}

/* Free all allocated memory */
void simple_free_all(void) {
    allocp = allocbuf;
}

int main() {
    char *p1, *p2, *p3;

    printf("Allocator buffer size: %d bytes\n", ALLOCSIZE);

    /* Allocate some memory */
    p1 = simple_alloc(100);
    if (p1 != NULL) {
        printf("Allocated 100 bytes at %p\n", (void *)p1);
        sprintf(p1, "Hello from p1");
    }

    p2 = simple_alloc(200);
    if (p2 != NULL) {
        printf("Allocated 200 bytes at %p\n", (void *)p2);
        sprintf(p2, "Hello from p2");
    }

    p3 = simple_alloc(5000);
    if (p3 != NULL) {
        printf("Allocated 5000 bytes at %p\n", (void *)p3);
    }

    printf("p1: %s\n", p1);
    printf("p2: %s\n", p2);

    /* Try to allocate too much */
    char *p4 = simple_alloc(20000);
    if (p4 == NULL) {
        printf("Failed to allocate 20000 bytes (buffer full)\n");
    }

    /* Free all */
    simple_free_all();
    printf("Freed all memory\n");

    return 0;
}
