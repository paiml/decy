/* K&R C Chapter 1.5.2: Character Counting
 * Page 16
 * Count characters in input
 */

#include <stdio.h>

int main() {
    long nc;

    nc = 0;
    while (getchar() != EOF)
        ++nc;
    printf("%ld\n", nc);
    return 0;
}
