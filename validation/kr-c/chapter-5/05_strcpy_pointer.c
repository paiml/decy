/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 100-101
 * String copy using pointers
 */

#include <stdio.h>

/* strcpy: copy t to s; array subscript version */
void strcpy1(char *s, char *t) {
    int i;

    i = 0;
    while ((s[i] = t[i]) != '\0')
        i++;
}

/* strcpy: copy t to s; pointer version 1 */
void strcpy2(char *s, char *t) {
    while ((*s = *t) != '\0') {
        s++;
        t++;
    }
}

/* strcpy: copy t to s; pointer version 2 */
void strcpy3(char *s, char *t) {
    while ((*s++ = *t++) != '\0')
        ;
}

int main() {
    char s[100];
    char *t = "hello, world";

    strcpy1(s, t);
    printf("strcpy1: %s\n", s);

    strcpy2(s, t);
    printf("strcpy2: %s\n", s);

    strcpy3(s, t);
    printf("strcpy3: %s\n", s);

    return 0;
}
