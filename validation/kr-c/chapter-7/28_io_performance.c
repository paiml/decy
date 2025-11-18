/* K&R C Chapter 7: I/O Performance Patterns
 * K&R §7.5, §8.5: Optimizing I/O operations
 * Tests buffering, batching, and performance techniques
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/time.h>

/* Measure elapsed time */
double elapsed_time(struct timeval start, struct timeval end) {
    return (end.tv_sec - start.tv_sec) + (end.tv_usec - start.tv_usec) / 1000000.0;
}

/* Benchmark: byte-by-byte write */
void benchmark_byte_by_byte(const char *filename, int bytes) {
    printf("=== Byte-by-Byte Write (%d bytes) ===\n", bytes);

    struct timeval start, end;
    gettimeofday(&start, NULL);

    FILE *fp = fopen(filename, "w");
    for (int i = 0; i < bytes; i++) {
        fputc('A', fp);
    }
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Time: %.6f seconds\n\n", elapsed_time(start, end));
}

/* Benchmark: buffered write */
void benchmark_buffered_write(const char *filename, int bytes) {
    printf("=== Buffered Write (%d bytes) ===\n", bytes);

    struct timeval start, end;
    gettimeofday(&start, NULL);

    char *buffer = malloc(bytes);
    memset(buffer, 'A', bytes);

    FILE *fp = fopen(filename, "w");
    fwrite(buffer, 1, bytes, fp);
    fclose(fp);

    free(buffer);

    gettimeofday(&end, NULL);
    printf("  Time: %.6f seconds\n\n", elapsed_time(start, end));
}

/* Benchmark: different buffer sizes */
void benchmark_buffer_sizes(const char *filename) {
    printf("=== Buffer Size Comparison ===\n");

    const int total_bytes = 1024 * 1024;  /* 1 MB */
    int buffer_sizes[] = {1, 64, 512, 4096, 65536};
    int num_sizes = sizeof(buffer_sizes) / sizeof(buffer_sizes[0]);

    for (int i = 0; i < num_sizes; i++) {
        int bufsize = buffer_sizes[i];
        char *buffer = malloc(bufsize);
        memset(buffer, 'A', bufsize);

        struct timeval start, end;
        gettimeofday(&start, NULL);

        FILE *fp = fopen(filename, "w");
        int iterations = total_bytes / bufsize;
        for (int j = 0; j < iterations; j++) {
            fwrite(buffer, 1, bufsize, fp);
        }
        fclose(fp);

        gettimeofday(&end, NULL);
        printf("  Buffer %6d bytes: %.6f seconds\n",
               bufsize, elapsed_time(start, end));

        free(buffer);
    }
    printf("\n");
}

/* Benchmark: stdio vs low-level I/O */
void benchmark_stdio_vs_lowlevel(const char *filename, int bytes) {
    printf("=== stdio vs Low-Level I/O (%d bytes) ===\n", bytes);

    char *buffer = malloc(bytes);
    memset(buffer, 'A', bytes);

    /* stdio */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    FILE *fp = fopen(filename, "w");
    fwrite(buffer, 1, bytes, fp);
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  stdio:      %.6f seconds\n", elapsed_time(start, end));

    /* Low-level */
    gettimeofday(&start, NULL);

    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    write(fd, buffer, bytes);
    close(fd);

    gettimeofday(&end, NULL);
    printf("  Low-level:  %.6f seconds\n\n", elapsed_time(start, end));

    free(buffer);
}

/* Benchmark: sequential vs random access */
void benchmark_access_patterns(const char *filename) {
    printf("=== Sequential vs Random Access ===\n");

    const int size = 1024 * 1024;  /* 1 MB */
    char *data = malloc(size);
    memset(data, 'A', size);

    /* Create file */
    FILE *fp = fopen(filename, "w");
    fwrite(data, 1, size, fp);
    fclose(fp);

    /* Sequential read */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    fp = fopen(filename, "r");
    char buffer[1024];
    while (fread(buffer, 1, sizeof(buffer), fp) > 0);
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Sequential read: %.6f seconds\n", elapsed_time(start, end));

    /* Random access read */
    gettimeofday(&start, NULL);

    fp = fopen(filename, "r");
    for (int i = 0; i < 1000; i++) {
        long offset = (rand() % (size - 1024));
        fseek(fp, offset, SEEK_SET);
        fread(buffer, 1, sizeof(buffer), fp);
    }
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Random read:     %.6f seconds\n\n", elapsed_time(start, end));

    free(data);
}

/* Benchmark: fflush frequency */
void benchmark_flush_frequency(const char *filename) {
    printf("=== Flush Frequency Impact ===\n");

    const int iterations = 10000;
    char line[] = "Test line\n";

    /* Flush every write */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    FILE *fp = fopen(filename, "w");
    for (int i = 0; i < iterations; i++) {
        fputs(line, fp);
        fflush(fp);
    }
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Flush every write: %.6f seconds\n", elapsed_time(start, end));

    /* No explicit flush */
    gettimeofday(&start, NULL);

    fp = fopen(filename, "w");
    for (int i = 0; i < iterations; i++) {
        fputs(line, fp);
    }
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  No explicit flush: %.6f seconds\n", elapsed_time(start, end));

    /* Flush every 100 writes */
    gettimeofday(&start, NULL);

    fp = fopen(filename, "w");
    for (int i = 0; i < iterations; i++) {
        fputs(line, fp);
        if (i % 100 == 0) {
            fflush(fp);
        }
    }
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Flush every 100:   %.6f seconds\n\n", elapsed_time(start, end));
}

/* Benchmark: append vs rewrite */
void benchmark_append_vs_rewrite(const char *filename) {
    printf("=== Append vs Rewrite ===\n");

    char line[] = "Test line\n";
    const int iterations = 1000;

    /* Rewrite entire file each time */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    for (int i = 0; i < iterations; i++) {
        FILE *fp = fopen(filename, "w");
        for (int j = 0; j <= i; j++) {
            fputs(line, fp);
        }
        fclose(fp);
    }

    gettimeofday(&end, NULL);
    printf("  Rewrite: %.6f seconds\n", elapsed_time(start, end));

    /* Append mode */
    remove(filename);
    gettimeofday(&start, NULL);

    for (int i = 0; i < iterations; i++) {
        FILE *fp = fopen(filename, "a");
        fputs(line, fp);
        fclose(fp);
    }

    gettimeofday(&end, NULL);
    printf("  Append:  %.6f seconds\n\n", elapsed_time(start, end));
}

/* Benchmark: text vs binary mode */
void benchmark_text_vs_binary(const char *filename) {
    printf("=== Text vs Binary Mode ===\n");

    const int size = 1024 * 1024;
    char *data = malloc(size);
    memset(data, 'A', size);

    /* Text mode */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    FILE *fp = fopen(filename, "w");
    fwrite(data, 1, size, fp);
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Text mode:   %.6f seconds\n", elapsed_time(start, end));

    /* Binary mode */
    gettimeofday(&start, NULL);

    fp = fopen(filename, "wb");
    fwrite(data, 1, size, fp);
    fclose(fp);

    gettimeofday(&end, NULL);
    printf("  Binary mode: %.6f seconds\n\n", elapsed_time(start, end));

    free(data);
}

int main() {
    const char *test_file = "perf_test.txt";

    printf("=== I/O Performance Patterns ===\n\n");

    /* Byte-by-byte vs buffered */
    benchmark_byte_by_byte(test_file, 10000);
    benchmark_buffered_write(test_file, 10000);

    /* Buffer size impact */
    benchmark_buffer_sizes(test_file);

    /* stdio vs low-level */
    benchmark_stdio_vs_lowlevel(test_file, 1024 * 1024);

    /* Access patterns */
    benchmark_access_patterns(test_file);

    /* Flush frequency */
    benchmark_flush_frequency(test_file);

    /* Append vs rewrite */
    benchmark_append_vs_rewrite(test_file);

    /* Text vs binary */
    benchmark_text_vs_binary(test_file);

    /* Cleanup */
    remove(test_file);

    printf("Performance optimization tips:\n");
    printf("  1. Use larger buffers (4K-64K optimal)\n");
    printf("  2. Batch writes instead of frequent small writes\n");
    printf("  3. Minimize fflush() calls\n");
    printf("  4. Use binary mode when possible\n");
    printf("  5. Prefer sequential access over random\n");
    printf("  6. Use append mode for log files\n");
    printf("  7. Consider memory-mapped I/O for large files\n");

    return 0;
}
