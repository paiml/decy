/* K&R C Chapter 3 Exercise: Nested loops
 * Based on Chapter 3 concepts
 * Multiplication table using nested loops
 */

#include <stdio.h>

int main() {
    int i, j;
    int size = 10;

    printf("Multiplication Table (1-10):\n");
    printf("   ");

    /* Print header */
    for (i = 1; i <= size; i++)
        printf("%4d", i);
    printf("\n");

    /* Print separator */
    printf("   ");
    for (i = 1; i <= size; i++)
        printf("----");
    printf("\n");

    /* Print table */
    for (i = 1; i <= size; i++) {
        printf("%2d|", i);
        for (j = 1; j <= size; j++) {
            printf("%4d", i * j);
        }
        printf("\n");
    }

    return 0;
}
