/* K&R C Chapter 2.12: Precedence and Order of Evaluation
 * Page 52-53
 * Operator precedence and associativity
 */

#include <stdio.h>

int main() {
    int a = 10, b = 20, c = 30;

    /* Arithmetic precedence */
    printf("Arithmetic precedence:\n");
    printf("a + b * c = %d (b*c first)\n", a + b * c);
    printf("(a + b) * c = %d (explicit grouping)\n", (a + b) * c);
    printf("a * b + c = %d (a*b first)\n", a * b + c);

    /* Mixed operators */
    printf("\nMixed operators:\n");
    int result1 = 5 + 3 * 2;  /* 5 + 6 = 11 */
    printf("5 + 3 * 2 = %d\n", result1);

    int result2 = 20 / 4 * 2;  /* Left-to-right: 5 * 2 = 10 */
    printf("20 / 4 * 2 = %d\n", result2);

    int result3 = 20 / (4 * 2);  /* 20 / 8 = 2 */
    printf("20 / (4 * 2) = %d\n", result3);

    /* Comparison and logical */
    printf("\nComparison and logical:\n");
    int x = 5, y = 10, z = 15;

    int cond1 = x < y && y < z;
    printf("x < y && y < z = %d\n", cond1);

    int cond2 = x < y || z < y;
    printf("x < y || z < y = %d\n", cond2);

    int cond3 = x < y < z;  /* Evaluated as (x < y) < z */
    printf("x < y < z = %d (misleading!)\n", cond3);

    /* Bitwise vs logical */
    printf("\nBitwise vs logical:\n");
    int bit_and = 5 & 3;      /* 0101 & 0011 = 0001 */
    int logical_and = 5 && 3;  /* 1 && 1 = 1 */
    printf("5 & 3 = %d (bitwise)\n", bit_and);
    printf("5 && 3 = %d (logical)\n", logical_and);

    /* Assignment associativity (right-to-left) */
    printf("\nAssignment associativity:\n");
    int p, q, r;
    p = q = r = 42;
    printf("p = q = r = 42 -> p=%d, q=%d, r=%d\n", p, q, r);

    /* Increment and arithmetic */
    printf("\nIncrement and arithmetic:\n");
    int n = 5;
    int res1 = ++n * 2;  /* 6 * 2 = 12 */
    printf("++n * 2 = %d (n=%d)\n", res1, n);

    n = 5;
    int res2 = n++ * 2;  /* 5 * 2 = 10, then n=6 */
    printf("n++ * 2 = %d (n=%d)\n", res2, n);

    return 0;
}
