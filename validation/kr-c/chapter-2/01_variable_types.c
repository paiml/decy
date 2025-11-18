/* K&R C Chapter 2.2: Data Types and Sizes
 * Page 35-36
 * Basic data type examples
 */

#include <stdio.h>

int main() {
    char c;
    int i;
    long l;
    float f;
    double d;

    c = 'A';
    i = 42;
    l = 1000000L;
    f = 3.14f;
    d = 2.71828;

    printf("char: %c\n", c);
    printf("int: %d\n", i);
    printf("long: %ld\n", l);
    printf("float: %f\n", f);
    printf("double: %f\n", d);
    return 0;
}
