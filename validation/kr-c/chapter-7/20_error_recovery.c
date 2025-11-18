/* K&R C Chapter 7: Error Recovery Patterns
 * K&R §7.5, §8.7: Error handling and recovery
 * Tests error detection, reporting, and recovery strategies
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

/* Error recovery: Retry pattern */
FILE *open_with_retry(const char *filename, const char *mode, int max_attempts) {
    FILE *fp = NULL;
    int attempts = 0;

    printf("Attempting to open: %s\n", filename);

    while (attempts < max_attempts) {
        attempts++;
        fp = fopen(filename, mode);

        if (fp != NULL) {
            printf("  Success on attempt %d\n", attempts);
            return fp;
        }

        printf("  Attempt %d failed: %s\n", attempts, strerror(errno));

        if (errno == EACCES || errno == EPERM) {
            printf("  Permission denied - retrying won't help\n");
            break;
        }

        if (attempts < max_attempts) {
            printf("  Retrying...\n");
        }
    }

    printf("  Failed after %d attempts\n", attempts);
    return NULL;
}

/* Error recovery: Fallback to alternative */
FILE *open_with_fallback(const char *primary, const char *fallback, const char *mode) {
    FILE *fp = fopen(primary, mode);

    if (fp != NULL) {
        printf("Opened primary file: %s\n", primary);
        return fp;
    }

    printf("Primary file failed: %s\n", strerror(errno));
    printf("Trying fallback: %s\n", fallback);

    fp = fopen(fallback, mode);
    if (fp != NULL) {
        printf("Opened fallback file: %s\n", fallback);
    } else {
        printf("Fallback also failed: %s\n", strerror(errno));
    }

    return fp;
}

/* Error recovery: Partial read recovery */
int read_with_recovery(FILE *fp, void *buffer, size_t size, size_t count) {
    size_t read_count = 0;
    size_t total_read = 0;

    while (total_read < count) {
        read_count = fread((char *)buffer + (total_read * size), size,
                          count - total_read, fp);

        total_read += read_count;

        if (read_count == 0) {
            if (feof(fp)) {
                printf("  Reached EOF after reading %zu items\n", total_read);
                break;
            }
            if (ferror(fp)) {
                printf("  Error after reading %zu items\n", total_read);
                clearerr(fp);

                /* Try to recover */
                if (total_read > 0) {
                    printf("  Partial data recovered\n");
                    return total_read;
                }
                return -1;
            }
        }
    }

    return total_read;
}

/* Error recovery: Transaction pattern */
int write_transaction(const char *filename, const char *data) {
    char temp_filename[256];
    snprintf(temp_filename, sizeof(temp_filename), "%s.tmp", filename);

    printf("Transaction: Writing to temp file\n");

    /* Write to temporary file first */
    FILE *fp = fopen(temp_filename, "w");
    if (fp == NULL) {
        printf("  Failed to create temp file: %s\n", strerror(errno));
        return -1;
    }

    if (fprintf(fp, "%s", data) < 0) {
        printf("  Failed to write data: %s\n", strerror(errno));
        fclose(fp);
        remove(temp_filename);
        return -1;
    }

    /* Ensure data is written */
    if (fflush(fp) != 0 || ferror(fp)) {
        printf("  Failed to flush: %s\n", strerror(errno));
        fclose(fp);
        remove(temp_filename);
        return -1;
    }

    fclose(fp);

    /* Atomic rename */
    printf("  Renaming temp file to target\n");
    if (rename(temp_filename, filename) != 0) {
        printf("  Failed to rename: %s\n", strerror(errno));
        remove(temp_filename);
        return -1;
    }

    printf("  Transaction committed successfully\n");
    return 0;
}

/* Error recovery: Checkpoint pattern */
typedef struct {
    FILE *fp;
    long checkpoint;
} CheckpointFile;

CheckpointFile *checkpoint_open(const char *filename, const char *mode) {
    CheckpointFile *cf = malloc(sizeof(CheckpointFile));
    if (cf == NULL) {
        return NULL;
    }

    cf->fp = fopen(filename, mode);
    if (cf->fp == NULL) {
        free(cf);
        return NULL;
    }

    cf->checkpoint = ftell(cf->fp);
    printf("Checkpoint created at position: %ld\n", cf->checkpoint);
    return cf;
}

void checkpoint_save(CheckpointFile *cf) {
    if (cf && cf->fp) {
        cf->checkpoint = ftell(cf->fp);
        printf("Checkpoint saved at position: %ld\n", cf->checkpoint);
    }
}

void checkpoint_restore(CheckpointFile *cf) {
    if (cf && cf->fp) {
        fseek(cf->fp, cf->checkpoint, SEEK_SET);
        clearerr(cf->fp);
        printf("Checkpoint restored to position: %ld\n", cf->checkpoint);
    }
}

void checkpoint_close(CheckpointFile *cf) {
    if (cf) {
        if (cf->fp) {
            fclose(cf->fp);
        }
        free(cf);
    }
}

/* Error recovery: Graceful degradation */
int process_file_with_degradation(const char *filename) {
    FILE *fp = fopen(filename, "r");
    if (fp == NULL) {
        printf("Cannot open file: %s\n", strerror(errno));
        printf("Using default configuration instead\n");
        return 0;  /* Graceful degradation - use defaults */
    }

    char buffer[256];
    int lines_read = 0;
    int errors = 0;

    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        lines_read++;

        /* Simulate parsing that might fail */
        if (buffer[0] == '#') {
            /* Skip comments */
            continue;
        }

        /* If parse fails, log but continue */
        if (strlen(buffer) > 100) {
            errors++;
            printf("  Warning: Line %d too long, skipping\n", lines_read);
            continue;
        }
    }

    fclose(fp);

    printf("Processed %d lines with %d errors\n", lines_read, errors);

    if (errors > lines_read / 2) {
        printf("Too many errors - using defaults\n");
        return 0;
    }

    return 1;
}

/* Error recovery: Cleanup on error */
int process_with_cleanup(const char *input, const char *output) {
    FILE *in = NULL;
    FILE *out = NULL;
    char *buffer = NULL;
    int result = -1;

    printf("Processing with automatic cleanup\n");

    /* Allocate resources */
    in = fopen(input, "r");
    if (in == NULL) {
        printf("  Error opening input: %s\n", strerror(errno));
        goto cleanup;
    }

    out = fopen(output, "w");
    if (out == NULL) {
        printf("  Error opening output: %s\n", strerror(errno));
        goto cleanup;
    }

    buffer = malloc(1024);
    if (buffer == NULL) {
        printf("  Error allocating buffer\n");
        goto cleanup;
    }

    /* Process */
    while (fgets(buffer, 1024, in) != NULL) {
        if (fputs(buffer, out) == EOF) {
            printf("  Error writing output\n");
            goto cleanup;
        }
    }

    result = 0;  /* Success */
    printf("  Processing completed successfully\n");

cleanup:
    printf("  Cleaning up resources\n");
    if (buffer) free(buffer);
    if (out) fclose(out);
    if (in) fclose(in);

    if (result != 0) {
        /* Remove partial output on error */
        remove(output);
        printf("  Partial output removed\n");
    }

    return result;
}

int main() {
    printf("=== Error Recovery Patterns ===\n\n");

    /* Create test files */
    FILE *fp = fopen("test_input.txt", "w");
    fprintf(fp, "Test data\n");
    fclose(fp);

    /* Retry pattern */
    printf("1. Retry Pattern:\n");
    fp = open_with_retry("test_input.txt", "r", 3);
    if (fp) fclose(fp);
    fp = open_with_retry("nonexistent.txt", "r", 3);
    if (fp) fclose(fp);
    printf("\n");

    /* Fallback pattern */
    printf("2. Fallback Pattern:\n");
    fp = open_with_fallback("nonexistent.txt", "test_input.txt", "r");
    if (fp) fclose(fp);
    printf("\n");

    /* Transaction pattern */
    printf("3. Transaction Pattern:\n");
    write_transaction("transaction_test.txt", "Critical data that must be atomic\n");
    printf("\n");

    /* Checkpoint pattern */
    printf("4. Checkpoint Pattern:\n");
    CheckpointFile *cf = checkpoint_open("test_input.txt", "r");
    if (cf) {
        char buffer[50];
        fgets(buffer, sizeof(buffer), cf->fp);
        printf("  Read: %s", buffer);
        checkpoint_save(cf);
        fgets(buffer, sizeof(buffer), cf->fp);
        checkpoint_restore(cf);
        fgets(buffer, sizeof(buffer), cf->fp);
        printf("  Read again: %s", buffer);
        checkpoint_close(cf);
    }
    printf("\n");

    /* Graceful degradation */
    printf("5. Graceful Degradation:\n");
    process_file_with_degradation("test_input.txt");
    printf("\n");

    /* Cleanup on error */
    printf("6. Cleanup on Error:\n");
    process_with_cleanup("test_input.txt", "test_output.txt");
    printf("\n");

    /* Cleanup test files */
    remove("test_input.txt");
    remove("test_output.txt");
    remove("transaction_test.txt");

    printf("Error recovery strategies:\n");
    printf("  1. Retry: Try operation multiple times\n");
    printf("  2. Fallback: Use alternative resource\n");
    printf("  3. Transaction: Atomic operations via temp files\n");
    printf("  4. Checkpoint: Save/restore file positions\n");
    printf("  5. Degradation: Continue with reduced functionality\n");
    printf("  6. Cleanup: Use goto for reliable resource cleanup\n");

    return 0;
}
