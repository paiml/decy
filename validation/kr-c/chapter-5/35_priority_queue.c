/* K&R C Chapter 5: Priority Queue (Binary Heap)
 * K&R §5.10: Heap-based priority queue
 * Tests min-heap implementation
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int *data;
    int size;
    int capacity;
} PriorityQueue;

/* Create priority queue */
PriorityQueue *pq_create(int capacity) {
    PriorityQueue *pq = malloc(sizeof(PriorityQueue));
    pq->data = malloc(capacity * sizeof(int));
    pq->size = 0;
    pq->capacity = capacity;
    return pq;
}

/* Swap elements */
void pq_swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

/* Get parent index */
int pq_parent(int i) {
    return (i - 1) / 2;
}

/* Get left child index */
int pq_left(int i) {
    return 2 * i + 1;
}

/* Get right child index */
int pq_right(int i) {
    return 2 * i + 2;
}

/* Bubble up */
void pq_bubble_up(PriorityQueue *pq, int index) {
    while (index > 0 && pq->data[pq_parent(index)] > pq->data[index]) {
        pq_swap(&pq->data[index], &pq->data[pq_parent(index)]);
        index = pq_parent(index);
    }
}

/* Bubble down */
void pq_bubble_down(PriorityQueue *pq, int index) {
    int min_index = index;
    int left = pq_left(index);
    int right = pq_right(index);

    if (left < pq->size && pq->data[left] < pq->data[min_index]) {
        min_index = left;
    }

    if (right < pq->size && pq->data[right] < pq->data[min_index]) {
        min_index = right;
    }

    if (min_index != index) {
        pq_swap(&pq->data[index], &pq->data[min_index]);
        pq_bubble_down(pq, min_index);
    }
}

/* Insert */
void pq_insert(PriorityQueue *pq, int value) {
    if (pq->size >= pq->capacity) {
        printf("Priority queue full\n");
        return;
    }

    pq->data[pq->size] = value;
    pq_bubble_up(pq, pq->size);
    pq->size++;
}

/* Extract min */
int pq_extract_min(PriorityQueue *pq) {
    if (pq->size == 0) {
        printf("Priority queue empty\n");
        return -1;
    }

    int min = pq->data[0];
    pq->data[0] = pq->data[pq->size - 1];
    pq->size--;
    pq_bubble_down(pq, 0);

    return min;
}

/* Peek min */
int pq_peek(PriorityQueue *pq) {
    if (pq->size == 0) return -1;
    return pq->data[0];
}

/* Print heap */
void pq_print(PriorityQueue *pq) {
    printf("[");
    for (int i = 0; i < pq->size; i++) {
        printf("%d", pq->data[i]);
        if (i < pq->size - 1) printf(", ");
    }
    printf("] (size=%d)\n", pq->size);
}

/* Destroy */
void pq_destroy(PriorityQueue *pq) {
    free(pq->data);
    free(pq);
}

int main() {
    printf("=== Priority Queue (Min-Heap) ===\n\n");

    PriorityQueue *pq = pq_create(10);

    printf("Insert: 15, 10, 20, 8, 12, 25, 6\n");
    pq_insert(pq, 15);
    pq_insert(pq, 10);
    pq_insert(pq, 20);
    pq_insert(pq, 8);
    pq_insert(pq, 12);
    pq_insert(pq, 25);
    pq_insert(pq, 6);

    printf("Heap: ");
    pq_print(pq);

    printf("\nPeek min: %d\n", pq_peek(pq));

    printf("\nExtract min: ");
    while (pq->size > 0) {
        printf("%d ", pq_extract_min(pq));
    }
    printf("\n");

    pq_destroy(pq);
    return 0;
}
