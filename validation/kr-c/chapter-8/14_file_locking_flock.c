/* K&R C Chapter 8: File Locking with flock
 * K&R §8.7: Locking mechanisms
 * Tests advisory file locking
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/file.h>
#include <fcntl.h>

void demo_exclusive_lock(void) {
    printf("=== Exclusive Lock Demo ===\n");

    const char *filename = "lockfile.txt";
    int fd = open(filename, O_RDWR | O_CREAT, 0644);

    if (fd == -1) {
        perror("open");
        return;
    }

    printf("Acquiring exclusive lock...\n");
    if (flock(fd, LOCK_EX) == 0) {
        printf("  Lock acquired\n");

        write(fd, "Locked data\n", 12);

        printf("  Holding lock for 2 seconds...\n");
        sleep(2);

        flock(fd, LOCK_UN);
        printf("  Lock released\n");
    }

    close(fd);
    unlink(filename);
    printf("\n");
}

void demo_non_blocking_lock(void) {
    printf("=== Non-Blocking Lock Demo ===\n");

    const char *filename = "lockfile2.txt";
    int fd = open(filename, O_RDWR | O_CREAT, 0644);

    printf("Trying non-blocking lock...\n");
    if (flock(fd, LOCK_EX | LOCK_NB) == 0) {
        printf("  Lock acquired immediately\n");
        flock(fd, LOCK_UN);
    } else {
        printf("  Lock would block (file locked by another process)\n");
    }

    close(fd);
    unlink(filename);
    printf("\n");
}

int main() {
    printf("=== File Locking ===\n\n");

    demo_exclusive_lock();
    demo_non_blocking_lock();

    printf("File locking:\n");
    printf("  - LOCK_EX: Exclusive lock\n");
    printf("  - LOCK_SH: Shared lock\n");
    printf("  - LOCK_UN: Unlock\n");
    printf("  - LOCK_NB: Non-blocking\n");
    printf("  - Advisory locking (cooperative)\n");

    return 0;
}
