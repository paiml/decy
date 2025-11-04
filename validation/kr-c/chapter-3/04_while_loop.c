/* K&R C Chapter 3.5: While and For Loops
 * Page 57-58
 * While loop example - string trim
 */

#include <stdio.h>
#include <string.h>

int trim(char s[]);

int main() {
    char str[] = "hello world   \n";
    int n;

    printf("Before: '%s'\n", str);
    n = trim(str);
    printf("After: '%s' (removed %d chars)\n", str, n);
    return 0;
}

/* trim: remove trailing blanks, tabs, newlines */
int trim(char s[]) {
    int n;

    for (n = strlen(s)-1; n >= 0; n--)
        if (s[n] != ' ' && s[n] != '\t' && s[n] != '\n')
            break;
    s[n+1] = '\0';
    return n;
}
