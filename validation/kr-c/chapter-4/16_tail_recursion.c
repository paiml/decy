/* K&R C Chapter 4: Tail Recursion Optimization
 * Comparing normal and tail-recursive functions
 */

#include <stdio.h>

/* Normal recursion - not tail-recursive */
int factorial_normal(int n) {
    if (n <= 1)
        return 1;
    return n * factorial_normal(n - 1);  /* Multiplication after recursive call */
}

/* Tail recursion - can be optimized */
int factorial_tail_helper(int n, int accumulator) {
    if (n <= 1)
        return accumulator;
    return factorial_tail_helper(n - 1, n * accumulator);  /* Nothing after recursive call */
}

int factorial_tail(int n) {
    return factorial_tail_helper(n, 1);
}

/* Normal recursion - Fibonacci */
int fib_normal(int n) {
    if (n <= 1)
        return n;
    return fib_normal(n - 1) + fib_normal(n - 2);  /* Two recursive calls */
}

/* Tail recursion - Fibonacci with accumulator */
int fib_tail_helper(int n, int a, int b) {
    if (n == 0)
        return a;
    if (n == 1)
        return b;
    return fib_tail_helper(n - 1, b, a + b);
}

int fib_tail(int n) {
    return fib_tail_helper(n, 0, 1);
}

/* Sum of array - normal recursion */
int sum_normal(int *arr, int n) {
    if (n == 0)
        return 0;
    return arr[0] + sum_normal(arr + 1, n - 1);
}

/* Sum of array - tail recursion */
int sum_tail_helper(int *arr, int n, int accumulator) {
    if (n == 0)
        return accumulator;
    return sum_tail_helper(arr + 1, n - 1, accumulator + arr[0]);
}

int sum_tail(int *arr, int n) {
    return sum_tail_helper(arr, n, 0);
}

int main() {
    printf("=== Tail Recursion ===\n\n");

    /* Factorial comparison */
    printf("Factorial:\n");
    for (int i = 0; i <= 10; i++) {
        printf("  %d! = %d (normal) = %d (tail)\n",
               i, factorial_normal(i), factorial_tail(i));
    }

    /* Fibonacci comparison */
    printf("\nFibonacci (first 15):\n");
    printf("Normal: ");
    for (int i = 0; i < 15; i++) {
        printf("%d ", fib_normal(i));
    }
    printf("\n");

    printf("Tail:   ");
    for (int i = 0; i < 15; i++) {
        printf("%d ", fib_tail(i));
    }
    printf("\n");

    /* Array sum */
    int arr[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    int n = sizeof(arr) / sizeof(arr[0]);

    printf("\nArray sum:\n");
    printf("  Normal recursion: %d\n", sum_normal(arr, n));
    printf("  Tail recursion: %d\n", sum_tail(arr, n));

    printf("\nTail recursion benefits:\n");
    printf("  - Can be optimized to iteration by compiler\n");
    printf("  - Uses constant stack space\n");
    printf("  - No risk of stack overflow for deep recursion\n");

    return 0;
}
