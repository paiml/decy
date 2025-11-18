/* K&R C Chapter 4: External and Static Variables Interaction
 * Demonstrates interaction between external and static variables
 */

#include <stdio.h>

static int internal = 10;  /* Static external variable */
int external = 20;         /* External variable */

void modify_vars() {
    internal += 5;
    external += 5;
}

int main() {
    printf("Before: internal=%d, external=%d\n", internal, external);
    modify_vars();
    printf("After: internal=%d, external=%d\n", internal, external);
    return 0;
}
