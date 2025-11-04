/* K&R C Chapter 3: Sequence Points
 * K&R §3.9: Undefined behavior and evaluation order
 * Tests sequence points and undefined behavior
 */

#include <stdio.h>

void demo_undefined_behavior(void) {
    printf("=== Undefined Behavior Examples ===\n");

    int i = 5;

    /* UNDEFINED: Modifying i twice without sequence point */
    // int bad = i++ + i++;  /* DON'T DO THIS */
    printf("i++ + i++ is UNDEFINED BEHAVIOR\n");

    /* CORRECT: Use separate statements */
    int a = i++;
    int b = i++;
    int good = a + b;
    printf("Correct way: a=%d, b=%d, sum=%d\n", a, b, good);

    printf("\n");
}

void demo_sequence_points(void) {
    printf("=== Sequence Points ===\n");

    int x = 1;

    /* Sequence point at end of full expression */
    x = x + 1;  /* OK: sequence point after statement */
    printf("After x = x + 1: x = %d\n", x);

    /* Sequence point at && and || */
    int y = 0;
    int result = (y = 5) && (x = y + 1);
    printf("(y=5) && (x=y+1): x=%d, y=%d (y assigned before x)\n", x, y);

    /* Sequence point at ?: */
    int z = 10;
    int r = (z > 5) ? (z = 20) : (z = 30);
    printf("Ternary: z=%d, r=%d\n", z, r);

    /* Sequence point in function calls */
    printf("\n");
}

void demo_function_call_sequence(void) {
    printf("=== Function Call Sequence ===\n");

    int i = 0;

    /* UNDEFINED: Order of argument evaluation unspecified */
    // printf("%d %d %d\n", i++, i++, i++);  /* UNDEFINED */
    printf("Argument evaluation order is UNSPECIFIED\n");

    /* CORRECT: Evaluate arguments separately */
    int a = i++;
    int b = i++;
    int c = i++;
    printf("Correct: %d %d %d\n", a, b, c);

    printf("\n");
}

void demo_array_index_undefined(void) {
    printf("=== Array Index Undefined Behavior ===\n");

    int arr[] = {1, 2, 3, 4, 5};
    int i = 0;

    /* UNDEFINED: Which i++ is evaluated first? */
    // arr[i++] = i++;  /* DON'T DO THIS */
    printf("arr[i++] = i++ is UNDEFINED BEHAVIOR\n");

    /* CORRECT */
    arr[i] = i + 1;
    i++;
    printf("Correct: arr[0] = %d, i = %d\n", arr[0], i);

    printf("\n");
}

void demo_safe_patterns(void) {
    printf("=== Safe Patterns ===\n");

    int x = 5;

    /* Safe: Separate statements */
    x++;
    x++;
    printf("x after two increments: %d\n", x);

    /* Safe: Single modification per expression */
    int y = x++;
    printf("y = x++: y=%d, x=%d\n", y, x);

    /* Safe: Modifications in different expressions */
    int z = (x = 10, x + 1);
    printf("z = (x=10, x+1): z=%d, x=%d\n", z, x);

    printf("\n");
}

int main() {
    printf("=== Sequence Points ===\n\n");

    demo_undefined_behavior();
    demo_sequence_points();
    demo_function_call_sequence();
    demo_array_index_undefined();
    demo_safe_patterns();

    printf("Sequence point locations:\n");
    printf("  - End of full expression (;)\n");
    printf("  - After && and || left operand\n");
    printf("  - After ?: first operand\n");
    printf("  - At function call (after arguments)\n");
    printf("  - At comma operator\n");
    printf("\nAvoid:\n");
    printf("  - Multiple modifications to same variable\n");
    printf("  - Accessing variable multiple times with modification\n");

    return 0;
}
