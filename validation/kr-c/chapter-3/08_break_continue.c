/* K&R C Chapter 3.7: Break and Continue
 * Page 64-65
 * String trimming with break and continue
 */

#include <stdio.h>
#include <string.h>

/* trim: remove trailing blanks, tabs, newlines */
int trim(char s[]) {
    int n;

    for (n = strlen(s) - 1; n >= 0; n--)
        if (s[n] != ' ' && s[n] != '\t' && s[n] != '\n')
            break;

    s[n + 1] = '\0';
    return n;
}

/* count_nonwhite: count non-whitespace characters using continue */
int count_nonwhite(char s[]) {
    int i, count;

    count = 0;
    for (i = 0; s[i] != '\0'; i++) {
        if (s[i] == ' ' || s[i] == '\t' || s[i] == '\n')
            continue;  /* skip whitespace */
        count++;
    }

    return count;
}

int main() {
    char str1[] = "hello world    \t\n";
    char str2[] = "  test  string  ";

    printf("Before trim: \"%s\" (length=%zu)\n", str1, strlen(str1));
    trim(str1);
    printf("After trim:  \"%s\" (length=%zu)\n", str1, strlen(str1));

    printf("\nNon-whitespace chars in \"%s\": %d\n", str2, count_nonwhite(str2));

    return 0;
}
