/* K&R C Chapter 5: Function Pointers - Callbacks
 * Using function pointers for callbacks and qsort
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Comparison functions for qsort */
int compare_ints_asc(const void *a, const void *b) {
    return (*(int*)a - *(int*)b);
}

int compare_ints_desc(const void *a, const void *b) {
    return (*(int*)b - *(int*)a);
}

int compare_strings(const void *a, const void *b) {
    return strcmp(*(const char**)a, *(const char**)b);
}

/* Generic apply function */
void apply_to_array(int *arr, int n, void (*func)(int*)) {
    for (int i = 0; i < n; i++)
        func(&arr[i]);
}

void double_value(int *x) {
    *x *= 2;
}

void square_value(int *x) {
    *x *= *x;
}

/* Callback-based iteration */
void foreach(int *arr, int n, void (*callback)(int, int)) {
    for (int i = 0; i < n; i++)
        callback(i, arr[i]);
}

void print_element(int index, int value) {
    printf("  arr[%d] = %d\n", index, value);
}

int main() {
    /* Sorting with function pointers */
    int numbers[] = {5, 2, 8, 1, 9, 3, 7, 4, 6};
    int n = sizeof(numbers) / sizeof(numbers[0]);

    printf("Original: ");
    for (int i = 0; i < n; i++) printf("%d ", numbers[i]);
    printf("\n");

    qsort(numbers, n, sizeof(int), compare_ints_asc);
    printf("Ascending: ");
    for (int i = 0; i < n; i++) printf("%d ", numbers[i]);
    printf("\n");

    qsort(numbers, n, sizeof(int), compare_ints_desc);
    printf("Descending: ");
    for (int i = 0; i < n; i++) printf("%d ", numbers[i]);
    printf("\n");

    /* String sorting */
    char *words[] = {"zebra", "apple", "mango", "banana", "cherry"};
    int nwords = sizeof(words) / sizeof(words[0]);

    printf("\nOriginal words: ");
    for (int i = 0; i < nwords; i++) printf("%s ", words[i]);
    printf("\n");

    qsort(words, nwords, sizeof(char*), compare_strings);
    printf("Sorted words: ");
    for (int i = 0; i < nwords; i++) printf("%s ", words[i]);
    printf("\n");

    /* Apply functions to array */
    int values[] = {1, 2, 3, 4, 5};
    int nvals = sizeof(values) / sizeof(values[0]);

    printf("\nOriginal values: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n");

    apply_to_array(values, nvals, double_value);
    printf("After doubling: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n");

    apply_to_array(values, nvals, square_value);
    printf("After squaring: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n");

    /* Callback iteration */
    printf("\nUsing callback:\n");
    foreach(values, nvals, print_element);

    return 0;
}
