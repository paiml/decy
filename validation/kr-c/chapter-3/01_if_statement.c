/* K&R C Chapter 3.2: If-Else
 * Page 52-53
 * Basic if-else statement
 */

#include <stdio.h>

int main() {
    int n = 10;

    if (n > 0)
        printf("n is positive\n");
    else if (n < 0)
        printf("n is negative\n");
    else
        printf("n is zero\n");

    return 0;
}
