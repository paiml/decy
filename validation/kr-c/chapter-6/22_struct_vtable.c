/* K&R C Chapter 6: Virtual Function Tables (VTables)
 * Simulating polymorphism with function pointers
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Forward declaration */
typedef struct shape Shape;

/* VTable - function pointers for "virtual" methods */
typedef struct {
    float (*area)(Shape *self);
    float (*perimeter)(Shape *self);
    void (*draw)(Shape *self);
} ShapeVTable;

/* Base shape "class" */
struct shape {
    const ShapeVTable *vtable;
    char name[30];
    float x, y;  /* Position */
};

/* Circle "derived class" */
typedef struct {
    Shape base;
    float radius;
} Circle;

float circle_area(Shape *self) {
    Circle *c = (Circle*)self;
    return 3.14159 * c->radius * c->radius;
}

float circle_perimeter(Shape *self) {
    Circle *c = (Circle*)self;
    return 2 * 3.14159 * c->radius;
}

void circle_draw(Shape *self) {
    Circle *c = (Circle*)self;
    printf("  Drawing circle '%s' at (%.1f,%.1f) radius=%.1f\n",
           c->base.name, c->base.x, c->base.y, c->radius);
}

const ShapeVTable circle_vtable = {
    .area = circle_area,
    .perimeter = circle_perimeter,
    .draw = circle_draw
};

/* Rectangle "derived class" */
typedef struct {
    Shape base;
    float width, height;
} Rectangle;

float rect_area(Shape *self) {
    Rectangle *r = (Rectangle*)self;
    return r->width * r->height;
}

float rect_perimeter(Shape *self) {
    Rectangle *r = (Rectangle*)self;
    return 2 * (r->width + r->height);
}

void rect_draw(Shape *self) {
    Rectangle *r = (Rectangle*)self;
    printf("  Drawing rectangle '%s' at (%.1f,%.1f) size=%.1fx%.1f\n",
           r->base.name, r->base.x, r->base.y, r->width, r->height);
}

const ShapeVTable rect_vtable = {
    .area = rect_area,
    .perimeter = rect_perimeter,
    .draw = rect_draw
};

/* Generic shape operations (polymorphic) */
float shape_area(Shape *s) {
    return s->vtable->area(s);
}

float shape_perimeter(Shape *s) {
    return s->vtable->perimeter(s);
}

void shape_draw(Shape *s) {
    s->vtable->draw(s);
}

/* Constructors */
Circle *circle_create(const char *name, float x, float y, float radius) {
    Circle *c = malloc(sizeof(Circle));
    c->base.vtable = &circle_vtable;
    strncpy(c->base.name, name, sizeof(c->base.name) - 1);
    c->base.x = x;
    c->base.y = y;
    c->radius = radius;
    return c;
}

Rectangle *rect_create(const char *name, float x, float y,
                      float width, float height) {
    Rectangle *r = malloc(sizeof(Rectangle));
    r->base.vtable = &rect_vtable;
    strncpy(r->base.name, name, sizeof(r->base.name) - 1);
    r->base.x = x;
    r->base.y = y;
    r->width = width;
    r->height = height;
    return r;
}

int main() {
    printf("=== VTable Polymorphism ===\n\n");

    /* Create shapes */
    Circle *c1 = circle_create("Circle1", 0, 0, 5.0);
    Circle *c2 = circle_create("Circle2", 10, 10, 3.0);
    Rectangle *r1 = rect_create("Rect1", 5, 5, 10.0, 20.0);
    Rectangle *r2 = rect_create("Rect2", 15, 15, 8.0, 12.0);

    /* Polymorphic array */
    Shape *shapes[] = {
        (Shape*)c1, (Shape*)c2, (Shape*)r1, (Shape*)r2
    };
    int n = sizeof(shapes) / sizeof(shapes[0]);

    /* Call virtual functions polymorphically */
    printf("Shape operations:\n");
    for (int i = 0; i < n; i++) {
        shape_draw(shapes[i]);
        printf("    Area: %.2f, Perimeter: %.2f\n",
               shape_area(shapes[i]), shape_perimeter(shapes[i]));
    }

    /* Calculate total area */
    float total_area = 0.0;
    for (int i = 0; i < n; i++)
        total_area += shape_area(shapes[i]);
    printf("\nTotal area: %.2f\n", total_area);

    /* Cleanup */
    free(c1);
    free(c2);
    free(r1);
    free(r2);

    return 0;
}
