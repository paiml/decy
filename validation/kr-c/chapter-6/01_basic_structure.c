/* K&R C Chapter 6.1: Basics of Structures
 * Page 127-128
 * Simple structure definition and usage
 */

#include <stdio.h>

struct point {
    int x;
    int y;
};

int main() {
    struct point pt;

    pt.x = 10;
    pt.y = 20;

    printf("Point: (%d, %d)\n", pt.x, pt.y);

    /* Structure initialization */
    struct point origin = {0, 0};
    printf("Origin: (%d, %d)\n", origin.x, origin.y);

    return 0;
}
