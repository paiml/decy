/* K&R C Chapter 7.5: File Copying
 * Page 162
 * Copy one file to another
 */

#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
    FILE *in, *out;
    int c;

    if (argc != 3) {
        fprintf(stderr, "Usage: %s <source> <destination>\n", argv[0]);
        exit(1);
    }

    in = fopen(argv[1], "r");
    if (in == NULL) {
        fprintf(stderr, "Error: Cannot open input file '%s'\n", argv[1]);
        exit(2);
    }

    out = fopen(argv[2], "w");
    if (out == NULL) {
        fprintf(stderr, "Error: Cannot open output file '%s'\n", argv[2]);
        fclose(in);
        exit(3);
    }

    /* Copy character by character */
    while ((c = getc(in)) != EOF)
        putc(c, out);

    fclose(in);
    fclose(out);

    printf("File copied successfully\n");

    return 0;
}
