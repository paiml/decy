/* K&R C Chapter 6: Tagged Unions
 * Variant types with type tag
 */

#include <stdio.h>
#include <string.h>

typedef enum {
    TYPE_INT,
    TYPE_FLOAT,
    TYPE_STRING,
    TYPE_POINTER
} ValueType;

typedef struct {
    ValueType type;
    union {
        int i;
        float f;
        char s[50];
        void *p;
    } data;
} Value;

Value make_int(int value) {
    Value v;
    v.type = TYPE_INT;
    v.data.i = value;
    return v;
}

Value make_float(float value) {
    Value v;
    v.type = TYPE_FLOAT;
    v.data.f = value;
    return v;
}

Value make_string(const char *value) {
    Value v;
    v.type = TYPE_STRING;
    strncpy(v.data.s, value, sizeof(v.data.s) - 1);
    v.data.s[sizeof(v.data.s) - 1] = '\0';
    return v;
}

void print_value(Value *v) {
    switch (v->type) {
        case TYPE_INT:
            printf("int: %d", v->data.i);
            break;
        case TYPE_FLOAT:
            printf("float: %.2f", v->data.f);
            break;
        case TYPE_STRING:
            printf("string: \"%s\"", v->data.s);
            break;
        case TYPE_POINTER:
            printf("pointer: %p", v->data.p);
            break;
        default:
            printf("unknown");
    }
}

int value_equals(Value *a, Value *b) {
    if (a->type != b->type)
        return 0;

    switch (a->type) {
        case TYPE_INT:
            return a->data.i == b->data.i;
        case TYPE_FLOAT:
            return a->data.f == b->data.f;
        case TYPE_STRING:
            return strcmp(a->data.s, b->data.s) == 0;
        case TYPE_POINTER:
            return a->data.p == b->data.p;
        default:
            return 0;
    }
}

int main() {
    printf("=== Tagged Union Demo ===\n\n");

    Value values[] = {
        make_int(42),
        make_float(3.14),
        make_string("Hello"),
        make_int(100),
        make_float(2.71)
    };
    int n = sizeof(values) / sizeof(values[0]);

    printf("Values:\n");
    for (int i = 0; i < n; i++) {
        printf("  [%d] ", i);
        print_value(&values[i]);
        printf("\n");
    }

    printf("\nComparisons:\n");
    printf("  values[0] == values[3]: %d\n",
           value_equals(&values[0], &values[3]));
    printf("  values[1] == values[4]: %d\n",
           value_equals(&values[1], &values[4]));

    Value v1 = make_int(42);
    Value v2 = make_int(42);
    printf("  make_int(42) == make_int(42): %d\n",
           value_equals(&v1, &v2));

    printf("\nSize of Value struct: %zu bytes\n", sizeof(Value));

    return 0;
}
