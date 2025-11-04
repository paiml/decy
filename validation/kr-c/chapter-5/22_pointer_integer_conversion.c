/* K&R C Chapter 5: Pointer and Integer Conversions
 * Converting between pointers and integers
 */

#include <stdio.h>
#include <stdint.h>

int main() {
    int x = 42;
    int *ptr = &x;

    /* Pointer to integer conversion */
    printf("Pointer value: %p\n", (void*)ptr);

    /* Using uintptr_t (guaranteed to hold pointer) */
    uintptr_t addr = (uintptr_t)ptr;
    printf("Address as integer: %lu (0x%lX)\n", addr, addr);

    /* Integer to pointer conversion */
    int *ptr2 = (int*)addr;
    printf("Converted back: %p\n", (void*)ptr2);
    printf("Value through converted pointer: %d\n", *ptr2);

    /* Pointer arithmetic as integer arithmetic */
    int arr[] = {10, 20, 30, 40, 50};
    int *p1 = &arr[0];
    int *p2 = &arr[4];

    printf("\nPointer arithmetic:\n");
    printf("p1 = %p, p2 = %p\n", (void*)p1, (void*)p2);
    printf("p2 - p1 = %td elements\n", p2 - p1);
    printf("(char*)p2 - (char*)p1 = %td bytes\n",
           (char*)p2 - (char*)p1);

    /* Pointer alignment */
    printf("\nPointer alignment:\n");
    printf("Address of arr[0]: 0x%lX\n", (uintptr_t)&arr[0]);
    printf("Address of arr[1]: 0x%lX\n", (uintptr_t)&arr[1]);
    printf("Difference: %td bytes\n",
           (char*)&arr[1] - (char*)&arr[0]);

    /* NULL pointer */
    int *null_ptr = NULL;
    printf("\nNULL pointer:\n");
    printf("NULL as pointer: %p\n", (void*)null_ptr);
    printf("NULL as integer: %lu\n", (uintptr_t)null_ptr);

    /* Check if pointer is NULL */
    if (null_ptr == NULL)
        printf("Pointer is NULL\n");

    if (!null_ptr)
        printf("Pointer is false (NULL)\n");

    /* Pointer comparison with NULL */
    if (ptr != NULL)
        printf("ptr is not NULL\n");

    /* Offset calculations */
    printf("\nOffset calculations:\n");
    int *base = arr;
    for (int i = 0; i < 5; i++) {
        int *current = base + i;
        ptrdiff_t offset = current - base;
        printf("arr[%d]: offset = %td, address = %p, value = %d\n",
               i, offset, (void*)current, *current);
    }

    return 0;
}
