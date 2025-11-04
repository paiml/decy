/* K&R C Chapter 4: Function Composition
 * Composing functions together
 */

#include <stdio.h>

/* Simple mathematical functions */
int add_one(int x) { return x + 1; }
int double_it(int x) { return x * 2; }
int square_it(int x) { return x * x; }
int negate_it(int x) { return -x; }

/* Function composition helper */
typedef int (*UnaryFunc)(int);

int compose(UnaryFunc f, UnaryFunc g, int x) {
    return f(g(x));
}

int compose3(UnaryFunc f, UnaryFunc g, UnaryFunc h, int x) {
    return f(g(h(x)));
}

/* Pipeline: apply functions in sequence */
int pipeline(int x, UnaryFunc *functions, int count) {
    int result = x;
    for (int i = 0; i < count; i++) {
        result = functions[i](result);
    }
    return result;
}

/* Higher-order function: returns a function */
UnaryFunc select_operation(char op) {
    switch (op) {
        case '+': return add_one;
        case '*': return double_it;
        case '^': return square_it;
        case '-': return negate_it;
        default: return NULL;
    }
}

/* Map function over array */
void map(int *arr, int n, UnaryFunc f) {
    for (int i = 0; i < n; i++) {
        arr[i] = f(arr[i]);
    }
}

/* Reduce/fold array */
int reduce(int *arr, int n, int (*f)(int, int), int initial) {
    int result = initial;
    for (int i = 0; i < n; i++) {
        result = f(result, arr[i]);
    }
    return result;
}

int add(int a, int b) { return a + b; }
int multiply(int a, int b) { return a * b; }
int max_func(int a, int b) { return (a > b) ? a : b; }

int main() {
    printf("=== Function Composition ===\n\n");

    /* Simple composition */
    int x = 5;
    printf("x = %d\n", x);
    printf("add_one(x) = %d\n", add_one(x));
    printf("double_it(x) = %d\n", double_it(x));
    printf("square_it(x) = %d\n", square_it(x));

    /* Compose two functions */
    printf("\nComposed (double then add_one):\n");
    printf("  compose(add_one, double_it, %d) = %d\n",
           x, compose(add_one, double_it, x));

    printf("Composed (add_one then double):\n");
    printf("  compose(double_it, add_one, %d) = %d\n",
           x, compose(double_it, add_one, x));

    /* Compose three functions */
    printf("\nTriple composition:\n");
    printf("  compose3(add_one, double_it, square_it, %d) = %d\n",
           x, compose3(add_one, double_it, square_it, x));

    /* Pipeline */
    printf("\nPipeline:\n");
    UnaryFunc pipeline_funcs[] = {square_it, double_it, add_one};
    printf("  square -> double -> add_one: %d -> %d\n",
           x, pipeline(x, pipeline_funcs, 3));

    /* Dynamic function selection */
    printf("\nDynamic selection:\n");
    char ops[] = {'+', '*', '^', '-'};
    for (int i = 0; i < 4; i++) {
        UnaryFunc f = select_operation(ops[i]);
        if (f)
            printf("  op '%c': %d -> %d\n", ops[i], x, f(x));
    }

    /* Map */
    int arr[] = {1, 2, 3, 4, 5};
    int n = sizeof(arr) / sizeof(arr[0]);

    printf("\nMap operations:\n");
    printf("Original: ");
    for (int i = 0; i < n; i++) printf("%d ", arr[i]);
    printf("\n");

    int arr_copy[5] = {1, 2, 3, 4, 5};
    map(arr_copy, n, double_it);
    printf("After map(double_it): ");
    for (int i = 0; i < n; i++) printf("%d ", arr_copy[i]);
    printf("\n");

    /* Reduce */
    printf("\nReduce operations:\n");
    printf("Sum: %d\n", reduce(arr, n, add, 0));
    printf("Product: %d\n", reduce(arr, n, multiply, 1));
    printf("Max: %d\n", reduce(arr, n, max_func, arr[0]));

    return 0;
}
