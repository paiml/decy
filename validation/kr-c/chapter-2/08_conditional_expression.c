/* K&R C Chapter 2.11: Conditional Expressions
 * Page 51
 * Ternary operator (? :)
 */

#include <stdio.h>

int main() {
    int a = 10, b = 20;
    int max, min;

    /* Find maximum using ternary operator */
    max = (a > b) ? a : b;
    printf("max(%d, %d) = %d\n", a, b, max);

    /* Find minimum */
    min = (a < b) ? a : b;
    printf("min(%d, %d) = %d\n", a, b, min);

    /* Nested conditional */
    int x = 5, y = 10, z = 15;
    int largest = (x > y) ? ((x > z) ? x : z) : ((y > z) ? y : z);
    printf("max(%d, %d, %d) = %d\n", x, y, z, largest);

    /* Conditional in expression */
    printf("%d is %s\n", a, (a % 2 == 0) ? "even" : "odd");
    printf("%d is %s\n", b + 1, ((b + 1) % 2 == 0) ? "even" : "odd");

    /* Return absolute value */
    int n = -42;
    int abs_n = (n < 0) ? -n : n;
    printf("abs(%d) = %d\n", n, abs_n);

    return 0;
}
