/* K&R C Chapter 2.8: Increment and Decrement Operators
 * Page 43
 * ++ and -- operators
 */

#include <stdio.h>

int main() {
    int n = 5;
    int x, y;

    /* Post-increment */
    x = n++;
    printf("After n++: x = %d, n = %d\n", x, n);

    /* Pre-increment */
    n = 5;
    x = ++n;
    printf("After ++n: x = %d, n = %d\n", x, n);

    /* Post-decrement */
    n = 5;
    y = n--;
    printf("After n--: y = %d, n = %d\n", y, n);

    /* Pre-decrement */
    n = 5;
    y = --n;
    printf("After --n: y = %d, n = %d\n", y, n);

    return 0;
}
