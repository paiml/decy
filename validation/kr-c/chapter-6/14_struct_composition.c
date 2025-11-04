/* K&R C Chapter 6: Structure Composition
 * Building complex structures from simpler ones
 */

#include <stdio.h>
#include <string.h>

typedef struct {
    double x, y;
} Point2D;

typedef struct {
    double x, y, z;
} Point3D;

typedef struct {
    Point2D min;
    Point2D max;
} Rectangle;

typedef struct {
    Point3D position;
    Point3D velocity;
    Point3D acceleration;
} Particle;

typedef struct {
    char name[50];
    Rectangle bounds;
    Point2D center;
} Shape;

double rect_area(Rectangle *r) {
    double width = r->max.x - r->min.x;
    double height = r->max.y - r->min.y;
    return width * height;
}

void rect_center(Rectangle *r, Point2D *center) {
    center->x = (r->min.x + r->max.x) / 2.0;
    center->y = (r->min.y + r->max.y) / 2.0;
}

void particle_update(Particle *p, double dt) {
    /* Update velocity */
    p->velocity.x += p->acceleration.x * dt;
    p->velocity.y += p->acceleration.y * dt;
    p->velocity.z += p->acceleration.z * dt;

    /* Update position */
    p->position.x += p->velocity.x * dt;
    p->position.y += p->velocity.y * dt;
    p->position.z += p->velocity.z * dt;
}

int main() {
    /* Rectangle example */
    Rectangle rect = {
        .min = {0.0, 0.0},
        .max = {10.0, 5.0}
    };

    printf("Rectangle: (%.1f, %.1f) to (%.1f, %.1f)\n",
           rect.min.x, rect.min.y, rect.max.x, rect.max.y);
    printf("Area: %.2f\n", rect_area(&rect));

    Point2D center;
    rect_center(&rect, &center);
    printf("Center: (%.1f, %.1f)\n\n", center.x, center.y);

    /* Particle simulation */
    Particle p = {
        .position = {0.0, 0.0, 0.0},
        .velocity = {1.0, 0.0, 0.0},
        .acceleration = {0.0, -9.8, 0.0}
    };

    printf("Particle simulation (dt=0.1):\n");
    for (int i = 0; i < 5; i++) {
        printf("  t=%.1f: pos=(%.2f, %.2f, %.2f) vel=(%.2f, %.2f, %.2f)\n",
               i * 0.1,
               p.position.x, p.position.y, p.position.z,
               p.velocity.x, p.velocity.y, p.velocity.z);
        particle_update(&p, 0.1);
    }

    /* Composite shape */
    Shape shape = {
        .name = "Box",
        .bounds = {{0.0, 0.0}, {100.0, 50.0}},
        .center = {50.0, 25.0}
    };

    printf("\nShape: %s\n", shape.name);
    printf("  Bounds: (%.1f, %.1f) to (%.1f, %.1f)\n",
           shape.bounds.min.x, shape.bounds.min.y,
           shape.bounds.max.x, shape.bounds.max.y);
    printf("  Center: (%.1f, %.1f)\n",
           shape.center.x, shape.center.y);

    return 0;
}
