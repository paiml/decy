/* K&R C Chapter 5: Circular Buffer (Ring Buffer)
 * K&R §5.10: Efficient fixed-size buffer
 * Tests circular buffer for streaming data
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int *data;
    size_t capacity;
    size_t head;   /* Write position */
    size_t tail;   /* Read position */
    size_t count;  /* Number of elements */
} CircularBuffer;

/* Create buffer */
CircularBuffer *cb_create(size_t capacity) {
    CircularBuffer *cb = malloc(sizeof(CircularBuffer));
    cb->data = malloc(capacity * sizeof(int));
    cb->capacity = capacity;
    cb->head = 0;
    cb->tail = 0;
    cb->count = 0;
    return cb;
}

/* Check if full */
int cb_is_full(CircularBuffer *cb) {
    return cb->count == cb->capacity;
}

/* Check if empty */
int cb_is_empty(CircularBuffer *cb) {
    return cb->count == 0;
}

/* Push element */
int cb_push(CircularBuffer *cb, int value) {
    if (cb_is_full(cb)) {
        printf("  Buffer full, overwriting oldest\n");
        cb->tail = (cb->tail + 1) % cb->capacity;
    } else {
        cb->count++;
    }

    cb->data[cb->head] = value;
    cb->head = (cb->head + 1) % cb->capacity;
    return 0;
}

/* Pop element */
int cb_pop(CircularBuffer *cb, int *value) {
    if (cb_is_empty(cb)) {
        return -1;
    }

    *value = cb->data[cb->tail];
    cb->tail = (cb->tail + 1) % cb->capacity;
    cb->count--;
    return 0;
}

/* Peek element */
int cb_peek(CircularBuffer *cb) {
    if (cb_is_empty(cb)) return -1;
    return cb->data[cb->tail];
}

/* Print buffer */
void cb_print(CircularBuffer *cb) {
    printf("[");
    for (size_t i = 0; i < cb->count; i++) {
        size_t index = (cb->tail + i) % cb->capacity;
        printf("%d", cb->data[index]);
        if (i < cb->count - 1) printf(", ");
    }
    printf("] (count=%zu/%zu)\n", cb->count, cb->capacity);
}

/* Destroy buffer */
void cb_destroy(CircularBuffer *cb) {
    free(cb->data);
    free(cb);
}

int main() {
    printf("=== Circular Buffer ===\n\n");

    CircularBuffer *cb = cb_create(5);

    printf("Push 10, 20, 30:\n");
    cb_push(cb, 10);
    cb_push(cb, 20);
    cb_push(cb, 30);
    cb_print(cb);

    int value;
    printf("\nPop: %d\n", (cb_pop(cb, &value) == 0) ? value : -1);
    cb_print(cb);

    printf("\nPush 40, 50, 60:\n");
    cb_push(cb, 40);
    cb_push(cb, 50);
    cb_push(cb, 60);
    cb_print(cb);

    printf("\nPush 70 (buffer full, overwrite):\n");
    cb_push(cb, 70);
    cb_print(cb);

    cb_destroy(cb);
    return 0;
}
