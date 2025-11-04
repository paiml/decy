/* K&R C Chapter 5.12: Complicated Declarations
 * Page 122-123
 * Examples of complex C declarations
 */

#include <stdio.h>

int main() {
    int *ap[10];           /* array of 10 pointers to int */
    int (*pa)[10];         /* pointer to array of 10 ints */
    int *f();              /* function returning pointer to int */
    int (*pf)();           /* pointer to function returning int */

    /* Simple demonstration */
    int x = 42;
    int arr[10];
    int i;

    /* Initialize */
    for (i = 0; i < 10; i++) {
        arr[i] = i * 10;
        ap[i] = &x;
    }

    pa = &arr;

    printf("(*pa)[5] = %d\n", (*pa)[5]);
    printf("*ap[0] = %d\n", *ap[0]);

    return 0;
}
