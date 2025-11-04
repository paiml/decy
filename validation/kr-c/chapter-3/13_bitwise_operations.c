/* K&R C Chapter 3: Bitwise Operations
 * K&R §3.4: Bit manipulation operators
 * Tests &, |, ^, ~, <<, >>
 */

#include <stdio.h>

void print_binary(unsigned int n) {
    for (int i = 31; i >= 0; i--) {
        printf("%d", (n >> i) & 1);
        if (i % 4 == 0) printf(" ");
    }
}

void demo_bitwise_and(void) {
    printf("=== Bitwise AND (&) ===\n");
    unsigned int a = 0b11110000;
    unsigned int b = 0b10101010;
    unsigned int result = a & b;

    printf("a:      ");
    print_binary(a);
    printf("\nb:      ");
    print_binary(b);
    printf("\na & b:  ");
    print_binary(result);
    printf("\n\n");
}

void demo_bitwise_or(void) {
    printf("=== Bitwise OR (|) ===\n");
    unsigned int a = 0b11110000;
    unsigned int b = 0b10101010;
    unsigned int result = a | b;

    printf("a:      ");
    print_binary(a);
    printf("\nb:      ");
    print_binary(b);
    printf("\na | b:  ");
    print_binary(result);
    printf("\n\n");
}

void demo_bitwise_xor(void) {
    printf("=== Bitwise XOR (^) ===\n");
    unsigned int a = 0b11110000;
    unsigned int b = 0b10101010;
    unsigned int result = a ^ b;

    printf("a:      ");
    print_binary(a);
    printf("\nb:      ");
    print_binary(b);
    printf("\na ^ b:  ");
    print_binary(result);
    printf("\n\n");
}

void demo_bitwise_not(void) {
    printf("=== Bitwise NOT (~) ===\n");
    unsigned char a = 0b11110000;
    unsigned char result = ~a;

    printf("a:    %02X (binary: %08b)\n", a, a);
    printf("~a:   %02X (binary: %08b)\n", result, result);
    printf("\n");
}

void demo_left_shift(void) {
    printf("=== Left Shift (<<) ===\n");
    unsigned int a = 0b00000101;

    printf("a:       %u\n", a);
    printf("a << 1:  %u (multiply by 2)\n", a << 1);
    printf("a << 2:  %u (multiply by 4)\n", a << 2);
    printf("a << 3:  %u (multiply by 8)\n", a << 3);
    printf("\n");
}

void demo_right_shift(void) {
    printf("=== Right Shift (>>) ===\n");
    unsigned int a = 40;

    printf("a:       %u\n", a);
    printf("a >> 1:  %u (divide by 2)\n", a >> 1);
    printf("a >> 2:  %u (divide by 4)\n", a >> 2);
    printf("a >> 3:  %u (divide by 8)\n", a >> 3);
    printf("\n");
}

void demo_bit_masking(void) {
    printf("=== Bit Masking Examples ===\n");

    unsigned int flags = 0;

    /* Set bit */
    flags |= (1 << 2);  /* Set bit 2 */
    printf("Set bit 2:     0x%02X\n", flags);

    /* Clear bit */
    flags &= ~(1 << 1);  /* Clear bit 1 */
    printf("Clear bit 1:   0x%02X\n", flags);

    /* Toggle bit */
    flags ^= (1 << 3);  /* Toggle bit 3 */
    printf("Toggle bit 3:  0x%02X\n", flags);

    /* Check bit */
    int is_set = (flags & (1 << 2)) != 0;
    printf("Bit 2 is set:  %s\n", is_set ? "Yes" : "No");

    printf("\n");
}

int main() {
    printf("=== Bitwise Operations ===\n\n");

    demo_bitwise_and();
    demo_bitwise_or();
    demo_bitwise_xor();
    demo_bitwise_not();
    demo_left_shift();
    demo_right_shift();
    demo_bit_masking();

    printf("Bitwise operators:\n");
    printf("  &   AND\n");
    printf("  |   OR\n");
    printf("  ^   XOR\n");
    printf("  ~   NOT (complement)\n");
    printf("  <<  Left shift\n");
    printf("  >>  Right shift\n");

    return 0;
}
