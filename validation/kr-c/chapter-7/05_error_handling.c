/* K&R C Chapter 7.6: Error Handling - stderr and exit
 * Page 163-164
 * Error reporting and program termination
 */

#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
    FILE *fp;

    if (argc < 2) {
        fprintf(stderr, "Usage: %s <filename>\n", argv[0]);
        exit(1);
    }

    fp = fopen(argv[1], "r");
    if (fp == NULL) {
        fprintf(stderr, "Error: Cannot open file '%s'\n", argv[1]);
        exit(2);
    }

    printf("Successfully opened: %s\n", argv[1]);

    /* File operations here */

    fclose(fp);

    return 0;
}
