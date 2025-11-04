/* K&R C Chapter 4.10: Recursion
 * Page 86-87
 * Recursive functions - factorial and Fibonacci
 */

#include <stdio.h>

/* factorial: compute n! recursively */
int factorial(int n) {
    if (n <= 1)
        return 1;
    else
        return n * factorial(n - 1);
}

/* fibonacci: compute nth Fibonacci number recursively */
int fibonacci(int n) {
    if (n <= 1)
        return n;
    else
        return fibonacci(n - 1) + fibonacci(n - 2);
}

/* power: compute x^n recursively */
double power(double x, int n) {
    if (n == 0)
        return 1.0;
    else if (n > 0)
        return x * power(x, n - 1);
    else
        return 1.0 / power(x, -n);
}

int main() {
    printf("Factorials:\n");
    for (int i = 0; i <= 10; i++)
        printf("%d! = %d\n", i, factorial(i));

    printf("\nFibonacci sequence:\n");
    for (int i = 0; i < 15; i++)
        printf("fib(%d) = %d\n", i, fibonacci(i));

    printf("\nPowers:\n");
    printf("2^10 = %.0f\n", power(2.0, 10));
    printf("3^5 = %.0f\n", power(3.0, 5));
    printf("2^-3 = %.3f\n", power(2.0, -3));

    return 0;
}
