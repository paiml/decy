/**
 * Sprint 19 Feature Validation Test File
 *
 * This file contains examples of all Sprint 19 features to validate
 * the parser works correctly with real-world C patterns.
 */

#include <stdio.h>
#include <stdlib.h>

// ============================================================================
// DECY-058: Global Variables with Storage Class Specifiers
// ============================================================================

// Regular global variable
int global_counter = 0;

// Static global (file-local)
static int file_local_state = 100;

// Extern declaration
extern int external_config;

// Const global
const int MAX_BUFFER_SIZE = 4096;

// Combined: static const
static const char* VERSION = "1.0.0";

// ============================================================================
// DECY-062: Enums with Explicit Values
// ============================================================================

// Enum with explicit values
enum Status {
    STATUS_OK = 0,
    STATUS_ERROR = 1,
    STATUS_PENDING = 2
};

// Enum with hex values (bitmask pattern)
enum FileMode {
    MODE_READ = 0x01,
    MODE_WRITE = 0x02,
    MODE_EXECUTE = 0x04,
    MODE_READ_WRITE = 0x03,
    MODE_ALL = 0x07
};

// Enum with mixed explicit/implicit
enum Priority {
    PRIORITY_LOW = 0,
    PRIORITY_NORMAL,      // Implicit: 1
    PRIORITY_HIGH,        // Implicit: 2
    PRIORITY_CRITICAL = 100
};

// ============================================================================
// DECY-058 + DECY-062: Struct Definition
// ============================================================================

struct Point {
    int x;
    int y;
};

struct Color {
    int r;
    int g;
    int b;
    int a;
};

struct Config {
    int width;
    int height;
    enum Status status;
};

// ============================================================================
// Functions using Sprint 19 features
// ============================================================================

// Function with cast expressions (DECY-059)
int test_cast_expressions() {
    // Float to int cast
    double pi = 3.14159;
    int rounded = (int)pi;

    // Pointer cast
    void* generic_ptr = malloc(100);
    char* str_ptr = (char*)generic_ptr;

    // Size cast
    size_t size = 1024;
    int size_int = (int)size;

    free(generic_ptr);
    return rounded;
}

// Function with compound literals (DECY-060)
struct Point create_point() {
    // Compound literal with positional initializers
    return (struct Point){10, 20};
}

// Function with designated initializers (DECY-061)
struct Color create_color() {
    // Designated initializer in return
    return (struct Color){.r = 255, .g = 128, .b = 64, .a = 255};
}

// Function using global variables (DECY-058)
void increment_global() {
    global_counter++;
    file_local_state--;
}

// Function with compound literal in call
void draw_point(struct Point p) {
    printf("Point: (%d, %d)\n", p.x, p.y);
}

void test_compound_literal_in_call() {
    // Compound literal passed as argument
    draw_point((struct Point){100, 200});
}

// Function with array compound literal
int sum_array() {
    int* arr = (int[]){1, 2, 3, 4, 5};
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum += arr[i];
    }
    return sum;
}

// Function with designated initializers (DECY-061)
struct Config create_config() {
    // Partial initialization with designated initializers
    struct Config cfg = {.width = 800, .height = 600, .status = STATUS_OK};
    return cfg;
}

// Function with out-of-order designated initializers
struct Point create_point_designated() {
    // Out of order is valid in C
    struct Point p = {.y = 50, .x = 25};
    return p;
}

// Function with nested compound literals
struct NestedStruct {
    struct Point position;
    struct Color color;
};

struct NestedStruct create_nested() {
    return (struct NestedStruct){
        .position = (struct Point){0, 0},
        .color = (struct Color){.r = 255, .g = 255, .b = 255, .a = 255}
    };
}

// Function with enum usage
enum Status check_status(int code) {
    if (code == 0) {
        return STATUS_OK;
    } else if (code < 0) {
        return STATUS_ERROR;
    } else {
        return STATUS_PENDING;
    }
}

// Function with bitmask enum
int check_permissions(enum FileMode mode) {
    // Bitmask operations with enum
    if ((mode & MODE_READ) && (mode & MODE_WRITE)) {
        return 1;  // Has both read and write
    }
    return 0;
}

// Main function demonstrating all features
int main() {
    printf("Sprint 19 Feature Validation\n");
    printf("============================\n\n");

    // Test global variables
    printf("Global counter: %d\n", global_counter);
    increment_global();
    printf("After increment: %d\n", global_counter);

    // Test cast expressions
    int result = test_cast_expressions();
    printf("Cast test result: %d\n", result);

    // Test compound literals
    struct Point p1 = create_point();
    printf("Point: (%d, %d)\n", p1.x, p1.y);

    // Test designated initializers
    struct Color c1 = create_color();
    printf("Color: RGBA(%d, %d, %d, %d)\n", c1.r, c1.g, c1.b, c1.a);

    // Test compound literal in call
    test_compound_literal_in_call();

    // Test array compound literal
    int sum = sum_array();
    printf("Array sum: %d\n", sum);

    // Test config creation
    struct Config cfg = create_config();
    printf("Config: %dx%d, status=%d\n", cfg.width, cfg.height, cfg.status);

    // Test enum functions
    enum Status status = check_status(0);
    printf("Status check: %d\n", status);

    // Test bitmask enum
    int has_rw = check_permissions(MODE_READ_WRITE);
    printf("Has read/write: %d\n", has_rw);

    printf("\n✅ All Sprint 19 features tested!\n");
    return 0;
}
