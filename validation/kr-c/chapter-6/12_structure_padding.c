/* K&R C Chapter 6: Structure Padding and Alignment
 * Understanding memory layout of structures
 */

#include <stdio.h>
#include <stddef.h>

/* Unoptimized structure - poor alignment */
struct unoptimized {
    char c1;      /* 1 byte */
    int i;        /* 4 bytes (usually 4-byte aligned) */
    char c2;      /* 1 byte */
    double d;     /* 8 bytes (usually 8-byte aligned) */
    char c3;      /* 1 byte */
};

/* Optimized structure - better alignment */
struct optimized {
    double d;     /* 8 bytes */
    int i;        /* 4 bytes */
    char c1;      /* 1 byte */
    char c2;      /* 1 byte */
    char c3;      /* 1 byte */
    /* Compiler adds 1 byte padding here for alignment */
};

/* Packed structure (may be less efficient) */
struct __attribute__((packed)) packed {
    char c1;
    int i;
    char c2;
    double d;
    char c3;
};

/* Demonstrate offsetof macro */
struct example {
    char a;
    int b;
    double c;
    char d;
};

int main() {
    printf("=== Structure Sizes ===\n");
    printf("sizeof(char) = %zu\n", sizeof(char));
    printf("sizeof(int) = %zu\n", sizeof(int));
    printf("sizeof(double) = %zu\n", sizeof(double));
    printf("\n");

    /* Unoptimized structure */
    printf("Unoptimized structure:\n");
    printf("  Expected minimum: %zu bytes (1+4+1+8+1)\n",
           sizeof(char) + sizeof(int) + sizeof(char) + sizeof(double) + sizeof(char));
    printf("  Actual size: %zu bytes\n", sizeof(struct unoptimized));
    printf("  Padding: %zu bytes\n",
           sizeof(struct unoptimized) - 15);

    /* Optimized structure */
    printf("\nOptimized structure:\n");
    printf("  Expected minimum: %zu bytes (8+4+1+1+1)\n",
           sizeof(double) + sizeof(int) + 3 * sizeof(char));
    printf("  Actual size: %zu bytes\n", sizeof(struct optimized));
    printf("  Padding: %zu bytes\n",
           sizeof(struct optimized) - 15);

    /* Packed structure */
    printf("\nPacked structure:\n");
    printf("  Size: %zu bytes (no padding)\n", sizeof(struct packed));

    /* Member offsets */
    printf("\n=== Member Offsets (offsetof) ===\n");
    printf("struct example:\n");
    printf("  offsetof(a) = %zu\n", offsetof(struct example, a));
    printf("  offsetof(b) = %zu\n", offsetof(struct example, b));
    printf("  offsetof(c) = %zu\n", offsetof(struct example, c));
    printf("  offsetof(d) = %zu\n", offsetof(struct example, d));
    printf("  sizeof(example) = %zu\n", sizeof(struct example));

    /* Visualize memory layout */
    struct unoptimized u;
    printf("\n=== Memory Layout (unoptimized) ===\n");
    printf("Base address: %p\n", (void*)&u);
    printf("  c1 at: %p (offset %zu)\n", (void*)&u.c1, (size_t)((char*)&u.c1 - (char*)&u));
    printf("  i  at: %p (offset %zu)\n", (void*)&u.i, (size_t)((char*)&u.i - (char*)&u));
    printf("  c2 at: %p (offset %zu)\n", (void*)&u.c2, (size_t)((char*)&u.c2 - (char*)&u));
    printf("  d  at: %p (offset %zu)\n", (void*)&u.d, (size_t)((char*)&u.d - (char*)&u));
    printf("  c3 at: %p (offset %zu)\n", (void*)&u.c3, (size_t)((char*)&u.c3 - (char*)&u));

    /* Alignment requirements */
    printf("\n=== Alignment Requirements ===\n");
    printf("_Alignof(char) = %zu\n", _Alignof(char));
    printf("_Alignof(int) = %zu\n", _Alignof(int));
    printf("_Alignof(double) = %zu\n", _Alignof(double));
    printf("_Alignof(struct unoptimized) = %zu\n", _Alignof(struct unoptimized));

    return 0;
}
