/* K&R C Chapter 4: Variadic Functions
 * Functions with variable number of arguments
 */

#include <stdio.h>
#include <stdarg.h>

/* Sum variable number of integers */
int sum(int count, ...) {
    va_list args;
    va_start(args, count);

    int total = 0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, int);
    }

    va_end(args);
    return total;
}

/* Find maximum of variable args */
int max(int count, ...) {
    if (count <= 0)
        return 0;

    va_list args;
    va_start(args, count);

    int maximum = va_arg(args, int);
    for (int i = 1; i < count; i++) {
        int val = va_arg(args, int);
        if (val > maximum)
            maximum = val;
    }

    va_end(args);
    return maximum;
}

/* Custom printf-like function */
void my_print(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);

    printf("my_print: ");
    vprintf(fmt, args);

    va_end(args);
}

/* Average of doubles */
double average(int count, ...) {
    if (count <= 0)
        return 0.0;

    va_list args;
    va_start(args, count);

    double total = 0.0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, double);
    }

    va_end(args);
    return total / count;
}

int main() {
    printf("=== Variadic Functions ===\n\n");

    /* Sum function */
    printf("sum(3, 10, 20, 30) = %d\n", sum(3, 10, 20, 30));
    printf("sum(5, 1, 2, 3, 4, 5) = %d\n", sum(5, 1, 2, 3, 4, 5));
    printf("sum(1, 100) = %d\n", sum(1, 100));

    /* Max function */
    printf("\nmax(4, 25, 10, 50, 30) = %d\n", max(4, 25, 10, 50, 30));
    printf("max(6, 3, 7, 2, 9, 1, 5) = %d\n", max(6, 3, 7, 2, 9, 1, 5));

    /* Custom printf */
    printf("\n");
    my_print("Hello, %s! Number: %d\n", "World", 42);
    my_print("Float: %.2f, String: %s\n", 3.14159, "test");

    /* Average */
    printf("\naverage(3, 10.0, 20.0, 30.0) = %.2f\n",
           average(3, 10.0, 20.0, 30.0));
    printf("average(5, 1.5, 2.5, 3.5, 4.5, 5.5) = %.2f\n",
           average(5, 1.5, 2.5, 3.5, 4.5, 5.5));

    return 0;
}
