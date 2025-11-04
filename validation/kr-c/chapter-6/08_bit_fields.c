/* K&R C Chapter 6.9: Bit-fields
 * Page 149-150
 * Bit-level structure packing
 */

#include <stdio.h>

struct flags {
    unsigned int is_keyword : 1;
    unsigned int is_extern : 1;
    unsigned int is_static : 1;
};

struct packed {
    unsigned int opcode : 8;
    unsigned int arg1 : 4;
    unsigned int arg2 : 4;
};

int main() {
    struct flags f;
    struct packed p;

    /* Set individual bits */
    f.is_keyword = 1;
    f.is_extern = 0;
    f.is_static = 1;

    printf("Flags: keyword=%u extern=%u static=%u\n",
           f.is_keyword, f.is_extern, f.is_static);

    /* Packed fields */
    p.opcode = 0x42;
    p.arg1 = 0xA;
    p.arg2 = 0xB;

    printf("Packed: opcode=0x%02X arg1=0x%X arg2=0x%X\n",
           p.opcode, p.arg1, p.arg2);

    printf("Size of flags: %zu bytes\n", sizeof(f));
    printf("Size of packed: %zu bytes\n", sizeof(p));

    return 0;
}
