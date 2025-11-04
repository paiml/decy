/* K&R C Chapter 4: Lazy Evaluation Pattern
 * Defer computation until value is needed
 */

#include <stdio.h>
#include <stdlib.h>

/* Thunk - deferred computation */
typedef struct {
    int (*compute)(void *);
    void *data;
    int cached;
    int value;
} Thunk;

/* Create a thunk */
Thunk make_thunk(int (*compute)(void *), void *data) {
    Thunk t = {compute, data, 0, 0};
    return t;
}

/* Force evaluation of thunk */
int force(Thunk *t) {
    if (!t->cached) {
        t->value = t->compute(t->data);
        t->cached = 1;
        printf("    [Computed value: %d]\n", t->value);
    } else {
        printf("    [Using cached value: %d]\n", t->value);
    }
    return t->value;
}

/* Example computations */
typedef struct {
    int a, b;
} AddData;

int compute_add(void *data) {
    AddData *d = (AddData*)data;
    return d->a + d->b;
}

typedef struct {
    int n;
} FactorialData;

int compute_factorial(void *data) {
    FactorialData *d = (FactorialData*)data;
    int result = 1;
    for (int i = 2; i <= d->n; i++)
        result *= i;
    return result;
}

/* Lazy list node */
typedef struct lazy_list {
    int value;
    struct lazy_list *(*next_func)(void);
    struct lazy_list *next_cached;
} LazyList;

/* Infinite sequence generator */
LazyList *next_natural(void) {
    static int counter = 1;
    LazyList *node = malloc(sizeof(LazyList));
    node->value = counter++;
    node->next_func = next_natural;
    node->next_cached = NULL;
    return node;
}

LazyList *lazy_list_next(LazyList *node) {
    if (node->next_cached == NULL) {
        node->next_cached = node->next_func();
    }
    return node->next_cached;
}

int main() {
    printf("=== Lazy Evaluation Pattern ===\n\n");

    /* Simple thunk */
    printf("Creating thunks (not computed yet):\n");
    AddData add_data = {10, 20};
    Thunk lazy_sum = make_thunk(compute_add, &add_data);

    FactorialData fact_data = {5};
    Thunk lazy_fact = make_thunk(compute_factorial, &fact_data);

    printf("  Thunks created\n\n");

    /* Force evaluation */
    printf("Forcing evaluation of lazy_sum:\n");
    printf("  Result: %d\n", force(&lazy_sum));
    printf("Forcing again (should use cache):\n");
    printf("  Result: %d\n\n", force(&lazy_sum));

    printf("Forcing evaluation of lazy_fact:\n");
    printf("  Result: %d\n", force(&lazy_fact));
    printf("Forcing again (should use cache):\n");
    printf("  Result: %d\n\n", force(&lazy_fact));

    /* Lazy infinite sequence */
    printf("Lazy infinite sequence (natural numbers):\n");
    LazyList *natural = next_natural();
    LazyList *current = natural;

    printf("  First 10 natural numbers: ");
    for (int i = 0; i < 10; i++) {
        printf("%d ", current->value);
        current = lazy_list_next(current);
    }
    printf("\n");

    /* Conditional evaluation */
    printf("\nConditional evaluation:\n");
    int use_expensive = 0;

    Thunk expensive = make_thunk(compute_factorial, &(FactorialData){10});

    if (use_expensive) {
        printf("  Using expensive computation: %d\n", force(&expensive));
    } else {
        printf("  Skipping expensive computation (never evaluated)\n");
    }

    printf("\nLazy evaluation benefits:\n");
    printf("  - Avoid unnecessary computations\n");
    printf("  - Handle infinite data structures\n");
    printf("  - Improve performance through caching\n");

    return 0;
}
