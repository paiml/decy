/* K&R C Chapter 7.8: Miscellaneous Functions - ungetc
 * Page 166
 * Push back characters with ungetc
 */

#include <stdio.h>
#include <ctype.h>

/* Read a number and push back the first non-digit */
int read_number(void) {
    int c, n = 0;

    /* Skip whitespace */
    while ((c = getchar()) != EOF && isspace(c))
        ;

    /* Read digits */
    if (isdigit(c)) {
        do {
            n = 10 * n + (c - '0');
        } while ((c = getchar()) != EOF && isdigit(c));

        /* Push back the non-digit character */
        ungetc(c, stdin);
    }

    return n;
}

int main() {
    int num;

    printf("Enter a number followed by any character: ");
    num = read_number();

    printf("Number read: %d\n", num);
    printf("Next character: %c\n", getchar());

    return 0;
}
