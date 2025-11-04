/* K&R C Chapter 6: Opaque Types (Information Hiding)
 * Forward declarations and opaque pointers
 */

#include <stdio.h>
#include <stdlib.h>

/* Opaque handle - definition hidden */
typedef struct counter Counter;

/* Counter implementation (normally in .c file) */
struct counter {
    int value;
    int step;
    const char *name;
};

/* Public API */
Counter *counter_create(const char *name, int initial, int step) {
    Counter *c = malloc(sizeof(Counter));
    if (c) {
        c->value = initial;
        c->step = step;
        c->name = name;
    }
    return c;
}

void counter_destroy(Counter *c) {
    free(c);
}

int counter_next(Counter *c) {
    int current = c->value;
    c->value += c->step;
    return current;
}

int counter_get(Counter *c) {
    return c->value;
}

void counter_reset(Counter *c, int value) {
    c->value = value;
}

void counter_print(Counter *c) {
    printf("%s: %d\n", c->name, c->value);
}

/* Another opaque type - Stack */
typedef struct stack Stack;

struct stack {
    int *data;
    int top;
    int capacity;
};

Stack *stack_create(int capacity) {
    Stack *s = malloc(sizeof(Stack));
    if (s) {
        s->data = malloc(capacity * sizeof(int));
        s->top = -1;
        s->capacity = capacity;
    }
    return s;
}

void stack_destroy(Stack *s) {
    free(s->data);
    free(s);
}

int stack_push(Stack *s, int value) {
    if (s->top >= s->capacity - 1)
        return 0;  /* Full */

    s->data[++s->top] = value;
    return 1;
}

int stack_pop(Stack *s, int *value) {
    if (s->top < 0)
        return 0;  /* Empty */

    *value = s->data[s->top--];
    return 1;
}

int stack_is_empty(Stack *s) {
    return s->top < 0;
}

int main() {
    printf("=== Opaque Types Demo ===\n\n");

    /* Counter usage */
    Counter *c1 = counter_create("Main", 0, 1);
    Counter *c2 = counter_create("Even", 0, 2);

    printf("Counters:\n");
    for (int i = 0; i < 5; i++) {
        printf("  %d: ", i);
        printf("Main=%d ", counter_next(c1));
        printf("Even=%d ", counter_next(c2));
        printf("\n");
    }

    counter_reset(c1, 100);
    printf("\nAfter reset:\n");
    counter_print(c1);
    counter_print(c2);

    counter_destroy(c1);
    counter_destroy(c2);

    /* Stack usage */
    printf("\nStack:\n");
    Stack *stack = stack_create(10);

    printf("Pushing: 10, 20, 30\n");
    stack_push(stack, 10);
    stack_push(stack, 20);
    stack_push(stack, 30);

    printf("Popping:\n");
    int value;
    while (stack_pop(stack, &value)) {
        printf("  %d\n", value);
    }

    if (stack_is_empty(stack))
        printf("Stack is empty\n");

    stack_destroy(stack);

    return 0;
}
