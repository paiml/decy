/* K&R C Chapter 8: Shared Memory with mmap
 * K&R §8.7: Memory-mapped files for IPC
 * Tests shared memory between processes
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/wait.h>

void demo_shared_memory(void) {
    printf("=== Shared Memory Demo ===\n");

    /* Create anonymous shared mapping */
    int *shared_counter = mmap(NULL, sizeof(int),
                               PROT_READ | PROT_WRITE,
                               MAP_SHARED | MAP_ANONYMOUS, -1, 0);

    if (shared_counter == MAP_FAILED) {
        perror("mmap");
        return;
    }

    *shared_counter = 0;

    pid_t pid = fork();

    if (pid == 0) {
        /* Child: Increment counter */
        for (int i = 0; i < 1000; i++) {
            (*shared_counter)++;
        }
        printf("  Child: Counter = %d\n", *shared_counter);
        exit(0);
    } else {
        /* Parent: Increment counter */
        for (int i = 0; i < 1000; i++) {
            (*shared_counter)++;
        }
        wait(NULL);
        printf("  Parent: Final counter = %d\n", *shared_counter);
    }

    munmap(shared_counter, sizeof(int));
    printf("\n");
}

int main() {
    printf("=== Shared Memory (mmap) ===\n\n");

    demo_shared_memory();

    printf("Shared memory:\n");
    printf("  - MAP_SHARED: Changes visible to all processes\n");
    printf("  - MAP_ANONYMOUS: Not backed by file\n");
    printf("  - Fast IPC (no system calls after setup)\n");
    printf("  - Requires synchronization for safety\n");

    return 0;
}
