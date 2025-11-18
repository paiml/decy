/* K&R C Chapter 3.6: Do-While
 * Page 60
 * Do-while loop example
 */

#include <stdio.h>

/* itoa: convert n to characters in s */
void itoa(int n, char s[]) {
    int i, sign;

    if ((sign = n) < 0)
        n = -n;
    i = 0;
    do {
        s[i++] = n % 10 + '0';
    } while ((n /= 10) > 0);
    if (sign < 0)
        s[i++] = '-';
    s[i] = '\0';
}

int main() {
    char s[100];

    itoa(123, s);
    printf("itoa(123) = %s\n", s);

    itoa(-456, s);
    printf("itoa(-456) = %s\n", s);

    return 0;
}
