/* K&R C Chapter 4.3: External Variables
 * Page 80-82
 * Simple global variable example
 */

#include <stdio.h>

int max;  /* External variable (global) */

int main() {
    max = 100;
    printf("max = %d\n", max);
    return 0;
}
