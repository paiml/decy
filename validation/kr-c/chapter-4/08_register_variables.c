/* K&R C Chapter 4.6: Static Variables
 * Page 83-84
 * Register variables for performance
 */

#include <stdio.h>
#include <time.h>

/* sum_array_normal: sum array without register hint */
long sum_array_normal(int arr[], int n) {
    long sum = 0;
    int i;

    for (i = 0; i < n; i++)
        sum += arr[i];

    return sum;
}

/* sum_array_register: sum array with register hint */
long sum_array_register(int arr[], int n) {
    register long sum = 0;
    register int i;

    for (i = 0; i < n; i++)
        sum += arr[i];

    return sum;
}

int main() {
    int size = 10000000;
    int *arr = malloc(size * sizeof(int));

    /* Initialize array */
    for (int i = 0; i < size; i++)
        arr[i] = i % 100;

    printf("Array size: %d elements\n\n", size);

    /* Normal version */
    clock_t start = clock();
    long result1 = sum_array_normal(arr, size);
    clock_t end = clock();
    double time1 = ((double)(end - start)) / CLOCKS_PER_SEC;
    printf("Normal:   sum = %ld, time = %.6f seconds\n", result1, time1);

    /* Register version */
    start = clock();
    long result2 = sum_array_register(arr, size);
    end = clock();
    double time2 = ((double)(end - start)) / CLOCKS_PER_SEC;
    printf("Register: sum = %ld, time = %.6f seconds\n", result2, time2);

    free(arr);

    return 0;
}
