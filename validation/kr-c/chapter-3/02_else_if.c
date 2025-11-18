/* K&R C Chapter 3.3: Else-If
 * Page 53-54
 * Binary search example with else-if
 */

#include <stdio.h>

int binsearch(int x, int v[], int n);

int main() {
    int arr[] = {1, 3, 5, 7, 9, 11, 13, 15, 17, 19};
    int n = 10;
    int x = 7;
    int result;

    result = binsearch(x, arr, n);
    printf("binsearch(%d) = %d\n", x, result);
    return 0;
}

/* binsearch: find x in v[0] <= v[1] <= ... <= v[n-1] */
int binsearch(int x, int v[], int n) {
    int low, high, mid;

    low = 0;
    high = n - 1;
    while (low <= high) {
        mid = (low+high) / 2;
        if (x < v[mid])
            high = mid - 1;
        else if (x > v[mid])
            low = mid + 1;
        else
            return mid;
    }
    return -1;
}
