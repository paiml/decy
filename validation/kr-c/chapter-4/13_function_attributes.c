/* K&R C Chapter 4: Function Attributes
 * GCC function attributes for optimization and control
 */

#include <stdio.h>
#include <stdlib.h>

/* Noreturn attribute */
__attribute__((noreturn))
void fatal_error(const char *msg) {
    fprintf(stderr, "FATAL: %s\n", msg);
    exit(1);
}

/* Pure function - no side effects, result depends only on args */
__attribute__((pure))
int pure_add(int a, int b) {
    return a + b;
}

/* Const function - even stricter than pure */
__attribute__((const))
int const_multiply(int a, int b) {
    return a * b;
}

/* Warn if return value unused */
__attribute__((warn_unused_result))
int important_function(void) {
    return 42;
}

/* Always inline */
__attribute__((always_inline))
static inline int force_inline(int x) {
    return x * 2;
}

/* Never inline */
__attribute__((noinline))
int never_inline(int x) {
    return x * 3;
}

/* Deprecated function */
__attribute__((deprecated))
void old_function(void) {
    printf("This function is deprecated\n");
}

/* Format checking like printf */
__attribute__((format(printf, 1, 2)))
void my_printf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
}

int main() {
    printf("=== Function Attributes ===\n\n");

    /* Pure and const functions */
    int result1 = pure_add(10, 20);
    int result2 = const_multiply(5, 6);

    printf("pure_add(10, 20) = %d\n", result1);
    printf("const_multiply(5, 6) = %d\n", result2);

    /* Inline attributes */
    int val = force_inline(5);
    printf("force_inline(5) = %d\n", val);

    val = never_inline(7);
    printf("never_inline(7) = %d\n", val);

    /* Important function - should use result */
    int important = important_function();
    printf("important_function() = %d\n", important);

    /* Using deprecated function (may warn) */
    // old_function();  /* Uncomment to see deprecation warning */

    /* Custom printf */
    my_printf("Testing format: %d, %s\n", 123, "hello");

    /* Noreturn example (commented to avoid exit) */
    // fatal_error("Program terminated");

    return 0;
}
