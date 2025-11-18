/* K&R C Chapter 5.11: Pointers to Functions
 * Page 118-119
 * Function pointers for callback
 */

#include <stdio.h>

int add(int a, int b) { return a + b; }
int subtract(int a, int b) { return a - b; }
int multiply(int a, int b) { return a * b; }

int apply(int (*operation)(int, int), int x, int y) {
    return operation(x, y);
}

int main() {
    int a = 10, b = 5;

    printf("add(%d, %d) = %d\n", a, b, apply(add, a, b));
    printf("subtract(%d, %d) = %d\n", a, b, apply(subtract, a, b));
    printf("multiply(%d, %d) = %d\n", a, b, apply(multiply, a, b));

    return 0;
}
