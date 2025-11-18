/* K&R C Chapter 5.7: Multi-dimensional Arrays
 * Page 110
 * Two-dimensional array example
 */

#include <stdio.h>

int main() {
    int i, j;
    int matrix[3][4] = {
        {1, 2, 3, 4},
        {5, 6, 7, 8},
        {9, 10, 11, 12}
    };

    printf("Matrix:\n");
    for (i = 0; i < 3; i++) {
        for (j = 0; j < 4; j++)
            printf("%3d ", matrix[i][j]);
        printf("\n");
    }

    return 0;
}
