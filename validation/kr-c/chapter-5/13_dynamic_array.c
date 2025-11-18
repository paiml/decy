/* K&R C Chapter 5: Dynamic Arrays with Pointers
 * Memory allocation and pointer manipulation
 */

#include <stdio.h>
#include <stdlib.h>

/* create_array: allocate and initialize dynamic array */
int *create_array(int size) {
    int *arr = (int *)malloc(size * sizeof(int));
    if (arr == NULL)
        return NULL;

    for (int i = 0; i < size; i++)
        arr[i] = i * 10;

    return arr;
}

/* resize_array: resize dynamic array */
int *resize_array(int *old_arr, int old_size, int new_size) {
    int *new_arr = (int *)realloc(old_arr, new_size * sizeof(int));
    if (new_arr == NULL)
        return old_arr;  /* Keep old array if realloc fails */

    /* Initialize new elements */
    for (int i = old_size; i < new_size; i++)
        new_arr[i] = 0;

    return new_arr;
}

int main() {
    int size = 5;
    int *arr;

    /* Create dynamic array */
    arr = create_array(size);
    if (arr == NULL) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }

    printf("Initial array (size %d):\n", size);
    for (int i = 0; i < size; i++)
        printf("  arr[%d] = %d\n", i, arr[i]);

    /* Resize array */
    int new_size = 10;
    arr = resize_array(arr, size, new_size);

    printf("\nAfter resize (size %d):\n", new_size);
    for (int i = 0; i < new_size; i++)
        printf("  arr[%d] = %d\n", i, arr[i]);

    /* Pointer arithmetic on dynamic array */
    printf("\nUsing pointer arithmetic:\n");
    int *ptr = arr;
    for (int i = 0; i < new_size; i++, ptr++)
        printf("  *(arr + %d) = %d\n", i, *ptr);

    /* Free memory */
    free(arr);
    printf("\nMemory freed successfully\n");

    return 0;
}
