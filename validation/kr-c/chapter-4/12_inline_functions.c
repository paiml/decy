/* K&R C Chapter 4: Inline Functions (C99)
 * Inline function optimization hint
 */

#include <stdio.h>

/* Inline functions for performance */
static inline int max_inline(int a, int b) {
    return (a > b) ? a : b;
}

static inline int min_inline(int a, int b) {
    return (a < b) ? a : b;
}

static inline int square_inline(int x) {
    return x * x;
}

/* Non-inline for comparison */
int square_normal(int x) {
    return x * x;
}

/* Inline with more complex logic */
static inline float clamp(float value, float min, float max) {
    if (value < min) return min;
    if (value > max) return max;
    return value;
}

static inline int abs_inline(int x) {
    return (x < 0) ? -x : x;
}

int main() {
    printf("=== Inline Functions ===\n\n");

    int a = 10, b = 20;

    printf("max(%d, %d) = %d\n", a, b, max_inline(a, b));
    printf("min(%d, %d) = %d\n", a, b, min_inline(a, b));

    printf("\nSquares:\n");
    for (int i = 1; i <= 5; i++) {
        printf("  square(%d) = %d\n", i, square_inline(i));
    }

    printf("\nClamp examples:\n");
    printf("  clamp(5.0, 0.0, 10.0) = %.1f\n", clamp(5.0, 0.0, 10.0));
    printf("  clamp(-5.0, 0.0, 10.0) = %.1f\n", clamp(-5.0, 0.0, 10.0));
    printf("  clamp(15.0, 0.0, 10.0) = %.1f\n", clamp(15.0, 0.0, 10.0));

    printf("\nAbsolute values:\n");
    printf("  abs(%d) = %d\n", -42, abs_inline(-42));
    printf("  abs(%d) = %d\n", 42, abs_inline(42));

    /* Inline vs normal */
    printf("\nInline vs normal (both should give same result):\n");
    printf("  inline: square(7) = %d\n", square_inline(7));
    printf("  normal: square(7) = %d\n", square_normal(7));

    return 0;
}
