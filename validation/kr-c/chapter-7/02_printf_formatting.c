/* K&R C Chapter 7.2: Formatted Output - printf
 * Page 153-154
 * Various printf format specifiers
 */

#include <stdio.h>

int main() {
    int i = 42;
    float f = 3.14159;
    double d = 2.71828;
    char c = 'A';
    char *s = "Hello, World!";

    /* Integer formats */
    printf("Integer: %d\n", i);
    printf("Hex: %x, %X\n", i, i);
    printf("Octal: %o\n", i);

    /* Float formats */
    printf("Float: %f\n", f);
    printf("Float (precision): %.2f\n", f);
    printf("Scientific: %e\n", f);

    /* Character and string */
    printf("Character: %c\n", c);
    printf("String: %s\n", s);

    /* Width and alignment */
    printf("Right-aligned: %10d\n", i);
    printf("Left-aligned: %-10d|\n", i);
    printf("Zero-padded: %05d\n", i);

    /* Mixed formats */
    printf("%s: %d = 0x%X = %o (octal)\n", "Number", i, i, i);

    return 0;
}
