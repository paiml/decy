/* K&R C Chapter 6.2: Structures and Functions
 * Page 129-130
 * Passing structures to functions
 */

#include <stdio.h>
#include <math.h>

struct point {
    int x;
    int y;
};

/* makepoint: make a point from x and y components */
struct point makepoint(int x, int y) {
    struct point temp;
    temp.x = x;
    temp.y = y;
    return temp;
}

/* addpoint: add two points */
struct point addpoint(struct point p1, struct point p2) {
    p1.x += p2.x;
    p1.y += p2.y;
    return p1;
}

int main() {
    struct point origin, pt1, pt2, result;

    origin = makepoint(0, 0);
    pt1 = makepoint(10, 20);
    pt2 = makepoint(30, 40);

    result = addpoint(pt1, pt2);

    printf("pt1: (%d, %d)\n", pt1.x, pt1.y);
    printf("pt2: (%d, %d)\n", pt2.x, pt2.y);
    printf("sum: (%d, %d)\n", result.x, result.y);

    return 0;
}
