/* K&R C Chapter 3: Operator Precedence and Associativity
 * K&R §3.7: Expression evaluation order
 * Tests operator precedence rules
 */

#include <stdio.h>

void demo_arithmetic_precedence(void) {
    printf("=== Arithmetic Precedence ===\n");

    int result = 2 + 3 * 4;
    printf("2 + 3 * 4 = %d (not 20)\n", result);

    result = (2 + 3) * 4;
    printf("(2 + 3) * 4 = %d\n", result);

    result = 10 - 5 - 2;
    printf("10 - 5 - 2 = %d (left associative)\n", result);

    printf("\n");
}

void demo_logical_precedence(void) {
    printf("=== Logical Precedence ===\n");

    int a = 1, b = 0, c = 1;

    int result = a || b && c;
    printf("1 || 0 && 1 = %d (AND before OR)\n", result);

    result = (a || b) && c;
    printf("(1 || 0) && 1 = %d\n", result);

    printf("\n");
}

void demo_pointer_precedence(void) {
    printf("=== Pointer Precedence ===\n");

    int arr[] = {1, 2, 3};
    int *p = arr;

    printf("*p++ = %d (postincrement has higher precedence)\n", *p++);
    printf("After: *p = %d\n", *p);

    p = arr;
    printf("(*p)++ = %d (increment value)\n", (*p)++);
    printf("After: arr[0] = %d\n", arr[0]);

    printf("\n");
}

void demo_bitwise_precedence(void) {
    printf("=== Bitwise Precedence ===\n");

    unsigned int x = 5;  /* 0101 */
    unsigned int y = 3;  /* 0011 */

    int result = x & y == 1;  /* Wrong */
    printf("x & y == 1 = %d (comparison first!)\n", result);

    result = (x & y) == 1;  /* Correct */
    printf("(x & y) == 1 = %d\n", result);

    printf("\n");
}

void demo_associativity(void) {
    printf("=== Associativity ===\n");

    int a = 16;

    /* Right-to-left: assignment */
    int x, y, z;
    x = y = z = 10;
    printf("x = y = z = 10: x=%d, y=%d, z=%d\n", x, y, z);

    /* Left-to-right: arithmetic */
    int result = a / 4 / 2;
    printf("16 / 4 / 2 = %d (left-to-right)\n", result);

    printf("\n");
}

int main() {
    printf("=== Operator Precedence and Associativity ===\n\n");

    demo_arithmetic_precedence();
    demo_logical_precedence();
    demo_pointer_precedence();
    demo_bitwise_precedence();
    demo_associativity();

    printf("Key precedence rules (high to low):\n");
    printf("  1. () [] -> .\n");
    printf("  2. ! ~ ++ -- * & (unary)\n");
    printf("  3. * / %%\n");
    printf("  4. + -\n");
    printf("  5. << >>\n");
    printf("  6. < <= > >=\n");
    printf("  7. == !=\n");
    printf("  8. &\n");
    printf("  9. ^\n");
    printf(" 10. |\n");
    printf(" 11. &&\n");
    printf(" 12. ||\n");
    printf(" 13. ?:\n");
    printf(" 14. = += -= etc\n");
    printf(" 15. ,\n");

    return 0;
}
