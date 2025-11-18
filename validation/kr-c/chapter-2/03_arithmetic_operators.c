/* K&R C Chapter 2.5: Arithmetic Operators
 * Page 39-40
 * Binary arithmetic operators
 */

#include <stdio.h>

int main() {
    int a = 10, b = 3;
    int sum, diff, prod, quot, rem;

    sum = a + b;
    diff = a - b;
    prod = a * b;
    quot = a / b;
    rem = a % b;

    printf("a = %d, b = %d\n", a, b);
    printf("a + b = %d\n", sum);
    printf("a - b = %d\n", diff);
    printf("a * b = %d\n", prod);
    printf("a / b = %d\n", quot);
    printf("a %% b = %d\n", rem);
    return 0;
}
