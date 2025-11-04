/* K&R C Chapter 3: Signed vs Unsigned Types
 * K&R §3.10: Integer type modifiers
 * Tests signed/unsigned behavior differences
 */

#include <stdio.h>
#include <limits.h>

void demo_range_differences(void) {
    printf("=== Range Differences ===\n");

    printf("signed char:   %d to %d\n", SCHAR_MIN, SCHAR_MAX);
    printf("unsigned char: 0 to %u\n", UCHAR_MAX);
    printf("\n");

    printf("signed int:    %d to %d\n", INT_MIN, INT_MAX);
    printf("unsigned int:  0 to %u\n", UINT_MAX);
    printf("\n");
}

void demo_wraparound(void) {
    printf("=== Wraparound Behavior ===\n");

    unsigned char uc = 255;
    printf("unsigned char = 255\n");
    printf("After +1: %u (wraps to 0)\n", (unsigned char)(uc + 1));
    printf("After +2: %u\n", (unsigned char)(uc + 2));
    printf("\n");

    signed char sc = 127;
    printf("signed char = 127\n");
    printf("After +1: %d (overflow - undefined!)\n", (signed char)(sc + 1));
    printf("\n");
}

void demo_comparison_pitfalls(void) {
    printf("=== Comparison Pitfalls ===\n");

    int si = -1;
    unsigned int ui = 1;

    if (si < ui) {
        printf("-1 < 1: This won't print!\n");
    } else {
        printf("-1 < 1: FALSE (si converted to unsigned, becomes huge)\n");
        printf("  si as unsigned: %u\n", (unsigned int)si);
    }
    printf("\n");

    /* Correct comparison */
    if ((int)ui > si) {
        printf("(int)1 > -1: TRUE (correct)\n");
    }
    printf("\n");
}

void demo_division_differences(void) {
    printf("=== Division Differences ===\n");

    int si = -7;
    int div_s = si / 3;
    printf("signed:   -7 / 3 = %d\n", div_s);

    unsigned int ui = (unsigned int)si;
    unsigned int div_u = ui / 3;
    printf("unsigned: (unsigned)-7 / 3 = %u\n", div_u);
    printf("\n");
}

void demo_right_shift(void) {
    printf("=== Right Shift Differences ===\n");

    int si = -8;  /* 11111000 in 8-bit */
    printf("signed -8 >> 1 = %d ", si >> 1);
    printf("(sign bit filled - implementation defined)\n");

    unsigned int ui = (unsigned int)si;
    printf("unsigned >> 1 = %u ", ui >> 1);
    printf("(zero filled)\n");
    printf("\n");
}

void demo_overflow_detection(void) {
    printf("=== Overflow Detection ===\n");

    /* Unsigned overflow check (well-defined) */
    unsigned int a = UINT_MAX;
    unsigned int b = 1;
    unsigned int sum = a + b;
    printf("UINT_MAX + 1 = %u (wraps to 0)\n", sum);

    if (sum < a) {
        printf("Overflow detected (sum < a)\n");
    }
    printf("\n");

    /* Signed overflow is undefined - don't rely on it */
    printf("Signed overflow is UNDEFINED BEHAVIOR\n");
    printf("Never rely on signed overflow wrapping\n");
    printf("\n");
}

void demo_loop_pitfall(void) {
    printf("=== Unsigned Loop Pitfall ===\n");

    printf("WRONG: Unsigned countdown loop:\n");
    printf("  for (unsigned i = 5; i >= 0; i--)\n");
    printf("  Problem: i >= 0 always true for unsigned!\n");
    printf("\n");

    printf("Correct countdown loops:\n");
    printf("  for (int i = 5; i >= 0; i--) or\n");
    printf("  for (unsigned i = 5; i > 0; i--) or\n");
    printf("  for (unsigned i = 5; i != UINT_MAX; i--)\n");
    printf("\n");
}

int main() {
    printf("=== Signed vs Unsigned Types ===\n\n");

    demo_range_differences();
    demo_wraparound();
    demo_comparison_pitfalls();
    demo_division_differences();
    demo_right_shift();
    demo_overflow_detection();
    demo_loop_pitfall();

    printf("Key differences:\n");
    printf("  - Unsigned overflow wraps (well-defined)\n");
    printf("  - Signed overflow is UNDEFINED BEHAVIOR\n");
    printf("  - Mixed comparisons promote signed to unsigned\n");
    printf("  - Right shift of signed is implementation-defined\n");
    printf("  - Use unsigned for bit operations and sizes\n");
    printf("  - Use signed for arithmetic that may be negative\n");

    return 0;
}
