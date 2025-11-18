/* K&R C Chapter 3.6: Loops - Do-While
 * Page 63
 * Convert integer to string using do-while
 */

#include <stdio.h>
#include <string.h>

/* itoa: convert n to characters in s (do-while version) */
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

    /* Reverse string */
    int j, k;
    char temp;
    for (j = 0, k = i - 1; j < k; j++, k--) {
        temp = s[j];
        s[j] = s[k];
        s[k] = temp;
    }
}

int main() {
    char buffer[100];

    itoa(12345, buffer);
    printf("12345 -> \"%s\"\n", buffer);

    itoa(-6789, buffer);
    printf("-6789 -> \"%s\"\n", buffer);

    itoa(0, buffer);
    printf("0 -> \"%s\"\n", buffer);

    return 0;
}
