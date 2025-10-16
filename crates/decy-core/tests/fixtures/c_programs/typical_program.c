/* Typical C program for testing transpilation
 *
 * This program demonstrates common C constructs that Decy should handle:
 * - Function definitions
 * - Variable declarations
 * - Arithmetic operations
 * - Control flow (if/else)
 * - Loops (for)
 * - I/O (printf)
 *
 * Expected output:
 *   Sum: 55
 *   Factorial: 120
 */

#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

int main() {
    // Calculate sum of 0..10
    int sum = 0;
    for (int i = 0; i <= 10; i++) {
        sum = add(sum, i);
    }

    printf("Sum: %d\n", sum);

    // Calculate factorial of 5
    int fact = factorial(5);
    printf("Factorial: %d\n", fact);

    return 0;
}
