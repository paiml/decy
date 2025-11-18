/* K&R C Chapter 1 Exercise: Detab
 * Based on Chapter 1 concepts
 * Replace tabs with spaces
 */

#include <stdio.h>

#define TABSTOP 8

int main() {
    int c, pos;

    pos = 0;
    while ((c = getchar()) != EOF) {
        if (c == '\t') {
            /* Replace tab with spaces to next tab stop */
            do {
                putchar(' ');
                ++pos;
            } while (pos % TABSTOP != 0);
        } else if (c == '\n') {
            putchar(c);
            pos = 0;
        } else {
            putchar(c);
            ++pos;
        }
    }

    return 0;
}
