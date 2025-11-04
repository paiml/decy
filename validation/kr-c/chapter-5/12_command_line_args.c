/* K&R C Chapter 5.10: Command-line Arguments
 * Page 114-118
 * Using argc and argv
 */

#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
    printf("Program name: %s\n", argv[0]);
    printf("Number of arguments: %d\n\n", argc - 1);

    if (argc == 1) {
        printf("No arguments provided.\n");
        printf("Usage: %s <arg1> <arg2> ...\n", argv[0]);
        return 0;
    }

    printf("Arguments:\n");
    for (int i = 1; i < argc; i++) {
        printf("  argv[%d] = \"%s\" (length: %zu)\n",
               i, argv[i], strlen(argv[i]));
    }

    /* Pointer arithmetic with argv */
    printf("\nUsing pointer arithmetic:\n");
    char **ptr = argv + 1;  /* Skip program name */
    while (*ptr != NULL) {
        printf("  *ptr = \"%s\"\n", *ptr);
        ptr++;
    }

    /* Search for specific argument */
    printf("\nSearching for '-v' flag:\n");
    int found = 0;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-v") == 0) {
            printf("  Found -v flag at position %d\n", i);
            found = 1;
            break;
        }
    }
    if (!found)
        printf("  -v flag not found\n");

    return 0;
}
