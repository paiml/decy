/* K&R C Chapter 3.8: Goto and Labels
 * Page 65-66
 * Error handling with goto
 */

#include <stdio.h>
#include <stdlib.h>

int process_data(int *data, int size) {
    int *temp1 = NULL;
    int *temp2 = NULL;
    int result = -1;

    /* Allocate resources */
    temp1 = (int *)malloc(size * sizeof(int));
    if (temp1 == NULL) {
        printf("Error: malloc failed for temp1\n");
        goto cleanup;
    }

    temp2 = (int *)malloc(size * sizeof(int));
    if (temp2 == NULL) {
        printf("Error: malloc failed for temp2\n");
        goto cleanup;
    }

    /* Process data */
    for (int i = 0; i < size; i++) {
        temp1[i] = data[i] * 2;
        temp2[i] = data[i] + 10;
    }

    /* Success */
    result = 0;
    printf("Data processed successfully\n");

cleanup:
    /* Cleanup - always executed */
    if (temp1 != NULL) {
        free(temp1);
        printf("Freed temp1\n");
    }
    if (temp2 != NULL) {
        free(temp2);
        printf("Freed temp2\n");
    }

    return result;
}

int main() {
    int data[] = {1, 2, 3, 4, 5};
    int size = sizeof(data) / sizeof(data[0]);

    printf("Processing with sufficient memory:\n");
    int result = process_data(data, size);
    printf("Result: %d\n\n", result);

    return 0;
}
