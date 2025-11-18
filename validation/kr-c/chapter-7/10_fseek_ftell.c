/* K&R C Chapter 7.5: Random Access - fseek and ftell
 * Page 163
 * File positioning with fseek and ftell
 */

#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *fp;
    long pos;
    int i;

    /* Create file with numbers */
    fp = fopen("positions.txt", "w");
    if (fp == NULL) {
        fprintf(stderr, "Error creating file\n");
        return 1;
    }

    for (i = 0; i < 10; i++)
        fprintf(fp, "%d\n", i * 10);

    fclose(fp);

    /* Read with random access */
    fp = fopen("positions.txt", "r");
    if (fp == NULL) {
        fprintf(stderr, "Error opening file\n");
        return 1;
    }

    /* Read normally */
    fscanf(fp, "%d", &i);
    printf("First number: %d\n", i);

    /* Save position */
    pos = ftell(fp);
    printf("Current position: %ld\n", pos);

    /* Seek to end */
    fseek(fp, 0L, SEEK_END);
    printf("File size: %ld bytes\n", ftell(fp));

    /* Seek back to saved position */
    fseek(fp, pos, SEEK_SET);
    fscanf(fp, "%d", &i);
    printf("Next number: %d\n", i);

    fclose(fp);

    return 0;
}
