/* K&R C Chapter 7: Memory-Mapped I/O
 * K&R §8.7: mmap for efficient file access
 * Tests memory-mapped file operations
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>

/* Create and map a file */
void simple_mmap_demo(void) {
    printf("=== Simple mmap Demo ===\n");

    const char *filename = "mmap_test.txt";
    const char *text = "Hello, memory-mapped I/O!\n";
    size_t text_len = strlen(text);

    /* Create file */
    int fd = open(filename, O_RDWR | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return;
    }

    /* Set file size */
    if (ftruncate(fd, text_len) == -1) {
        perror("ftruncate");
        close(fd);
        return;
    }

    /* Map file to memory */
    char *mapped = mmap(NULL, text_len, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (mapped == MAP_FAILED) {
        perror("mmap");
        close(fd);
        return;
    }

    printf("File mapped to memory at %p\n", (void*)mapped);

    /* Write to mapped memory (writes to file) */
    memcpy(mapped, text, text_len);
    printf("Wrote: %s", text);

    /* Read from mapped memory */
    printf("Read:  %.*s", (int)text_len, mapped);

    /* Unmap and close */
    munmap(mapped, text_len);
    close(fd);

    /* Verify file contents */
    FILE *fp = fopen(filename, "r");
    char buffer[100];
    fgets(buffer, sizeof(buffer), fp);
    printf("File contents: %s", buffer);
    fclose(fp);

    remove(filename);
    printf("\n");
}

/* mmap for reading large files */
void mmap_read_large_file(void) {
    printf("=== mmap for Reading Large File ===\n");

    const char *filename = "large_file.txt";

    /* Create test file */
    FILE *fp = fopen(filename, "w");
    for (int i = 0; i < 1000; i++) {
        fprintf(fp, "Line %d: Some data here\n", i);
    }
    fclose(fp);

    /* Open and map file */
    int fd = open(filename, O_RDONLY);
    if (fd == -1) {
        perror("open");
        return;
    }

    /* Get file size */
    struct stat sb;
    if (fstat(fd, &sb) == -1) {
        perror("fstat");
        close(fd);
        return;
    }

    printf("File size: %ld bytes\n", (long)sb.st_size);

    /* Map file */
    char *mapped = mmap(NULL, sb.st_size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (mapped == MAP_FAILED) {
        perror("mmap");
        close(fd);
        return;
    }

    /* Count lines by scanning mapped memory */
    int line_count = 0;
    for (size_t i = 0; i < (size_t)sb.st_size; i++) {
        if (mapped[i] == '\n') {
            line_count++;
        }
    }

    printf("Lines counted: %d\n", line_count);

    /* Search for text */
    char *found = memmem(mapped, sb.st_size, "Line 500", 8);
    if (found) {
        printf("Found 'Line 500' at offset: %ld\n", found - mapped);
    }

    munmap(mapped, sb.st_size);
    close(fd);
    remove(filename);
    printf("\n");
}

/* mmap for in-place modification */
void mmap_modify_file(void) {
    printf("=== mmap for In-Place Modification ===\n");

    const char *filename = "modify_test.txt";

    /* Create file with lowercase text */
    FILE *fp = fopen(filename, "w");
    fprintf(fp, "hello world from memory mapped io\n");
    fclose(fp);

    /* Map file for read/write */
    int fd = open(filename, O_RDWR);
    struct stat sb;
    fstat(fd, &sb);

    char *mapped = mmap(NULL, sb.st_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (mapped == MAP_FAILED) {
        perror("mmap");
        close(fd);
        return;
    }

    printf("Original: %.*s", (int)sb.st_size, mapped);

    /* Modify in place - convert to uppercase */
    for (size_t i = 0; i < (size_t)sb.st_size; i++) {
        if (mapped[i] >= 'a' && mapped[i] <= 'z') {
            mapped[i] = mapped[i] - 'a' + 'A';
        }
    }

    printf("Modified: %.*s", (int)sb.st_size, mapped);

    /* Ensure changes are written to disk */
    msync(mapped, sb.st_size, MS_SYNC);

    munmap(mapped, sb.st_size);
    close(fd);

    /* Verify changes */
    fp = fopen(filename, "r");
    char buffer[100];
    fgets(buffer, sizeof(buffer), fp);
    printf("Verified: %s", buffer);
    fclose(fp);

    remove(filename);
    printf("\n");
}

/* Shared memory between processes */
void mmap_shared_memory(void) {
    printf("=== mmap for Shared Memory ===\n");

    /* Create anonymous shared mapping */
    int *shared = mmap(NULL, sizeof(int), PROT_READ | PROT_WRITE,
                       MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    if (shared == MAP_FAILED) {
        perror("mmap");
        return;
    }

    *shared = 0;
    printf("Initial shared value: %d\n", *shared);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child process */
        printf("  Child: Reading shared value: %d\n", *shared);
        *shared = 42;
        printf("  Child: Set shared value to 42\n");
        exit(0);
    } else {
        /* Parent process */
        wait(NULL);
        printf("  Parent: Reading shared value: %d\n", *shared);
    }

    munmap(shared, sizeof(int));
    printf("\n");
}

/* mmap with different protections */
void mmap_protections(void) {
    printf("=== mmap Protection Modes ===\n");

    const char *filename = "prot_test.txt";
    const char *text = "Protected text\n";
    size_t text_len = strlen(text);

    /* Create file */
    int fd = open(filename, O_RDWR | O_CREAT | O_TRUNC, 0644);
    write(fd, text, text_len);

    /* Read-only mapping */
    char *readonly = mmap(NULL, text_len, PROT_READ, MAP_PRIVATE, fd, 0);
    if (readonly != MAP_FAILED) {
        printf("Read-only mapping: %.*s", (int)text_len, readonly);
        /* Attempting to write would cause SIGSEGV */
        munmap(readonly, text_len);
    }

    /* Read-write mapping */
    char *readwrite = mmap(NULL, text_len, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (readwrite != MAP_FAILED) {
        printf("Read-write mapping (can modify)\n");
        readwrite[0] = 'X';  /* This works */
        printf("Modified: %.*s", (int)text_len, readwrite);
        munmap(readwrite, text_len);
    }

    close(fd);
    remove(filename);
    printf("\n");
}

/* mmap vs read performance comparison */
void mmap_vs_read_performance(void) {
    printf("=== mmap vs read Performance ===\n");

    const char *filename = "perf_test.txt";
    const size_t size = 10 * 1024 * 1024;  /* 10 MB */

    /* Create test file */
    int fd = open(filename, O_RDWR | O_CREAT | O_TRUNC, 0644);
    char *buf = malloc(size);
    memset(buf, 'A', size);
    write(fd, buf, size);
    close(fd);
    free(buf);

    printf("File size: %zu MB\n", size / (1024 * 1024));

    /* Benchmark read() */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    fd = open(filename, O_RDONLY);
    buf = malloc(size);
    read(fd, buf, size);
    close(fd);
    free(buf);

    gettimeofday(&end, NULL);
    double read_time = (end.tv_sec - start.tv_sec) +
                       (end.tv_usec - start.tv_usec) / 1000000.0;
    printf("  read(): %.6f seconds\n", read_time);

    /* Benchmark mmap() */
    gettimeofday(&start, NULL);

    fd = open(filename, O_RDONLY);
    char *mapped = mmap(NULL, size, PROT_READ, MAP_PRIVATE, fd, 0);
    /* Access the data (page fault will occur) */
    volatile char c = mapped[0];
    (void)c;
    munmap(mapped, size);
    close(fd);

    gettimeofday(&end, NULL);
    double mmap_time = (end.tv_sec - start.tv_sec) +
                       (end.tv_usec - start.tv_usec) / 1000000.0;
    printf("  mmap(): %.6f seconds\n", mmap_time);

    remove(filename);
    printf("\n");
}

int main() {
    printf("=== Memory-Mapped I/O ===\n\n");

    simple_mmap_demo();
    mmap_read_large_file();
    mmap_modify_file();
    mmap_shared_memory();
    mmap_protections();
    mmap_vs_read_performance();

    printf("mmap advantages:\n");
    printf("  - Fast access to file contents\n");
    printf("  - Efficient for large files\n");
    printf("  - OS handles paging automatically\n");
    printf("  - Multiple processes can share mappings\n");
    printf("  - In-place modification without explicit I/O\n");
    printf("\nmmap disadvantages:\n");
    printf("  - Address space consumption\n");
    printf("  - Page faults on first access\n");
    printf("  - Not portable to all platforms\n");

    return 0;
}
