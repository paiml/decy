/* K&R C Chapter 3.3: Else-If
 * Page 58
 * Binary search using else-if
 */

#include <stdio.h>

/* Binary search: find x in v[0] <= v[1] <= ... <= v[n-1] */
int binsearch(int x, int v[], int n) {
    int low, high, mid;

    low = 0;
    high = n - 1;

    while (low <= high) {
        mid = (low + high) / 2;
        if (x < v[mid])
            high = mid - 1;
        else if (x > v[mid])
            low = mid + 1;
        else
            return mid;  /* found match */
    }

    return -1;  /* no match */
}

int main() {
    int arr[] = {2, 5, 8, 12, 16, 23, 38, 45, 56, 67, 78};
    int n = sizeof(arr) / sizeof(arr[0]);

    printf("Binary search in array: ");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n\n");

    /* Test searches */
    int searches[] = {23, 45, 1, 100, 8};
    for (int i = 0; i < 5; i++) {
        int result = binsearch(searches[i], arr, n);
        if (result >= 0)
            printf("Found %d at index %d\n", searches[i], result);
        else
            printf("%d not found\n", searches[i]);
    }

    return 0;
}
