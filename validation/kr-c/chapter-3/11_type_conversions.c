/* K&R C Chapter 3: Type Conversions
 * K&R §3.2: Implicit and explicit type conversions
 * Tests type casting and promotion rules
 */

#include <stdio.h>
#include <limits.h>

void demo_implicit_conversions(void) {
    printf("=== Implicit Type Conversions ===\n");

    /* Integer promotion */
    char c = 100;
    short s = 200;
    int result = c + s;
    printf("char(100) + short(200) = int(%d)\n", result);

    /* Float/double promotion */
    float f = 3.14f;
    double d = f * 2.0;  /* f promoted to double */
    printf("float(3.14) * double(2.0) = double(%.2f)\n", d);

    /* Mixed arithmetic */
    int i = 5;
    float f2 = 2.5f;
    float result2 = i * f2;  /* i promoted to float */
    printf("int(5) * float(2.5) = float(%.2f)\n", result2);

    printf("\n");
}

void demo_explicit_casts(void) {
    printf("=== Explicit Type Casts ===\n");

    double d = 3.7;
    int i = (int)d;  /* Truncation */
    printf("(int)3.7 = %d\n", i);

    int num = 7;
    int den = 2;
    double ratio = (double)num / den;
    printf("(double)7 / 2 = %.2f\n", ratio);

    /* Unsigned to signed */
    unsigned int u = UINT_MAX;
    int signed_u = (int)u;
    printf("(int)UINT_MAX = %d\n", signed_u);

    printf("\n");
}

void demo_integer_overflow(void) {
    printf("=== Integer Overflow ===\n");

    signed char c = 127;
    printf("signed char: %d\n", c);
    c = c + 1;
    printf("After +1: %d (overflow)\n", c);

    unsigned char uc = 255;
    printf("\nunsigned char: %u\n", uc);
    uc = uc + 1;
    printf("After +1: %u (wraps to 0)\n", uc);

    printf("\n");
}

void demo_pointer_casts(void) {
    printf("=== Pointer Type Casts ===\n");

    int arr[] = {1, 2, 3, 4};
    int *p = arr;

    /* Pointer to char */
    char *cp = (char*)p;
    printf("int array as bytes: ");
    for (int i = 0; i < (int)sizeof(arr); i++) {
        printf("%02X ", (unsigned char)cp[i]);
    }
    printf("\n");

    /* void pointer */
    void *vp = arr;
    int *back_to_int = (int*)vp;
    printf("Through void*: %d %d %d %d\n",
           back_to_int[0], back_to_int[1], back_to_int[2], back_to_int[3]);

    printf("\n");
}

int main() {
    printf("=== Type Conversions ===\n\n");

    demo_implicit_conversions();
    demo_explicit_casts();
    demo_integer_overflow();
    demo_pointer_casts();

    printf("Conversion rules:\n");
    printf("  - char/short promoted to int\n");
    printf("  - int promoted to long/float/double as needed\n");
    printf("  - Explicit casts override defaults\n");
    printf("  - Integer overflow wraps for unsigned\n");
    printf("  - Integer overflow undefined for signed\n");

    return 0;
}
