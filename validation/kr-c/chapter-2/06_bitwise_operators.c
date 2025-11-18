/* K&R C Chapter 2.9: Bitwise Operators
 * Page 48-49
 * Bitwise AND, OR, XOR, complement, shift operations
 */

#include <stdio.h>

int main() {
    unsigned int a = 0x0F;  /* 00001111 */
    unsigned int b = 0x55;  /* 01010101 */

    printf("a = 0x%02X (%u)\n", a, a);
    printf("b = 0x%02X (%u)\n", b, b);
    printf("\n");

    /* Bitwise AND */
    printf("a & b  = 0x%02X (%u)\n", a & b, a & b);

    /* Bitwise OR */
    printf("a | b  = 0x%02X (%u)\n", a | b, a | b);

    /* Bitwise XOR */
    printf("a ^ b  = 0x%02X (%u)\n", a ^ b, a ^ b);

    /* Complement */
    printf("~a     = 0x%02X\n", ~a & 0xFF);

    /* Left shift */
    printf("a << 2 = 0x%02X (%u)\n", a << 2, a << 2);

    /* Right shift */
    printf("b >> 2 = 0x%02X (%u)\n", b >> 2, b >> 2);

    return 0;
}
