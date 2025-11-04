/* K&R C Chapter 6.1: Structure Initialization
 * Page 128-129
 * Various ways to initialize structures
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

struct person {
    char *name;
    int age;
    float height;
};

int main() {
    /* Basic initialization */
    struct point origin = {0, 0};

    /* Partial initialization (rest zeroed) */
    struct point pt1 = {10};

    /* Nested structure initialization */
    struct rect screen = {{0, 0}, {100, 100}};

    /* Named member initialization (C99) */
    struct point pt2 = {.y = 20, .x = 10};

    /* Complex structure */
    struct person p = {"Alice", 30, 5.6};

    printf("Origin: (%d, %d)\n", origin.x, origin.y);
    printf("pt1: (%d, %d)\n", pt1.x, pt1.y);
    printf("Screen: (%d,%d) to (%d,%d)\n",
           screen.pt1.x, screen.pt1.y,
           screen.pt2.x, screen.pt2.y);
    printf("pt2: (%d, %d)\n", pt2.x, pt2.y);
    printf("Person: %s, age %d, height %.1f\n",
           p.name, p.age, p.height);

    return 0;
}
