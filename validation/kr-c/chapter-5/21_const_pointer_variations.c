/* K&R C Chapter 5: Pointer to Const vs Const Pointer
 * All combinations of const with pointers
 */

#include <stdio.h>

void demo_pointer_to_const(void) {
    int x = 10, y = 20;

    /* Pointer to const int - can change pointer, not value */
    const int *ptr = &x;

    printf("Pointer to const int:\n");
    printf("  *ptr = %d\n", *ptr);

    // *ptr = 15;  // ERROR: cannot modify through const pointer
    ptr = &y;  // OK: can change where pointer points
    printf("  After ptr = &y: *ptr = %d\n", *ptr);
}

void demo_const_pointer(void) {
    int x = 10, y = 20;

    /* Const pointer to int - cannot change pointer, can change value */
    int * const ptr = &x;

    printf("\nConst pointer to int:\n");
    printf("  *ptr = %d\n", *ptr);

    *ptr = 15;  // OK: can modify value
    printf("  After *ptr = 15: *ptr = %d\n", *ptr);

    // ptr = &y;  // ERROR: cannot change const pointer
}

void demo_const_pointer_to_const(void) {
    int x = 10;

    /* Const pointer to const int - cannot change either */
    const int * const ptr = &x;

    printf("\nConst pointer to const int:\n");
    printf("  *ptr = %d\n", *ptr);

    // *ptr = 15;  // ERROR: cannot modify value
    // ptr = &y;   // ERROR: cannot change pointer
}

/* Function parameters with const */
void print_array(const int *arr, int n) {
    printf("Array: ");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");

    // arr[0] = 100;  // ERROR: arr is pointer to const
}

void modify_array(int * const arr, int n) {
    for (int i = 0; i < n; i++)
        arr[i] *= 2;

    // arr = NULL;  // ERROR: arr is const pointer
}

int main() {
    demo_pointer_to_const();
    demo_const_pointer();
    demo_const_pointer_to_const();

    /* Function parameter examples */
    int numbers[] = {1, 2, 3, 4, 5};
    int n = sizeof(numbers) / sizeof(numbers[0]);

    printf("\nFunction with const int *:\n");
    print_array(numbers, n);

    printf("\nFunction with int * const:\n");
    printf("Before: ");
    print_array(numbers, n);
    modify_array(numbers, n);
    printf("After:  ");
    print_array(numbers, n);

    /* Const with arrays */
    const int const_arr[] = {10, 20, 30};
    printf("\nConst array:\n");
    print_array(const_arr, 3);
    // const_arr[0] = 100;  // ERROR: array is const

    return 0;
}
