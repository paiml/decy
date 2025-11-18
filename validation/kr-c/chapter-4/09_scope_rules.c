/* K&R C Chapter 4.4: Scope Rules
 * Page 80-82
 * Variable scope and shadowing
 */

#include <stdio.h>

int global_var = 100;  /* File scope */

void function1(void) {
    int local_var = 10;  /* Block scope */
    printf("function1: local_var = %d, global_var = %d\n", local_var, global_var);
}

void function2(void) {
    int global_var = 200;  /* Shadows global */
    printf("function2: global_var (shadowed) = %d\n", global_var);
}

void nested_scopes(void) {
    int x = 1;
    printf("Outer: x = %d\n", x);

    {
        int x = 2;  /* Inner scope shadows outer */
        printf("Inner: x = %d\n", x);

        {
            int x = 3;  /* Even more inner */
            printf("Innermost: x = %d\n", x);
        }

        printf("Back to inner: x = %d\n", x);
    }

    printf("Back to outer: x = %d\n", x);
}

int main() {
    printf("Global variable: %d\n\n", global_var);

    function1();
    function2();
    printf("After function2, global_var still = %d\n\n", global_var);

    nested_scopes();

    return 0;
}
