/* K&R C Chapter 7.5: File Access
 * Page 160-162
 * Basic file operations (fopen, fclose, fprintf, fscanf)
 */

#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *fp;
    int i;

    /* Write to file */
    fp = fopen("numbers.txt", "w");
    if (fp == NULL) {
        fprintf(stderr, "Error: Cannot open file for writing\n");
        return 1;
    }

    for (i = 1; i <= 10; i++)
        fprintf(fp, "%d\n", i * i);

    fclose(fp);

    /* Read from file */
    fp = fopen("numbers.txt", "r");
    if (fp == NULL) {
        fprintf(stderr, "Error: Cannot open file for reading\n");
        return 1;
    }

    printf("Squares from file:\n");
    while (fscanf(fp, "%d", &i) == 1)
        printf("%d\n", i);

    fclose(fp);

    return 0;
}
