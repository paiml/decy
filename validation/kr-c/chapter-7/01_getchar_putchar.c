/* K&R C Chapter 7.1: Standard Input and Output
 * Page 151-152
 * Basic character I/O with getchar and putchar
 */

#include <stdio.h>

int main() {
    int c;

    printf("Type some characters (Ctrl+D to end):\n");

    /* Copy input to output */
    while ((c = getchar()) != EOF)
        putchar(c);

    return 0;
}
