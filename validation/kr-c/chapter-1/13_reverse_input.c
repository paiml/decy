/* K&R C Chapter 1 Exercise: Reverse input
 * Based on Chapter 1 concepts
 * Read input and print lines in reverse
 */

#include <stdio.h>

#define MAXLINE 1000

int getline_kr(char line[], int max);
void reverse(char s[]);

int main() {
    char line[MAXLINE];

    while (getline_kr(line, MAXLINE) > 0) {
        reverse(line);
        printf("%s", line);
    }

    return 0;
}

int getline_kr(char s[], int lim) {
    int c, i;

    for (i = 0; i < lim - 1 && (c = getchar()) != EOF && c != '\n'; ++i)
        s[i] = c;

    if (c == '\n') {
        s[i] = c;
        ++i;
    }

    s[i] = '\0';
    return i;
}

void reverse(char s[]) {
    int i, j;
    char temp;

    /* Find length */
    i = 0;
    while (s[i] != '\0')
        i++;

    /* Don't reverse newline */
    if (i > 0 && s[i - 1] == '\n')
        i--;

    /* Reverse */
    for (j = 0; j < i / 2; j++) {
        temp = s[j];
        s[j] = s[i - 1 - j];
        s[i - 1 - j] = temp;
    }
}
