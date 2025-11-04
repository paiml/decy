/* K&R C Chapter 3: Short-Circuit Evaluation
 * K&R §3.8: Logical operators and side effects
 * Tests && and || short-circuiting
 */

#include <stdio.h>

int side_effect_true(void) {
    printf("  side_effect_true() called\n");
    return 1;
}

int side_effect_false(void) {
    printf("  side_effect_false() called\n");
    return 0;
}

void demo_and_short_circuit(void) {
    printf("=== AND Short-Circuit ===\n");

    printf("0 && side_effect_true():\n");
    int result = 0 && side_effect_true();
    printf("Result: %d (function not called)\n\n", result);

    printf("1 && side_effect_true():\n");
    result = 1 && side_effect_true();
    printf("Result: %d (function called)\n\n", result);
}

void demo_or_short_circuit(void) {
    printf("=== OR Short-Circuit ===\n");

    printf("1 || side_effect_false():\n");
    int result = 1 || side_effect_false();
    printf("Result: %d (function not called)\n\n", result);

    printf("0 || side_effect_false():\n");
    result = 0 || side_effect_false();
    printf("Result: %d (function called)\n\n", result);
}

void demo_null_check(void) {
    printf("=== Null Pointer Check ===\n");

    int arr[] = {1, 2, 3};
    int *p = arr;

    if (p != NULL && *p > 0) {
        printf("Safe: *p = %d\n", *p);
    }

    p = NULL;
    if (p != NULL && *p > 0) {  /* *p not evaluated */
        printf("This won't print\n");
    } else {
        printf("Null pointer, *p not dereferenced\n");
    }

    printf("\n");
}

void demo_range_check(void) {
    printf("=== Range Check ===\n");

    int arr[] = {10, 20, 30};
    int size = 3;
    int index = 5;

    /* Safe bounds check */
    if (index >= 0 && index < size && arr[index] > 15) {
        printf("Element: %d\n", arr[index]);
    } else {
        printf("Index %d out of bounds or condition false\n", index);
    }

    printf("\n");
}

int main() {
    printf("=== Short-Circuit Evaluation ===\n\n");

    demo_and_short_circuit();
    demo_or_short_circuit();
    demo_null_check();
    demo_range_check();

    printf("Short-circuit rules:\n");
    printf("  - AND (&&): if left is false, right not evaluated\n");
    printf("  - OR (||): if left is true, right not evaluated\n");
    printf("  - Use for safe pointer/array checks\n");
    printf("  - Side effects may not occur\n");

    return 0;
}
