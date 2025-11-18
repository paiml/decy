/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 97-99
 * Pointer arithmetic with arrays
 */

#include <stdio.h>

int main() {
    int a[10];
    int *pa;
    int i;

    /* Initialize array */
    for (i = 0; i < 10; i++)
        a[i] = i * 10;

    /* Pointer arithmetic */
    pa = &a[0];
    printf("*pa = %d\n", *pa);

    pa = pa + 1;  /* Now points to a[1] */
    printf("*pa = %d\n", *pa);

    /* Array indexing with pointers */
    pa = a;  /* Equivalent to pa = &a[0] */
    for (i = 0; i < 10; i++)
        printf("*(pa + %d) = %d\n", i, *(pa + i));

    return 0;
}
