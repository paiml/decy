/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 99
 * String length using pointer arithmetic
 */

#include <stdio.h>

/* strlen: return length of string s */
int strlen_ptr(char *s) {
    int n;

    for (n = 0; *s != '\0'; s++)
        n++;
    return n;
}

/* Alternative version using pointer subtraction */
int strlen_ptr2(char *s) {
    char *p = s;

    while (*p != '\0')
        p++;
    return p - s;
}

int main() {
    char *str = "hello";

    printf("strlen_ptr(\"%s\") = %d\n", str, strlen_ptr(str));
    printf("strlen_ptr2(\"%s\") = %d\n", str, strlen_ptr2(str));
    return 0;
}
