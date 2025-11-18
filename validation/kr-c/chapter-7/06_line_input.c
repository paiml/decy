/* K&R C Chapter 7.7: Line Input and Output
 * Page 164-165
 * Reading lines with fgets and writing with fputs
 */

#include <stdio.h>
#include <string.h>

#define MAXLINE 1000

int main() {
    char line[MAXLINE];
    int count = 0;

    printf("Enter lines of text (Ctrl+D to end):\n");

    /* Read lines and echo them */
    while (fgets(line, MAXLINE, stdin) != NULL) {
        count++;
        printf("%3d: ", count);
        fputs(line, stdout);
    }

    printf("\nTotal lines: %d\n", count);

    return 0;
}
