/* K&R C Chapter 1.5.3: Line Counting
 * Page 17
 * Count lines in input
 */

#include <stdio.h>

int main() {
    int c, nl;

    nl = 0;
    while ((c = getchar()) != EOF)
        if (c == '\n')
            ++nl;
    printf("%d\n", nl);
    return 0;
}
