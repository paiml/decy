/* K&R C Chapter 6: Structure Copy vs Reference
 * Pass by value vs pass by pointer
 */

#include <stdio.h>
#include <string.h>

typedef struct {
    int x, y;
} Point;

typedef struct {
    char name[50];
    int value;
} Config;

/* Pass by value - structure copied */
void modify_by_value(Point p) {
    p.x = 999;
    p.y = 999;
    printf("  Inside modify_by_value: (%d, %d)\n", p.x, p.y);
}

/* Pass by pointer - modifies original */
void modify_by_pointer(Point *p) {
    p->x = 888;
    p->y = 888;
    printf("  Inside modify_by_pointer: (%d, %d)\n", p->x, p->y);
}

/* Return by value - structure copied */
Point create_point(int x, int y) {
    Point p = {x, y};
    return p;
}

/* Large structure example */
typedef struct {
    char buffer[1000];
    int numbers[100];
} LargeStruct;

void process_by_value(LargeStruct s) {
    /* Expensive: entire structure copied */
    printf("  Processing by value (copied %zu bytes)\n", sizeof(s));
}

void process_by_pointer(LargeStruct *s) {
    /* Efficient: only pointer passed */
    printf("  Processing by pointer (passed %zu bytes)\n", sizeof(s));
}

void process_by_const_pointer(const LargeStruct *s) {
    /* Efficient and safe: cannot modify */
    printf("  Processing by const pointer (passed %zu bytes, read-only)\n",
           sizeof(s));
}

int main() {
    printf("=== Structure Copy vs Reference ===\n\n");

    /* Pass by value */
    Point p1 = {10, 20};
    printf("Original: (%d, %d)\n", p1.x, p1.y);
    modify_by_value(p1);
    printf("After modify_by_value: (%d, %d) (unchanged)\n\n",
           p1.x, p1.y);

    /* Pass by pointer */
    printf("Original: (%d, %d)\n", p1.x, p1.y);
    modify_by_pointer(&p1);
    printf("After modify_by_pointer: (%d, %d) (changed)\n\n",
           p1.x, p1.y);

    /* Return by value */
    Point p2 = create_point(100, 200);
    printf("Created point: (%d, %d)\n\n", p2.x, p2.y);

    /* Structure assignment (copy) */
    Point p3 = p2;
    p3.x = 300;
    printf("p2: (%d, %d), p3: (%d, %d) (independent copies)\n\n",
           p2.x, p2.y, p3.x, p3.y);

    /* Large structure performance */
    LargeStruct large;
    memset(&large, 0, sizeof(large));

    printf("Large structure size: %zu bytes\n\n", sizeof(LargeStruct));

    process_by_value(large);
    process_by_pointer(&large);
    process_by_const_pointer(&large);

    /* Array of structures - memory layout */
    Config configs[3] = {
        {"Config1", 100},
        {"Config2", 200},
        {"Config3", 300}
    };

    printf("\nArray of structures:\n");
    printf("  sizeof(Config) = %zu bytes\n", sizeof(Config));
    printf("  sizeof(configs) = %zu bytes\n", sizeof(configs));
    printf("  Addresses:\n");
    for (int i = 0; i < 3; i++) {
        printf("    configs[%d] at %p\n", i, (void*)&configs[i]);
    }

    return 0;
}
