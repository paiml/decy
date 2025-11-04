/* K&R C Chapter 6: Anonymous Struct/Union Members (C11)
 * Nested anonymous structs and unions
 */

#include <stdio.h>

/* Anonymous union in struct */
typedef struct {
    int type;  /* 0 = int, 1 = float */
    union {
        int i;
        float f;
    };  /* Anonymous union */
} Value;

/* Anonymous struct in struct */
typedef struct {
    union {
        struct {
            float x, y;
        };  /* Anonymous struct */
        float coords[2];
    };
} Point2D;

/* Complex example */
typedef struct {
    char name[30];
    union {
        struct {
            int hours;
            int minutes;
            int seconds;
        };  /* Anonymous struct */
        int time_components[3];
    };
} TimeStamp;

int main() {
    printf("=== Anonymous Members ===\n\n");

    /* Value with anonymous union */
    Value v1, v2;

    v1.type = 0;
    v1.i = 42;  /* Access directly, not v1.union.i */

    v2.type = 1;
    v2.f = 3.14;

    printf("Value 1 (type %d): %d\n", v1.type, v1.i);
    printf("Value 2 (type %d): %.2f\n", v2.type, v2.f);

    /* Point with anonymous struct */
    Point2D pt;

    pt.x = 10.0;  /* Access directly, not pt.struct.x */
    pt.y = 20.0;

    printf("\nPoint: (%.1f, %.1f)\n", pt.x, pt.y);
    printf("Using array: (%.1f, %.1f)\n", pt.coords[0], pt.coords[1]);

    /* Modify via array */
    pt.coords[0] = 30.0;
    pt.coords[1] = 40.0;

    printf("After array modification: (%.1f, %.1f)\n", pt.x, pt.y);

    /* TimeStamp */
    TimeStamp ts;
    strcpy(ts.name, "Event1");

    ts.hours = 14;
    ts.minutes = 30;
    ts.seconds = 45;

    printf("\nTimeStamp: %s\n", ts.name);
    printf("  Time: %02d:%02d:%02d\n",
           ts.hours, ts.minutes, ts.seconds);
    printf("  Array: [%d, %d, %d]\n",
           ts.time_components[0],
           ts.time_components[1],
           ts.time_components[2]);

    /* Modify via array */
    ts.time_components[0] = 23;
    printf("  After array modification: %02d:%02d:%02d\n",
           ts.hours, ts.minutes, ts.seconds);

    printf("\nSizes:\n");
    printf("  sizeof(Value) = %zu\n", sizeof(Value));
    printf("  sizeof(Point2D) = %zu\n", sizeof(Point2D));
    printf("  sizeof(TimeStamp) = %zu\n", sizeof(TimeStamp));

    return 0;
}
