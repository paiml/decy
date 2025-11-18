/* K&R C Chapter 2.2: Data Types - Signed and Unsigned
 * Page 37-38
 * Signed and unsigned integer types
 */

#include <stdio.h>
#include <limits.h>

int main() {
    /* Signed integers */
    signed char sc = -128;
    signed short ss = -32768;
    signed int si = -2147483648;

    printf("Signed types:\n");
    printf("  signed char:  %d\n", sc);
    printf("  signed short: %d\n", ss);
    printf("  signed int:   %d\n", si);

    /* Unsigned integers */
    unsigned char uc = 255;
    unsigned short us = 65535;
    unsigned int ui = 4294967295U;

    printf("\nUnsigned types:\n");
    printf("  unsigned char:  %u\n", uc);
    printf("  unsigned short: %u\n", us);
    printf("  unsigned int:   %u\n", ui);

    /* Limits */
    printf("\nType limits:\n");
    printf("  CHAR_MIN = %d, CHAR_MAX = %d\n", CHAR_MIN, CHAR_MAX);
    printf("  SHRT_MIN = %d, SHRT_MAX = %d\n", SHRT_MIN, SHRT_MAX);
    printf("  INT_MIN = %d, INT_MAX = %d\n", INT_MIN, INT_MAX);
    printf("  UINT_MAX = %u\n", UINT_MAX);

    /* Overflow behavior */
    printf("\nOverflow:\n");
    unsigned char uc_max = 255;
    printf("  uc_max = %u\n", uc_max);
    printf("  uc_max + 1 = %u (wraps to 0)\n", (unsigned char)(uc_max + 1));

    unsigned char uc_min = 0;
    printf("  uc_min = %u\n", uc_min);
    printf("  uc_min - 1 = %u (wraps to 255)\n", (unsigned char)(uc_min - 1));

    /* Mixed signed/unsigned */
    printf("\nMixed signed/unsigned:\n");
    signed int s = -1;
    unsigned int u = 1;

    printf("  signed -1 = %d\n", s);
    printf("  unsigned 1 = %u\n", u);

    if (s < u)
        printf("  -1 < 1 (with mixing)\n");
    else
        printf("  -1 >= 1 (unexpected! -1 becomes large unsigned)\n");

    /* Explicit comparison */
    if ((int)s < (int)u)
        printf("  (int)-1 < (int)1 (explicit cast)\n");

    /* Hexadecimal notation */
    printf("\nHexadecimal:\n");
    unsigned int hex1 = 0xFF;
    unsigned int hex2 = 0xDEADBEEF;
    printf("  0xFF = %u (%d)\n", hex1, hex1);
    printf("  0xDEADBEEF = %u\n", hex2);

    return 0;
}
