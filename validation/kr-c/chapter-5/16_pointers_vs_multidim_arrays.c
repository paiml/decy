/* K&R C Chapter 5.9: Pointers vs. Multi-dimensional Arrays
 * Page 110-113
 * Difference between array of pointers and 2D array
 */

#include <stdio.h>
#include <string.h>

int main() {
    /* Array of pointers - each row can have different length */
    char *month_names[] = {
        "Illegal month",
        "January", "February", "March",
        "April", "May", "June",
        "July", "August", "September",
        "October", "November", "December"
    };

    /* 2D array - fixed row length */
    char days[][4] = {
        "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
    };

    printf("Array of pointers (variable length strings):\n");
    for (int i = 0; i < 13; i++) {
        printf("  month_names[%2d] = \"%s\" (ptr: %p)\n",
               i, month_names[i], (void*)month_names[i]);
    }

    printf("\n2D array (fixed length):\n");
    for (int i = 0; i < 7; i++) {
        printf("  days[%d] = \"%s\"\n", i, days[i]);
    }

    /* Memory layout difference */
    printf("\nMemory layout:\n");
    printf("  sizeof(month_names) = %zu bytes (array of %zu pointers)\n",
           sizeof(month_names), sizeof(month_names) / sizeof(char*));
    printf("  sizeof(days) = %zu bytes (7 rows × 4 chars)\n",
           sizeof(days));

    /* Accessing elements */
    printf("\nAccessing elements:\n");
    printf("  month_names[5] = \"%s\" (pointer dereference)\n", month_names[5]);
    printf("  days[3] = \"%s\" (2D array indexing)\n", days[3]);

    /* Pointer arithmetic on array of pointers */
    char **ptr = month_names + 1;  /* Skip "Illegal month" */
    printf("\nPointer arithmetic:\n");
    for (int i = 0; i < 12; i++, ptr++) {
        printf("  Month %2d: %s\n", i + 1, *ptr);
    }

    return 0;
}
