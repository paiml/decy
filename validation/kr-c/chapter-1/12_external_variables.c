/* K&R C Chapter 1.10: External Variables and Scope
 * Page 31-33
 * Longest line using external variables
 */

#include <stdio.h>

#define MAXLINE 1000

int max;
char line[MAXLINE];
char longest[MAXLINE];

int getline_ext(void);
void copy_ext(void);

int main() {
    int len;
    extern int max;
    extern char longest[];

    max = 0;
    while ((len = getline_ext()) > 0) {
        if (len > max) {
            max = len;
            copy_ext();
        }
    }

    if (max > 0)
        printf("Longest: %s", longest);

    return 0;
}

int getline_ext(void) {
    int c, i;
    extern char line[];

    for (i = 0; i < MAXLINE - 1 && (c = getchar()) != EOF && c != '\n'; ++i)
        line[i] = c;

    if (c == '\n') {
        line[i] = c;
        ++i;
    }

    line[i] = '\0';
    return i;
}

void copy_ext(void) {
    int i;
    extern char line[], longest[];

    i = 0;
    while ((longest[i] = line[i]) != '\0')
        ++i;
}
