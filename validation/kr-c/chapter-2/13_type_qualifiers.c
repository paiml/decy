/* K&R C Chapter 2.4: Declarations - Type Qualifiers
 * Page 40-41
 * const and volatile qualifiers
 */

#include <stdio.h>

/* Const global */
const double PI = 3.14159265359;
const int MAX_SIZE = 1000;

int main() {
    /* Const variables */
    const int max = 100;
    const char *message = "Hello, World!";

    printf("PI = %.10f\n", PI);
    printf("MAX_SIZE = %d\n", MAX_SIZE);
    printf("max = %d\n", max);
    printf("message = %s\n", message);

    /* Cannot modify const */
    // PI = 3.14;  // ERROR
    // max = 200;  // ERROR

    /* Can modify pointer, not pointee */
    message = "Goodbye!";  /* OK: pointer changed */
    printf("message = %s\n", message);
    // message[0] = 'X';  /* ERROR: cannot modify const char */

    /* Const pointer to non-const */
    int value = 42;
    int * const ptr1 = &value;
    *ptr1 = 100;  /* OK: can modify value */
    printf("value = %d\n", value);
    // ptr1 = &max;  /* ERROR: cannot change const pointer */

    /* Const array */
    const int primes[] = {2, 3, 5, 7, 11, 13};
    int size = sizeof(primes) / sizeof(primes[0]);

    printf("\nPrime numbers: ");
    for (int i = 0; i < size; i++)
        printf("%d ", primes[i]);
    printf("\n");

    /* Cannot modify const array */
    // primes[0] = 1;  /* ERROR */

    /* Volatile qualifier (rarely used, for hardware registers) */
    volatile int hardware_status = 0;
    hardware_status = 1;  /* May change unexpectedly by hardware */

    printf("\nhardware_status = %d\n", hardware_status);

    /* Const function parameters (promise not to modify) */
    const int nums[] = {10, 20, 30, 40, 50};
    int sum = 0;
    for (int i = 0; i < 5; i++)
        sum += nums[i];

    printf("Sum of const array: %d\n", sum);

    return 0;
}
