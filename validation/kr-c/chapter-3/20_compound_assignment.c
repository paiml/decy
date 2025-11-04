/* K&R C Chapter 3: Compound Assignment Operators
 * K&R §3.11: Assignment shorthand operators
 * Tests +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
 */

#include <stdio.h>

void demo_arithmetic_compound(void) {
    printf("=== Arithmetic Compound Assignment ===\n");

    int x = 10;
    printf("x = %d\n", x);

    x += 5;
    printf("x += 5:  x = %d\n", x);

    x -= 3;
    printf("x -= 3:  x = %d\n", x);

    x *= 2;
    printf("x *= 2:  x = %d\n", x);

    x /= 4;
    printf("x /= 4:  x = %d\n", x);

    x %= 5;
    printf("x %%= 5:  x = %d\n", x);

    printf("\n");
}

void demo_bitwise_compound(void) {
    printf("=== Bitwise Compound Assignment ===\n");

    unsigned int flags = 0b11110000;
    printf("flags = 0x%02X\n", flags);

    flags |= 0b00001111;
    printf("flags |= 0x0F: flags = 0x%02X\n", flags);

    flags &= 0b10101010;
    printf("flags &= 0xAA: flags = 0x%02X\n", flags);

    flags ^= 0b11111111;
    printf("flags ^= 0xFF: flags = 0x%02X\n", flags);

    printf("\n");
}

void demo_shift_compound(void) {
    printf("=== Shift Compound Assignment ===\n");

    unsigned int x = 4;
    printf("x = %u\n", x);

    x <<= 2;
    printf("x <<= 2: x = %u (multiply by 4)\n", x);

    x >>= 1;
    printf("x >>= 1: x = %u (divide by 2)\n", x);

    printf("\n");
}

void demo_array_operations(void) {
    printf("=== Array Operations ===\n");

    int arr[] = {1, 2, 3, 4, 5};
    int n = sizeof(arr) / sizeof(arr[0]);

    printf("Original: ");
    for (int i = 0; i < n; i++) {
        printf("%d ", arr[i]);
    }
    printf("\n");

    /* Double all elements */
    for (int i = 0; i < n; i++) {
        arr[i] *= 2;
    }

    printf("After *=2: ");
    for (int i = 0; i < n; i++) {
        printf("%d ", arr[i]);
    }
    printf("\n\n");
}

void demo_complex_expression(void) {
    printf("=== Complex Expressions ===\n");

    int x = 5;
    printf("x = %d\n", x);

    /* Equivalent to: x = x * 2 + 3 */
    x = x * 2;
    x += 3;
    printf("x = x * 2 + 3: x = %d\n", x);

    /* Can't write: x *= 2 + 3 (that's x = x * (2 + 3)) */
    x = 5;
    x *= 2 + 3;
    printf("x *= 2 + 3: x = %d (x = x * (2 + 3))\n", x);

    printf("\n");
}

void demo_pointer_arithmetic(void) {
    printf("=== Pointer Compound Assignment ===\n");

    int arr[] = {10, 20, 30, 40, 50};
    int *p = arr;

    printf("*p = %d\n", *p);

    p += 2;
    printf("After p += 2: *p = %d\n", *p);

    p -= 1;
    printf("After p -= 1: *p = %d\n", *p);

    printf("\n");
}

int main() {
    printf("=== Compound Assignment Operators ===\n\n");

    demo_arithmetic_compound();
    demo_bitwise_compound();
    demo_shift_compound();
    demo_array_operations();
    demo_complex_expression();
    demo_pointer_arithmetic();

    printf("Compound assignment operators:\n");
    printf("  +=  -=  *=  /=  %%=\n");
    printf("  &=  |=  ^=  <<=  >>=\n");
    printf("\nBehavior:\n");
    printf("  x op= y is equivalent to x = x op y\n");
    printf("  But x is evaluated only once\n");
    printf("  More concise and potentially more efficient\n");

    return 0;
}
