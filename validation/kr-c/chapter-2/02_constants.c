/* K&R C Chapter 2.3: Constants
 * Page 36-37
 * Various constant examples
 */

#include <stdio.h>

int main() {
    int decimal = 100;
    int octal = 0144;      /* 100 in octal */
    int hex = 0x64;        /* 100 in hexadecimal */
    long long_const = 123456789L;
    float float_const = 123.45f;
    double double_const = 1e-2;  /* 0.01 */
    char char_const = 'x';
    char newline = '\n';
    char tab = '\t';

    printf("decimal: %d\n", decimal);
    printf("octal: %d\n", octal);
    printf("hex: %d\n", hex);
    printf("long: %ld\n", long_const);
    printf("float: %f\n", float_const);
    printf("double: %f\n", double_const);
    printf("char: %c\n", char_const);
    return 0;
}
