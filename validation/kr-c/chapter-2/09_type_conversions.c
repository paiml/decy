/* K&R C Chapter 2.7: Type Conversions
 * Page 44-46
 * Implicit and explicit type conversions
 */

#include <stdio.h>

int main() {
    int i = 42;
    float f = 3.14;
    double d = 2.71828;
    char c = 'A';

    printf("Implicit conversions:\n");

    /* int to float */
    float f1 = i;
    printf("int %d to float: %f\n", i, f1);

    /* float to int (truncation) */
    int i1 = f;
    printf("float %f to int: %d\n", f, i1);

    /* char to int */
    int i2 = c;
    printf("char '%c' to int: %d\n", c, i2);

    /* Mixed arithmetic */
    float result = i + f;
    printf("int + float: %d + %f = %f\n", i, f, result);

    printf("\nExplicit conversions (casts):\n");

    /* Explicit cast */
    int i3 = (int)d;
    printf("(int)%f = %d\n", d, i3);

    /* Cast in expression */
    float ratio = (float)i / 3;
    printf("(float)%d / 3 = %f\n", i, ratio);

    /* Without cast */
    int int_div = i / 3;
    printf("%d / 3 (int division) = %d\n", i, int_div);

    /* Pointer cast */
    long addr = (long)&i;
    printf("Address of i: %ld (0x%lX)\n", addr, addr);

    return 0;
}
