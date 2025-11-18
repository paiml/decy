/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 101
 * String compare using pointers
 */

#include <stdio.h>

/* strcmp: return <0 if s<t, 0 if s==t, >0 if s>t */
int strcmp_ptr(char *s, char *t) {
    for ( ; *s == *t; s++, t++)
        if (*s == '\0')
            return 0;
    return *s - *t;
}

int main() {
    printf("strcmp(\"abc\", \"abc\") = %d\n", strcmp_ptr("abc", "abc"));
    printf("strcmp(\"abc\", \"def\") = %d\n", strcmp_ptr("abc", "def"));
    printf("strcmp(\"def\", \"abc\") = %d\n", strcmp_ptr("def", "abc"));
    return 0;
}
