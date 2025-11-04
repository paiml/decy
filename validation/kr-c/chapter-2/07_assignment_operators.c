/* K&R C Chapter 2.10: Assignment Operators and Expressions
 * Page 50
 * Compound assignment operators (+=, -=, *=, /=, etc.)
 */

#include <stdio.h>

int main() {
    int n = 10;
    int i;

    printf("Initial value: n = %d\n\n", n);

    /* Arithmetic assignment */
    n += 5;
    printf("n += 5  -> n = %d\n", n);

    n -= 3;
    printf("n -= 3  -> n = %d\n", n);

    n *= 2;
    printf("n *= 2  -> n = %d\n", n);

    n /= 4;
    printf("n /= 4  -> n = %d\n", n);

    n %= 5;
    printf("n %%= 5  -> n = %d\n", n);

    /* Bitwise assignment */
    i = 0x0F;
    printf("\ni = 0x%02X\n", i);

    i &= 0x55;
    printf("i &= 0x55  -> i = 0x%02X\n", i);

    i |= 0xAA;
    printf("i |= 0xAA  -> i = 0x%02X\n", i);

    i ^= 0xFF;
    printf("i ^= 0xFF  -> i = 0x%02X\n", i);

    i <<= 2;
    printf("i <<= 2    -> i = 0x%02X\n", i & 0xFF);

    i >>= 1;
    printf("i >>= 1    -> i = 0x%02X\n", i);

    return 0;
}
