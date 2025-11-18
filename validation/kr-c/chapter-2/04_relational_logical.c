/* K&R C Chapter 2.6: Relational and Logical Operators
 * Page 40-41
 * Relational and logical operators
 */

#include <stdio.h>

int main() {
    int a = 5, b = 10;
    int result;

    /* Relational operators */
    result = (a > b);
    printf("a > b: %d\n", result);

    result = (a < b);
    printf("a < b: %d\n", result);

    result = (a >= b);
    printf("a >= b: %d\n", result);

    result = (a <= b);
    printf("a <= b: %d\n", result);

    result = (a == b);
    printf("a == b: %d\n", result);

    result = (a != b);
    printf("a != b: %d\n", result);

    /* Logical operators */
    result = (a < b && b < 20);
    printf("a < b && b < 20: %d\n", result);

    result = (a > b || b > 0);
    printf("a > b || b > 0: %d\n", result);

    result = !(a == b);
    printf("!(a == b): %d\n", result);

    return 0;
}
