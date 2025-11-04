/* K&R C Chapter 7: Buffering Modes
 * K&R §7.5: setvbuf, setbuf, fflush
 * Tests stream buffering control
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* Demo unbuffered I/O */
void demo_unbuffered(void) {
    printf("=== Unbuffered I/O ===\n");

    FILE *fp = fopen("unbuffered.txt", "w");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    /* Set to unbuffered mode */
    setvbuf(fp, NULL, _IONBF, 0);

    printf("Writing with unbuffered mode...\n");
    fprintf(fp, "Line 1\n");
    printf("  (written immediately)\n");
    sleep(1);

    fprintf(fp, "Line 2\n");
    printf("  (written immediately)\n");
    sleep(1);

    fprintf(fp, "Line 3\n");
    printf("  (written immediately)\n");

    fclose(fp);
    printf("\n");
}

/* Demo line buffered I/O */
void demo_line_buffered(void) {
    printf("=== Line Buffered I/O ===\n");

    FILE *fp = fopen("line_buffered.txt", "w");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    /* Set to line buffered mode */
    setvbuf(fp, NULL, _IOLBF, 0);

    printf("Writing with line buffered mode...\n");
    fprintf(fp, "Partial line... ");
    printf("  (not yet written)\n");
    sleep(1);

    fprintf(fp, "complete\\n\n");
    printf("  (written on newline)\n");
    sleep(1);

    fclose(fp);
    printf("\n");
}

/* Demo fully buffered I/O */
void demo_fully_buffered(void) {
    printf("=== Fully Buffered I/O ===\n");

    FILE *fp = fopen("fully_buffered.txt", "w");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    /* Set to fully buffered mode with custom buffer */
    char buffer[1024];
    setvbuf(fp, buffer, _IOFBF, sizeof(buffer));

    printf("Writing with fully buffered mode...\n");
    fprintf(fp, "Line 1\n");
    fprintf(fp, "Line 2\n");
    fprintf(fp, "Line 3\n");
    printf("  (buffered in memory, not written yet)\n");
    sleep(1);

    printf("Flushing buffer...\n");
    fflush(fp);
    printf("  (now written to file)\n");
    sleep(1);

    fprintf(fp, "Line 4\n");
    printf("  (buffered again)\n");

    fclose(fp);  /* Implicit flush on close */
    printf("  (flushed on close)\n\n");
}

/* Demo manual flushing */
void demo_manual_flush(void) {
    printf("=== Manual Flushing ===\n");

    printf("Writing to stdout with manual flush:\n");

    printf("Progress: ");
    fflush(stdout);  /* Ensure "Progress: " is displayed */

    for (int i = 0; i <= 10; i++) {
        printf("#");
        fflush(stdout);  /* Force immediate output */
        usleep(200000);  /* 0.2 seconds */
    }
    printf(" Done!\n\n");
}

/* Demo buffer size effects */
void demo_buffer_size(void) {
    printf("=== Buffer Size Effects ===\n");

    const char *filename = "buffer_test.txt";

    /* Small buffer */
    FILE *fp1 = fopen(filename, "w");
    char small_buffer[64];
    setvbuf(fp1, small_buffer, _IOFBF, sizeof(small_buffer));

    printf("Writing with 64-byte buffer:\n");
    for (int i = 0; i < 100; i++) {
        fprintf(fp1, "%d ", i);
        if (i % 10 == 0) {
            printf("  Position %d: buffer may flush\n", i);
        }
    }
    fclose(fp1);

    /* Large buffer */
    FILE *fp2 = fopen(filename, "w");
    char large_buffer[4096];
    setvbuf(fp2, large_buffer, _IOFBF, sizeof(large_buffer));

    printf("\nWriting with 4096-byte buffer:\n");
    for (int i = 0; i < 100; i++) {
        fprintf(fp2, "%d ", i);
    }
    printf("  All data likely stayed in buffer\n");
    fclose(fp2);

    printf("\n");
}

/* Demo buffering for performance */
void performance_comparison(void) {
    printf("=== Performance Comparison ===\n");

    const int iterations = 10000;
    const char *filename = "perf_test.txt";

    /* Unbuffered writes */
    FILE *fp = fopen(filename, "w");
    setvbuf(fp, NULL, _IONBF, 0);

    printf("Writing %d lines unbuffered...\n", iterations);
    for (int i = 0; i < iterations; i++) {
        fprintf(fp, "Line %d\n", i);
    }
    fclose(fp);
    printf("  (slower due to frequent system calls)\n");

    /* Buffered writes */
    fp = fopen(filename, "w");
    char buffer[8192];
    setvbuf(fp, buffer, _IOFBF, sizeof(buffer));

    printf("Writing %d lines buffered...\n", iterations);
    for (int i = 0; i < iterations; i++) {
        fprintf(fp, "Line %d\n", i);
    }
    fclose(fp);
    printf("  (faster - fewer system calls)\n\n");
}

/* Demo error handling with buffering */
void demo_buffering_errors(void) {
    printf("=== Buffering and Error Handling ===\n");

    FILE *fp = fopen("error_test.txt", "w");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    char buffer[1024];
    setvbuf(fp, buffer, _IOFBF, sizeof(buffer));

    /* Write data */
    fprintf(fp, "Important data\n");

    /* Check for errors before close */
    if (ferror(fp)) {
        printf("Error detected in stream\n");
        clearerr(fp);
    }

    /* Explicit flush to catch write errors */
    if (fflush(fp) != 0) {
        printf("Flush failed\n");
    } else {
        printf("Flush successful\n");
    }

    fclose(fp);
    printf("\n");
}

int main() {
    printf("=== Buffering Modes ===\n\n");

    demo_unbuffered();
    demo_line_buffered();
    demo_fully_buffered();
    demo_manual_flush();
    demo_buffer_size();
    performance_comparison();
    demo_buffering_errors();

    /* Cleanup */
    remove("unbuffered.txt");
    remove("line_buffered.txt");
    remove("fully_buffered.txt");
    remove("buffer_test.txt");
    remove("perf_test.txt");
    remove("error_test.txt");

    printf("Buffering modes:\n");
    printf("  _IONBF: Unbuffered - immediate I/O\n");
    printf("  _IOLBF: Line buffered - flush on newline\n");
    printf("  _IOFBF: Fully buffered - flush when full\n");
    printf("\nDefault buffering:\n");
    printf("  stdout/stderr: Line buffered (terminal)\n");
    printf("  stdout/stderr: Fully buffered (redirected)\n");
    printf("  Files: Fully buffered\n");
    printf("\nUse fflush() to force immediate output\n");

    return 0;
}
