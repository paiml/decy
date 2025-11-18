/* K&R C Chapter 3: Comma Operator
 * K&R §3.6: Expression separator
 * Tests comma operator in expressions
 */

#include <stdio.h>

void demo_basic_comma(void) {
    printf("=== Basic Comma Operator ===\n");

    int a, b, c;
    a = (b = 2, c = 3, b + c);
    printf("a = (b=2, c=3, b+c): a=%d, b=%d, c=%d\n", a, b, c);
    printf("\n");
}

void demo_for_loop(void) {
    printf("=== Comma in for Loop ===\n");

    /* Multiple initializations and increments */
    for (int i = 0, j = 10; i < j; i++, j--) {
        printf("i=%d, j=%d\n", i, j);
    }
    printf("\n");
}

void demo_swap_with_comma(void) {
    printf("=== Swap with Comma ===\n");

    int x = 5, y = 10;
    printf("Before: x=%d, y=%d\n", x, y);

    int temp;
    (temp = x, x = y, y = temp);
    printf("After:  x=%d, y=%d\n", x, y);
    printf("\n");
}

int main() {
    printf("=== Comma Operator ===\n\n");

    demo_basic_comma();
    demo_for_loop();
    demo_swap_with_comma();

    printf("Comma operator:\n");
    printf("  - Evaluates left to right\n");
    printf("  - Returns rightmost expression value\n");
    printf("  - Lowest precedence operator\n");
    printf("  - Often used in for loops\n");

    return 0;
}
