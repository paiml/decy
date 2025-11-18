/* K&R C Chapter 3: sizeof Operator
 * K&R §3.3: Determining type and object sizes
 * Tests sizeof with various types
 */

#include <stdio.h>
#include <stdint.h>

struct example {
    char c;
    int i;
    double d;
};

void basic_types(void) {
    printf("=== Basic Type Sizes ===\n");
    printf("char:      %zu bytes\n", sizeof(char));
    printf("short:     %zu bytes\n", sizeof(short));
    printf("int:       %zu bytes\n", sizeof(int));
    printf("long:      %zu bytes\n", sizeof(long));
    printf("long long: %zu bytes\n", sizeof(long long));
    printf("float:     %zu bytes\n", sizeof(float));
    printf("double:    %zu bytes\n", sizeof(double));
    printf("pointer:   %zu bytes\n", sizeof(void*));
    printf("\n");
}

void array_sizes(void) {
    printf("=== Array Sizes ===\n");
    int arr[10];
    printf("int arr[10]: %zu bytes\n", sizeof(arr));
    printf("Element count: %zu\n", sizeof(arr) / sizeof(arr[0]));

    char str[] = "Hello";
    printf("char str[] = \"Hello\": %zu bytes (includes \\0)\n", sizeof(str));
    printf("\n");
}

void struct_sizes(void) {
    printf("=== Structure Sizes ===\n");
    printf("struct example: %zu bytes\n", sizeof(struct example));
    printf("  char c:   %zu bytes\n", sizeof(((struct example*)0)->c));
    printf("  int i:    %zu bytes\n", sizeof(((struct example*)0)->i));
    printf("  double d: %zu bytes\n", sizeof(((struct example*)0)->d));
    printf("\n");
}

int main() {
    printf("=== sizeof Operator ===\n\n");

    basic_types();
    array_sizes();
    struct_sizes();

    printf("sizeof is:\n");
    printf("  - Compile-time operator\n");
    printf("  - Returns size_t (unsigned)\n");
    printf("  - Works on types and variables\n");
    printf("  - Accounts for padding in structs\n");

    return 0;
}
