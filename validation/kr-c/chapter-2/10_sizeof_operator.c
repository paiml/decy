/* K&R C Chapter 2.2: Data Types and Sizes
 * Page 36
 * sizeof operator for type sizes
 */

#include <stdio.h>

int main() {
    printf("Size of fundamental types:\n");
    printf("sizeof(char)      = %zu bytes\n", sizeof(char));
    printf("sizeof(short)     = %zu bytes\n", sizeof(short));
    printf("sizeof(int)       = %zu bytes\n", sizeof(int));
    printf("sizeof(long)      = %zu bytes\n", sizeof(long));
    printf("sizeof(float)     = %zu bytes\n", sizeof(float));
    printf("sizeof(double)    = %zu bytes\n", sizeof(double));
    printf("\n");

    printf("Size of pointer types:\n");
    printf("sizeof(char*)     = %zu bytes\n", sizeof(char*));
    printf("sizeof(int*)      = %zu bytes\n", sizeof(int*));
    printf("sizeof(void*)     = %zu bytes\n", sizeof(void*));
    printf("\n");

    /* sizeof on variables */
    int arr[10];
    char str[100];

    printf("Size of arrays:\n");
    printf("sizeof(arr[10])   = %zu bytes\n", sizeof(arr));
    printf("sizeof(str[100])  = %zu bytes\n", sizeof(str));
    printf("\n");

    /* sizeof in expressions */
    int n = sizeof(int) * 8;
    printf("Bits in int: %d\n", n);

    /* Array length */
    int len = sizeof(arr) / sizeof(arr[0]);
    printf("Length of arr: %d\n", len);

    return 0;
}
