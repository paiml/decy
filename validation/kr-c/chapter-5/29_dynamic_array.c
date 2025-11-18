/* K&R C Chapter 5: Dynamic Array (Vector)
 * K&R §5.10: Dynamic memory allocation
 * Tests dynamic array implementation with growth strategy
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int *data;
    size_t size;      /* Number of elements */
    size_t capacity;  /* Allocated capacity */
} DynamicArray;

/* Create dynamic array */
DynamicArray *dynarr_create(size_t initial_capacity) {
    DynamicArray *arr = malloc(sizeof(DynamicArray));
    arr->data = malloc(initial_capacity * sizeof(int));
    arr->size = 0;
    arr->capacity = initial_capacity;
    return arr;
}

/* Destroy dynamic array */
void dynarr_destroy(DynamicArray *arr) {
    free(arr->data);
    free(arr);
}

/* Grow array capacity */
void dynarr_grow(DynamicArray *arr) {
    size_t new_capacity = arr->capacity * 2;
    int *new_data = realloc(arr->data, new_capacity * sizeof(int));
    if (new_data == NULL) {
        fprintf(stderr, "Failed to grow array\n");
        return;
    }
    arr->data = new_data;
    arr->capacity = new_capacity;
    printf("  Array grown to capacity %zu\n", new_capacity);
}

/* Push element to end */
void dynarr_push(DynamicArray *arr, int value) {
    if (arr->size >= arr->capacity) {
        dynarr_grow(arr);
    }
    arr->data[arr->size++] = value;
}

/* Pop element from end */
int dynarr_pop(DynamicArray *arr) {
    if (arr->size == 0) {
        fprintf(stderr, "Array is empty\n");
        return -1;
    }
    return arr->data[--arr->size];
}

/* Get element at index */
int dynarr_get(DynamicArray *arr, size_t index) {
    if (index >= arr->size) {
        fprintf(stderr, "Index out of bounds\n");
        return -1;
    }
    return arr->data[index];
}

/* Set element at index */
void dynarr_set(DynamicArray *arr, size_t index, int value) {
    if (index >= arr->size) {
        fprintf(stderr, "Index out of bounds\n");
        return;
    }
    arr->data[index] = value;
}

/* Insert element at index */
void dynarr_insert(DynamicArray *arr, size_t index, int value) {
    if (index > arr->size) {
        fprintf(stderr, "Index out of bounds\n");
        return;
    }

    if (arr->size >= arr->capacity) {
        dynarr_grow(arr);
    }

    /* Shift elements right */
    for (size_t i = arr->size; i > index; i--) {
        arr->data[i] = arr->data[i - 1];
    }

    arr->data[index] = value;
    arr->size++;
}

/* Remove element at index */
void dynarr_remove(DynamicArray *arr, size_t index) {
    if (index >= arr->size) {
        fprintf(stderr, "Index out of bounds\n");
        return;
    }

    /* Shift elements left */
    for (size_t i = index; i < arr->size - 1; i++) {
        arr->data[i] = arr->data[i + 1];
    }

    arr->size--;
}

/* Clear array */
void dynarr_clear(DynamicArray *arr) {
    arr->size = 0;
}

/* Shrink capacity to fit size */
void dynarr_shrink_to_fit(DynamicArray *arr) {
    if (arr->size < arr->capacity) {
        int *new_data = realloc(arr->data, arr->size * sizeof(int));
        if (new_data != NULL) {
            arr->data = new_data;
            arr->capacity = arr->size;
            printf("  Array shrunk to capacity %zu\n", arr->capacity);
        }
    }
}

/* Print array */
void dynarr_print(DynamicArray *arr) {
    printf("[");
    for (size_t i = 0; i < arr->size; i++) {
        printf("%d", arr->data[i]);
        if (i < arr->size - 1) printf(", ");
    }
    printf("] (size=%zu, capacity=%zu)\n", arr->size, arr->capacity);
}

int main() {
    printf("=== Dynamic Array (Vector) ===\n\n");

    /* Create array */
    printf("Creating array with capacity 2:\n");
    DynamicArray *arr = dynarr_create(2);
    dynarr_print(arr);
    printf("\n");

    /* Push elements */
    printf("Pushing elements:\n");
    for (int i = 1; i <= 5; i++) {
        printf("  Push %d: ", i * 10);
        dynarr_push(arr, i * 10);
        dynarr_print(arr);
    }
    printf("\n");

    /* Pop element */
    printf("Pop element: %d\n", dynarr_pop(arr));
    dynarr_print(arr);
    printf("\n");

    /* Get and set */
    printf("Get arr[2]: %d\n", dynarr_get(arr, 2));
    printf("Set arr[2] = 99:\n");
    dynarr_set(arr, 2, 99);
    dynarr_print(arr);
    printf("\n");

    /* Insert */
    printf("Insert 55 at index 1:\n");
    dynarr_insert(arr, 1, 55);
    dynarr_print(arr);
    printf("\n");

    /* Remove */
    printf("Remove element at index 2:\n");
    dynarr_remove(arr, 2);
    dynarr_print(arr);
    printf("\n");

    /* Shrink to fit */
    printf("Shrink to fit:\n");
    dynarr_shrink_to_fit(arr);
    dynarr_print(arr);
    printf("\n");

    /* Clear */
    printf("Clear array:\n");
    dynarr_clear(arr);
    dynarr_print(arr);

    dynarr_destroy(arr);

    printf("\nDynamic array characteristics:\n");
    printf("  - O(1) amortized push\n");
    printf("  - O(1) random access\n");
    printf("  - O(n) insert/remove (middle)\n");
    printf("  - Automatic growth via realloc\n");

    return 0;
}
