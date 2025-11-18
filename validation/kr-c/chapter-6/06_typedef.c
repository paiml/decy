/* K&R C Chapter 6.7: Typedef
 * Page 146-147
 * Type aliases with typedef
 */

#include <stdio.h>

/* Simple typedef */
typedef int Length;

/* Structure typedef */
typedef struct {
    int x;
    int y;
} Point;

/* Pointer typedef */
typedef Point *PointPtr;

/* Function pointer typedef */
typedef int (*CompareFunc)(const void *, const void *);

int main() {
    Length width, height;
    Point origin, pt;
    PointPtr pp;

    width = 100;
    height = 200;

    origin.x = 0;
    origin.y = 0;

    pt.x = 10;
    pt.y = 20;

    pp = &pt;

    printf("Width: %d, Height: %d\n", width, height);
    printf("Origin: (%d, %d)\n", origin.x, origin.y);
    printf("Point via pointer: (%d, %d)\n", pp->x, pp->y);

    return 0;
}
