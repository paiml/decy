/* K&R C Chapter 5: Pointers to Pointers
 * Advanced pointer concepts
 * Double pointers and pointer arrays
 */

#include <stdio.h>
#include <string.h>

/* sort_strings: sort array of strings using pointers */
void sort_strings(char *strings[], int n) {
    int i, j;
    char *temp;

    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            if (strcmp(strings[i], strings[j]) > 0) {
                temp = strings[i];
                strings[i] = strings[j];
                strings[j] = temp;
            }
        }
    }
}

int main() {
    char *fruits[] = {
        "banana",
        "apple",
        "cherry",
        "date",
        "elderberry"
    };
    int n = sizeof(fruits) / sizeof(fruits[0]);

    printf("Before sorting:\n");
    for (int i = 0; i < n; i++)
        printf("  %s\n", fruits[i]);

    sort_strings(fruits, n);

    printf("\nAfter sorting:\n");
    for (int i = 0; i < n; i++)
        printf("  %s\n", fruits[i]);

    /* Pointer to pointer example */
    char **ptr = fruits;
    printf("\nUsing pointer to pointer:\n");
    for (int i = 0; i < n; i++) {
        printf("  ptr[%d] = %s\n", i, ptr[i]);
    }

    return 0;
}
