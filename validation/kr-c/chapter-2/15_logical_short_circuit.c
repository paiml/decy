/* K&R C Chapter 2.6: Relational and Logical Operators
 * Page 42-43
 * Short-circuit evaluation
 */

#include <stdio.h>

int count = 0;

/* Function with side effect */
int increment_and_return(int x) {
    count++;
    printf("  increment_and_return(%d) called, count=%d\n", x, count);
    return x;
}

int main() {
    printf("=== Logical AND short-circuit ===\n");
    count = 0;

    /* First operand is false, second not evaluated */
    if (increment_and_return(0) && increment_and_return(1)) {
        printf("Both true\n");
    } else {
        printf("Result: false (second function NOT called)\n");
    }

    printf("\n");
    count = 0;

    /* First operand is true, second is evaluated */
    if (increment_and_return(1) && increment_and_return(1)) {
        printf("Result: both true (second function WAS called)\n");
    }

    printf("\n=== Logical OR short-circuit ===\n");
    count = 0;

    /* First operand is true, second not evaluated */
    if (increment_and_return(1) || increment_and_return(1)) {
        printf("Result: true (second function NOT called)\n");
    }

    printf("\n");
    count = 0;

    /* First operand is false, second is evaluated */
    if (increment_and_return(0) || increment_and_return(1)) {
        printf("Result: true (second function WAS called)\n");
    }

    printf("\n=== Practical example: safe array access ===\n");
    int arr[] = {10, 20, 30, 40, 50};
    int size = sizeof(arr) / sizeof(arr[0]);
    int index = 3;

    /* Safe: checks bounds before accessing */
    if (index >= 0 && index < size && arr[index] > 25) {
        printf("arr[%d] = %d is > 25\n", index, arr[index]);
    }

    /* Short-circuit prevents out-of-bounds access */
    index = 10;
    if (index >= 0 && index < size && arr[index] > 25) {
        printf("arr[%d] > 25\n", index);
    } else {
        printf("Index %d out of bounds (safe due to short-circuit)\n", index);
    }

    printf("\n=== Ternary operator (also short-circuits) ===\n");
    count = 0;
    int result = (0) ? increment_and_return(1) : increment_and_return(2);
    printf("Result: %d (only second branch evaluated)\n", result);

    count = 0;
    result = (1) ? increment_and_return(1) : increment_and_return(2);
    printf("Result: %d (only first branch evaluated)\n", result);

    return 0;
}
