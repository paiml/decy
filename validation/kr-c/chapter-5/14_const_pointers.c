/* K&R C Chapter 5: Const Pointers and Pointers to Const
 * Different const pointer variations
 */

#include <stdio.h>

int main() {
    int x = 10, y = 20;

    /* Regular pointer - can change pointer and value */
    int *p1 = &x;
    printf("p1 = %d\n", *p1);
    *p1 = 15;
    printf("After *p1 = 15: x = %d\n", x);
    p1 = &y;
    printf("After p1 = &y: *p1 = %d\n", *p1);

    /* Pointer to const - can change pointer, cannot change value */
    const int *p2 = &x;
    printf("\nPointer to const:\n");
    printf("*p2 = %d\n", *p2);
    // *p2 = 25;  // ERROR: cannot modify through const pointer
    p2 = &y;  // OK: can change where pointer points
    printf("After p2 = &y: *p2 = %d\n", *p2);

    /* Const pointer - cannot change pointer, can change value */
    int * const p3 = &x;
    printf("\nConst pointer:\n");
    printf("*p3 = %d\n", *p3);
    *p3 = 30;  // OK: can modify value
    printf("After *p3 = 30: x = %d\n", x);
    // p3 = &y;  // ERROR: cannot change where const pointer points

    /* Const pointer to const - cannot change pointer or value */
    const int * const p4 = &y;
    printf("\nConst pointer to const:\n");
    printf("*p4 = %d\n", *p4);
    // *p4 = 40;  // ERROR: cannot modify value
    // p4 = &x;   // ERROR: cannot change pointer

    /* Array of const strings */
    const char *messages[] = {
        "Hello",
        "World",
        "Const",
        "Pointers"
    };

    printf("\nArray of const strings:\n");
    for (int i = 0; i < 4; i++)
        printf("  messages[%d] = \"%s\"\n", i, messages[i]);

    return 0;
}
