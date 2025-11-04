/* K&R C Chapter 6.8: Unions
 * Page 147-149
 * Union for variant types
 */

#include <stdio.h>

union u_tag {
    int ival;
    float fval;
    char *sval;
};

struct variant {
    enum { INT, FLOAT, STRING } type;
    union {
        int ival;
        float fval;
        char *sval;
    } u;
};

int main() {
    union u_tag uval;
    struct variant v;

    /* Store int */
    uval.ival = 42;
    printf("Integer: %d\n", uval.ival);

    /* Store float (overwrites int) */
    uval.fval = 3.14;
    printf("Float: %.2f\n", uval.fval);

    /* Store string pointer (overwrites float) */
    uval.sval = "hello";
    printf("String: %s\n", uval.sval);

    /* Tagged union */
    v.type = INT;
    v.u.ival = 100;

    if (v.type == INT)
        printf("Variant int: %d\n", v.u.ival);

    return 0;
}
