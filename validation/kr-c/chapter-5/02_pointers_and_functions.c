/* K&R C Chapter 5.2: Pointers and Function Arguments
 * Page 95-96
 * Pass-by-reference using pointers
 */

#include <stdio.h>

void swap(int *px, int *py);

int main() {
    int a = 10, b = 20;

    printf("Before swap: a = %d, b = %d\n", a, b);
    swap(&a, &b);
    printf("After swap: a = %d, b = %d\n", a, b);
    return 0;
}

/* swap: interchange *px and *py */
void swap(int *px, int *py) {
    int temp;

    temp = *px;
    *px = *py;
    *py = temp;
}
