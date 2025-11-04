/* K&R C Chapter 4: Callback Patterns
 * Using function pointers for callbacks
 */

#include <stdio.h>
#include <stdlib.h>

/* Callback for array processing */
typedef void (*ArrayCallback)(int index, int value, void *user_data);

void array_foreach(int *arr, int n, ArrayCallback callback, void *user_data) {
    for (int i = 0; i < n; i++) {
        callback(i, arr[i], user_data);
    }
}

void print_element(int index, int value, void *user_data) {
    printf("  arr[%d] = %d\n", index, value);
}

void sum_elements(int index, int value, void *user_data) {
    int *sum = (int*)user_data;
    *sum += value;
}

/* Filter callback */
typedef int (*FilterFunc)(int value);

int filter_array(int *arr, int n, FilterFunc filter, int *result) {
    int count = 0;
    for (int i = 0; i < n; i++) {
        if (filter(arr[i])) {
            result[count++] = arr[i];
        }
    }
    return count;
}

int is_even(int x) { return x % 2 == 0; }
int is_positive(int x) { return x > 0; }
int is_greater_than_10(int x) { return x > 10; }

/* Transform callback */
typedef int (*TransformFunc)(int value);

void array_map(int *arr, int n, TransformFunc transform) {
    for (int i = 0; i < n; i++) {
        arr[i] = transform(arr[i]);
    }
}

int double_value(int x) { return x * 2; }
int square_value(int x) { return x * x; }

/* Comparison callback for sorting */
typedef int (*CompareFunc)(int a, int b);

void bubble_sort(int *arr, int n, CompareFunc compare) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (compare(arr[j], arr[j + 1]) > 0) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int compare_asc(int a, int b) { return a - b; }
int compare_desc(int a, int b) { return b - a; }

int main() {
    printf("=== Callback Patterns ===\n\n");

    int numbers[] = {5, -2, 8, -3, 12, 1, 15, -7};
    int n = sizeof(numbers) / sizeof(numbers[0]);

    /* Foreach with callback */
    printf("Array elements:\n");
    array_foreach(numbers, n, print_element, NULL);

    int sum = 0;
    array_foreach(numbers, n, sum_elements, &sum);
    printf("Sum: %d\n\n", sum);

    /* Filter with callback */
    int filtered[10];
    int count;

    count = filter_array(numbers, n, is_even, filtered);
    printf("Even numbers (%d): ", count);
    for (int i = 0; i < count; i++) printf("%d ", filtered[i]);
    printf("\n");

    count = filter_array(numbers, n, is_positive, filtered);
    printf("Positive numbers (%d): ", count);
    for (int i = 0; i < count; i++) printf("%d ", filtered[i]);
    printf("\n");

    count = filter_array(numbers, n, is_greater_than_10, filtered);
    printf("Greater than 10 (%d): ", count);
    for (int i = 0; i < count; i++) printf("%d ", filtered[i]);
    printf("\n\n");

    /* Map/transform */
    int values[] = {1, 2, 3, 4, 5};
    int nvals = sizeof(values) / sizeof(values[0]);

    printf("Original: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n");

    array_map(values, nvals, double_value);
    printf("Doubled: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n");

    array_map(values, nvals, square_value);
    printf("Squared: ");
    for (int i = 0; i < nvals; i++) printf("%d ", values[i]);
    printf("\n\n");

    /* Sort with callback */
    int nums[] = {5, 2, 8, 1, 9};
    int nnums = sizeof(nums) / sizeof(nums[0]);

    printf("Original: ");
    for (int i = 0; i < nnums; i++) printf("%d ", nums[i]);
    printf("\n");

    bubble_sort(nums, nnums, compare_asc);
    printf("Sorted ascending: ");
    for (int i = 0; i < nnums; i++) printf("%d ", nums[i]);
    printf("\n");

    bubble_sort(nums, nnums, compare_desc);
    printf("Sorted descending: ");
    for (int i = 0; i < nnums; i++) printf("%d ", nums[i]);
    printf("\n");

    return 0;
}
