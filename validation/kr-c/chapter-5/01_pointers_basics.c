/* K&R C Chapter 5.1: Pointers and Addresses
 * Page 93-94
 * Basic pointer operations
 */

#include <stdio.h>

int main() {
    int x = 1, y = 2, z[10];
    int *ip;    /* ip is a pointer to int */

    ip = &x;    /* ip now points to x */
    y = *ip;    /* y is now 1 */
    *ip = 0;    /* x is now 0 */
    ip = &z[0]; /* ip now points to z[0] */

    printf("x = %d, y = %d\n", x, y);
    printf("*ip = %d\n", *ip);
    return 0;
}
