/* K&R C Chapter 7: Temporary Files
 * K&R §7.5: tmpfile, tmpnam, mkstemp
 * Tests creation and usage of temporary files
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Use tmpfile for anonymous temporary file */
void demo_tmpfile(void) {
    printf("=== tmpfile() demo ===\n");

    FILE *tmp = tmpfile();
    if (tmp == NULL) {
        perror("tmpfile");
        return;
    }

    /* Write data to temp file */
    fprintf(tmp, "Line 1: Temporary data\n");
    fprintf(tmp, "Line 2: More data\n");
    fprintf(tmp, "Line 3: Final line\n");

    /* Read back */
    rewind(tmp);
    char buffer[100];
    printf("Reading from tmpfile:\n");
    while (fgets(buffer, sizeof(buffer), tmp) != NULL) {
        printf("  %s", buffer);
    }

    /* tmpfile is automatically deleted on close */
    fclose(tmp);
    printf("  (tmpfile automatically deleted)\n\n");
}

/* Use tmpnam to generate unique filename */
void demo_tmpnam(void) {
    printf("=== tmpnam() demo ===\n");

    char filename[L_tmpnam];
    if (tmpnam(filename) == NULL) {
        fprintf(stderr, "tmpnam failed\n");
        return;
    }

    printf("Generated temp filename: %s\n", filename);

    /* Create and use the file */
    FILE *fp = fopen(filename, "w+");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    fprintf(fp, "Data in named temporary file\n");
    rewind(fp);

    char buffer[100];
    fgets(buffer, sizeof(buffer), fp);
    printf("Contents: %s", buffer);

    fclose(fp);

    /* Must manually delete */
    printf("Removing temp file: %s\n", filename);
    remove(filename);
    printf("\n");
}

/* Temporary file for sorting large data */
void demo_sorting_with_temp(void) {
    printf("=== Sorting with temporary file ===\n");

    int numbers[] = {64, 34, 25, 12, 22, 11, 90, 88, 45, 50, 33, 78, 21};
    int count = sizeof(numbers) / sizeof(numbers[0]);

    /* Write unsorted data to temp file */
    FILE *tmp = tmpfile();
    if (tmp == NULL) {
        perror("tmpfile");
        return;
    }

    printf("Original numbers: ");
    for (int i = 0; i < count; i++) {
        fprintf(tmp, "%d\n", numbers[i]);
        printf("%d ", numbers[i]);
    }
    printf("\n");

    /* Read back and sort in memory */
    rewind(tmp);
    int sorted[count];
    for (int i = 0; i < count; i++) {
        fscanf(tmp, "%d", &sorted[i]);
    }

    /* Simple bubble sort */
    for (int i = 0; i < count - 1; i++) {
        for (int j = 0; j < count - i - 1; j++) {
            if (sorted[j] > sorted[j + 1]) {
                int temp = sorted[j];
                sorted[j] = sorted[j + 1];
                sorted[j + 1] = temp;
            }
        }
    }

    printf("Sorted numbers:   ");
    for (int i = 0; i < count; i++) {
        printf("%d ", sorted[i]);
    }
    printf("\n");

    fclose(tmp);
    printf("\n");
}

/* Multiple temp files for merge sort */
void demo_merge_sort_temp(void) {
    printf("=== Merge sort with temp files ===\n");

    int data[] = {9, 7, 5, 3, 1, 8, 6, 4, 2, 0};
    int n = sizeof(data) / sizeof(data[0]);

    printf("Original: ");
    for (int i = 0; i < n; i++)
        printf("%d ", data[i]);
    printf("\n");

    /* Split into two temp files */
    FILE *left = tmpfile();
    FILE *right = tmpfile();

    int mid = n / 2;
    for (int i = 0; i < mid; i++)
        fprintf(left, "%d\n", data[i]);
    for (int i = mid; i < n; i++)
        fprintf(right, "%d\n", data[i]);

    printf("Split into two temp files:\n");
    printf("  Left file: %d elements\n", mid);
    printf("  Right file: %d elements\n", n - mid);

    /* Read back and display */
    rewind(left);
    rewind(right);

    int val;
    printf("Left half: ");
    while (fscanf(left, "%d", &val) == 1)
        printf("%d ", val);
    printf("\n");

    printf("Right half: ");
    while (fscanf(right, "%d", &val) == 1)
        printf("%d ", val);
    printf("\n");

    fclose(left);
    fclose(right);
    printf("\n");
}

/* Temporary work file for text processing */
void demo_text_processing_temp(void) {
    printf("=== Text processing with temp file ===\n");

    const char *input_lines[] = {
        "Hello, World!",
        "This is a test.",
        "Temporary files are useful.",
        "For processing data.",
        "End of input."
    };
    int line_count = sizeof(input_lines) / sizeof(input_lines[0]);

    FILE *tmp = tmpfile();
    if (tmp == NULL) {
        perror("tmpfile");
        return;
    }

    /* Write input */
    printf("Processing %d lines...\n", line_count);
    for (int i = 0; i < line_count; i++) {
        fprintf(tmp, "%s\n", input_lines[i]);
    }

    /* Read back and count words */
    rewind(tmp);
    char buffer[100];
    int total_words = 0;

    while (fgets(buffer, sizeof(buffer), tmp) != NULL) {
        int words = 0;
        char *token = strtok(buffer, " \t\n");
        while (token != NULL) {
            words++;
            token = strtok(NULL, " \t\n");
        }
        total_words += words;
    }

    printf("Total words: %d\n", total_words);

    fclose(tmp);
}

int main() {
    printf("=== Temporary Files ===\n\n");

    demo_tmpfile();
    demo_tmpnam();
    demo_sorting_with_temp();
    demo_merge_sort_temp();
    demo_text_processing_temp();

    printf("\nTemporary file benefits:\n");
    printf("  - Automatic cleanup (tmpfile)\n");
    printf("  - No name conflicts\n");
    printf("  - Useful for intermediate results\n");
    printf("  - External memory sorting\n");

    return 0;
}
