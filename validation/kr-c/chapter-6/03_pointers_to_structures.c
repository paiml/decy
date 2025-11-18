/* K&R C Chapter 6.4: Pointers to Structures
 * Page 131-132
 * Pointer to structure and arrow operator
 */

#include <stdio.h>

struct point {
    int x;
    int y;
};

struct rect {
    struct point pt1;
    struct point pt2;
};

int main() {
    struct point pt = {10, 20};
    struct point *pp;

    pp = &pt;

    /* Two ways to access structure members via pointer */
    printf("pt.x = %d, pt.y = %d\n", pt.x, pt.y);
    printf("(*pp).x = %d, (*pp).y = %d\n", (*pp).x, (*pp).y);
    printf("pp->x = %d, pp->y = %d\n", pp->x, pp->y);

    /* Nested structures */
    struct rect screen;
    screen.pt1.x = 0;
    screen.pt1.y = 0;
    screen.pt2.x = 100;
    screen.pt2.y = 100;

    printf("Rectangle: (%d,%d) to (%d,%d)\n",
           screen.pt1.x, screen.pt1.y,
           screen.pt2.x, screen.pt2.y);

    return 0;
}
