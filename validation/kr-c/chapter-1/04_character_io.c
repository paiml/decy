/* K&R C Chapter 1.5.1: File Copying
 * Page 15
 * Character input/output example
 */

#include <stdio.h>

int main() {
    int c;

    c = getchar();
    while (c != EOF) {
        putchar(c);
        c = getchar();
    }
    return 0;
}
