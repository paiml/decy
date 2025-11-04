/* K&R C Chapter 4.4: Static Variables
 * Page 83
 * Static variable example
 */

#include <stdio.h>

static int counter = 0;  /* Static global variable */

void increment() {
    counter++;
}

int main() {
    increment();
    increment();
    printf("counter = %d\n", counter);
    return 0;
}
