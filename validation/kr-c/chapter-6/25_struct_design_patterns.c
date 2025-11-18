/* K&R C Chapter 6: Structure Design Patterns
 * K&R §6: Advanced structure usage patterns
 * Tests factory, builder, and other design patterns
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ========== Factory Pattern ========== */

typedef enum {
    SHAPE_CIRCLE,
    SHAPE_RECTANGLE,
    SHAPE_TRIANGLE
} ShapeType;

typedef struct {
    ShapeType type;
    float (*area)(void *self);
    void (*destroy)(void *self);
} Shape;

typedef struct {
    Shape base;
    float radius;
} Circle;

typedef struct {
    Shape base;
    float width;
    float height;
} Rectangle;

float circle_area(void *self) {
    Circle *c = (Circle*)self;
    return 3.14159 * c->radius * c->radius;
}

float rectangle_area(void *self) {
    Rectangle *r = (Rectangle*)self;
    return r->width * r->height;
}

void shape_destroy(void *self) {
    free(self);
}

/* Factory function */
Shape *shape_factory(ShapeType type, ...) {
    va_list args;
    va_start(args, type);

    Shape *shape = NULL;

    switch (type) {
        case SHAPE_CIRCLE: {
            Circle *c = malloc(sizeof(Circle));
            c->base.type = SHAPE_CIRCLE;
            c->base.area = circle_area;
            c->base.destroy = shape_destroy;
            c->radius = va_arg(args, double);
            shape = (Shape*)c;
            break;
        }
        case SHAPE_RECTANGLE: {
            Rectangle *r = malloc(sizeof(Rectangle));
            r->base.type = SHAPE_RECTANGLE;
            r->base.area = rectangle_area;
            r->base.destroy = shape_destroy;
            r->width = va_arg(args, double);
            r->height = va_arg(args, double);
            shape = (Shape*)r;
            break;
        }
        default:
            break;
    }

    va_end(args);
    return shape;
}

void factory_pattern_demo(void) {
    printf("=== Factory Pattern ===\n");

    Shape *circle = shape_factory(SHAPE_CIRCLE, 5.0);
    Shape *rectangle = shape_factory(SHAPE_RECTANGLE, 4.0, 6.0);

    printf("Circle area:    %.2f\n", circle->area(circle));
    printf("Rectangle area: %.2f\n", rectangle->area(rectangle));

    circle->destroy(circle);
    rectangle->destroy(rectangle);
    printf("\n");
}

/* ========== Builder Pattern ========== */

typedef struct {
    char name[50];
    int age;
    char email[100];
    char phone[20];
    char address[200];
} Person;

typedef struct {
    Person person;
} PersonBuilder;

PersonBuilder *person_builder_create(void) {
    PersonBuilder *builder = malloc(sizeof(PersonBuilder));
    memset(&builder->person, 0, sizeof(Person));
    return builder;
}

PersonBuilder *person_builder_set_name(PersonBuilder *b, const char *name) {
    strncpy(b->person.name, name, sizeof(b->person.name) - 1);
    return b;
}

PersonBuilder *person_builder_set_age(PersonBuilder *b, int age) {
    b->person.age = age;
    return b;
}

PersonBuilder *person_builder_set_email(PersonBuilder *b, const char *email) {
    strncpy(b->person.email, email, sizeof(b->person.email) - 1);
    return b;
}

PersonBuilder *person_builder_set_phone(PersonBuilder *b, const char *phone) {
    strncpy(b->person.phone, phone, sizeof(b->person.phone) - 1);
    return b;
}

Person *person_builder_build(PersonBuilder *b) {
    Person *person = malloc(sizeof(Person));
    *person = b->person;
    free(b);
    return person;
}

void builder_pattern_demo(void) {
    printf("=== Builder Pattern ===\n");

    Person *person = person_builder_set_name(
                        person_builder_set_age(
                            person_builder_set_email(
                                person_builder_create(),
                                "alice@example.com"),
                            30),
                        "Alice");

    printf("Person:\n");
    printf("  Name:  %s\n", person->name);
    printf("  Age:   %d\n", person->age);
    printf("  Email: %s\n", person->email);

    free(person);
    printf("\n");
}

/* ========== Singleton Pattern ========== */

typedef struct {
    int value;
    char message[100];
} Config;

static Config *singleton_instance = NULL;

Config *config_get_instance(void) {
    if (singleton_instance == NULL) {
        singleton_instance = malloc(sizeof(Config));
        singleton_instance->value = 42;
        strcpy(singleton_instance->message, "Default config");
    }
    return singleton_instance;
}

void singleton_pattern_demo(void) {
    printf("=== Singleton Pattern ===\n");

    Config *config1 = config_get_instance();
    Config *config2 = config_get_instance();

    printf("config1 address: %p\n", (void*)config1);
    printf("config2 address: %p\n", (void*)config2);
    printf("Same instance: %s\n", (config1 == config2) ? "Yes" : "No");
    printf("Config value: %d\n", config1->value);
    printf("\n");
}

/* ========== Observer Pattern ========== */

typedef struct observer Observer;

typedef void (*ObserverCallback)(Observer *observer, void *data);

struct observer {
    ObserverCallback callback;
    void *user_data;
    Observer *next;
};

typedef struct {
    Observer *observers;
    int value;
} Subject;

void subject_attach(Subject *subject, Observer *observer) {
    observer->next = subject->observers;
    subject->observers = observer;
}

void subject_notify(Subject *subject) {
    Observer *obs = subject->observers;
    while (obs != NULL) {
        obs->callback(obs, &subject->value);
        obs = obs->next;
    }
}

void subject_set_value(Subject *subject, int value) {
    subject->value = value;
    subject_notify(subject);
}

void observer_callback_print(Observer *obs, void *data) {
    int value = *(int*)data;
    printf("  Observer notified: value = %d\n", value);
}

void observer_pattern_demo(void) {
    printf("=== Observer Pattern ===\n");

    Subject subject = {NULL, 0};

    Observer obs1 = {observer_callback_print, NULL, NULL};
    Observer obs2 = {observer_callback_print, NULL, NULL};

    subject_attach(&subject, &obs1);
    subject_attach(&subject, &obs2);

    printf("Setting value to 10:\n");
    subject_set_value(&subject, 10);

    printf("Setting value to 20:\n");
    subject_set_value(&subject, 20);

    printf("\n");
}

/* ========== Iterator Pattern ========== */

typedef struct {
    int *data;
    int size;
    int current;
} Iterator;

Iterator *iterator_create(int *data, int size) {
    Iterator *it = malloc(sizeof(Iterator));
    it->data = data;
    it->size = size;
    it->current = 0;
    return it;
}

int iterator_has_next(Iterator *it) {
    return it->current < it->size;
}

int iterator_next(Iterator *it) {
    return it->data[it->current++];
}

void iterator_pattern_demo(void) {
    printf("=== Iterator Pattern ===\n");

    int data[] = {10, 20, 30, 40, 50};
    int size = sizeof(data) / sizeof(data[0]);

    Iterator *it = iterator_create(data, size);

    printf("Iterating through data:\n");
    while (iterator_has_next(it)) {
        printf("  %d\n", iterator_next(it));
    }

    free(it);
    printf("\n");
}

/* ========== Strategy Pattern ========== */

typedef int (*SortStrategy)(int a, int b);

int sort_ascending(int a, int b) {
    return a - b;
}

int sort_descending(int a, int b) {
    return b - a;
}

void sort_array(int *arr, int size, SortStrategy strategy) {
    for (int i = 0; i < size - 1; i++) {
        for (int j = 0; j < size - i - 1; j++) {
            if (strategy(arr[j], arr[j + 1]) > 0) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

void strategy_pattern_demo(void) {
    printf("=== Strategy Pattern ===\n");

    int data[] = {5, 2, 8, 1, 9};
    int size = sizeof(data) / sizeof(data[0]);

    printf("Original: ");
    for (int i = 0; i < size; i++) printf("%d ", data[i]);
    printf("\n");

    sort_array(data, size, sort_ascending);
    printf("Ascending: ");
    for (int i = 0; i < size; i++) printf("%d ", data[i]);
    printf("\n");

    sort_array(data, size, sort_descending);
    printf("Descending: ");
    for (int i = 0; i < size; i++) printf("%d ", data[i]);
    printf("\n\n");
}

/* ========== Composite Pattern ========== */

typedef struct component Component;

typedef int (*ComponentOperation)(Component *self);

struct component {
    char name[50];
    ComponentOperation operation;
    Component **children;
    int child_count;
};

int leaf_operation(Component *self) {
    return 1;
}

int composite_operation(Component *self) {
    int total = 0;
    for (int i = 0; i < self->child_count; i++) {
        total += self->children[i]->operation(self->children[i]);
    }
    return total;
}

void composite_pattern_demo(void) {
    printf("=== Composite Pattern ===\n");

    Component leaf1 = {"Leaf1", leaf_operation, NULL, 0};
    Component leaf2 = {"Leaf2", leaf_operation, NULL, 0};
    Component leaf3 = {"Leaf3", leaf_operation, NULL, 0};

    Component *children[] = {&leaf1, &leaf2, &leaf3};
    Component composite = {"Composite", composite_operation, children, 3};

    printf("Leaf operation:      %d\n", leaf1.operation(&leaf1));
    printf("Composite operation: %d\n", composite.operation(&composite));
    printf("\n");
}

int main() {
    printf("=== Structure Design Patterns ===\n\n");

    factory_pattern_demo();
    builder_pattern_demo();
    singleton_pattern_demo();
    observer_pattern_demo();
    iterator_pattern_demo();
    strategy_pattern_demo();
    composite_pattern_demo();

    printf("Design patterns in C:\n");
    printf("  - Factory:    Object creation abstraction\n");
    printf("  - Builder:    Step-by-step object construction\n");
    printf("  - Singleton:  Single instance guarantee\n");
    printf("  - Observer:   Publish-subscribe notifications\n");
    printf("  - Iterator:   Sequential access abstraction\n");
    printf("  - Strategy:   Interchangeable algorithms\n");
    printf("  - Composite:  Tree structure handling\n");

    return 0;
}
