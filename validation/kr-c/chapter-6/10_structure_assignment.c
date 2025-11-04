/* K&R C Chapter 6.2: Structure Assignment
 * Page 129
 * Structure assignment and copying
 */

#include <stdio.h>
#include <string.h>

struct point {
    int x;
    int y;
};

struct person {
    char name[50];
    int age;
};

int main() {
    struct point pt1, pt2;
    struct person p1, p2;

    /* Initialize first structure */
    pt1.x = 10;
    pt1.y = 20;

    /* Structure assignment (member-wise copy) */
    pt2 = pt1;

    printf("pt1: (%d, %d)\n", pt1.x, pt1.y);
    printf("pt2: (%d, %d)\n", pt2.x, pt2.y);

    /* Modify copy doesn't affect original */
    pt2.x = 30;
    printf("After pt2.x = 30:\n");
    printf("pt1: (%d, %d)\n", pt1.x, pt1.y);
    printf("pt2: (%d, %d)\n", pt2.x, pt2.y);

    /* Structure with array */
    strcpy(p1.name, "Bob");
    p1.age = 25;

    p2 = p1;  /* Arrays are copied! */

    printf("p1: %s, age %d\n", p1.name, p1.age);
    printf("p2: %s, age %d\n", p2.name, p2.age);

    return 0;
}
