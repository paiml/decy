/* K&R C Chapter 5: Doubly-Linked List
 * K&R §6.5: Pointer-based data structures
 * Tests bidirectional linked list implementation
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct node {
    int data;
    struct node *prev;
    struct node *next;
} Node;

typedef struct {
    Node *head;
    Node *tail;
    int size;
} DoublyLinkedList;

/* Create list */
DoublyLinkedList *dll_create(void) {
    DoublyLinkedList *list = malloc(sizeof(DoublyLinkedList));
    list->head = NULL;
    list->tail = NULL;
    list->size = 0;
    return list;
}

/* Create node */
Node *dll_create_node(int data) {
    Node *node = malloc(sizeof(Node));
    node->data = data;
    node->prev = NULL;
    node->next = NULL;
    return node;
}

/* Push front */
void dll_push_front(DoublyLinkedList *list, int data) {
    Node *node = dll_create_node(data);

    if (list->head == NULL) {
        list->head = list->tail = node;
    } else {
        node->next = list->head;
        list->head->prev = node;
        list->head = node;
    }

    list->size++;
}

/* Push back */
void dll_push_back(DoublyLinkedList *list, int data) {
    Node *node = dll_create_node(data);

    if (list->tail == NULL) {
        list->head = list->tail = node;
    } else {
        node->prev = list->tail;
        list->tail->next = node;
        list->tail = node;
    }

    list->size++;
}

/* Pop front */
int dll_pop_front(DoublyLinkedList *list) {
    if (list->head == NULL) return -1;

    Node *node = list->head;
    int data = node->data;

    list->head = node->next;
    if (list->head) {
        list->head->prev = NULL;
    } else {
        list->tail = NULL;
    }

    free(node);
    list->size--;
    return data;
}

/* Pop back */
int dll_pop_back(DoublyLinkedList *list) {
    if (list->tail == NULL) return -1;

    Node *node = list->tail;
    int data = node->data;

    list->tail = node->prev;
    if (list->tail) {
        list->tail->next = NULL;
    } else {
        list->head = NULL;
    }

    free(node);
    list->size--;
    return data;
}

/* Print forward */
void dll_print_forward(DoublyLinkedList *list) {
    printf("[");
    Node *current = list->head;
    while (current) {
        printf("%d", current->data);
        if (current->next) printf(" <-> ");
        current = current->next;
    }
    printf("]\n");
}

/* Print backward */
void dll_print_backward(DoublyLinkedList *list) {
    printf("[");
    Node *current = list->tail;
    while (current) {
        printf("%d", current->data);
        if (current->prev) printf(" <-> ");
        current = current->prev;
    }
    printf("]\n");
}

/* Destroy list */
void dll_destroy(DoublyLinkedList *list) {
    Node *current = list->head;
    while (current) {
        Node *next = current->next;
        free(current);
        current = next;
    }
    free(list);
}

int main() {
    printf("=== Doubly-Linked List ===\n\n");

    DoublyLinkedList *list = dll_create();

    printf("Push back: 10, 20, 30\n");
    dll_push_back(list, 10);
    dll_push_back(list, 20);
    dll_push_back(list, 30);
    printf("Forward:  ");
    dll_print_forward(list);
    printf("Backward: ");
    dll_print_backward(list);
    printf("\n");

    printf("Push front: 5\n");
    dll_push_front(list, 5);
    printf("Forward:  ");
    dll_print_forward(list);
    printf("\n");

    printf("Pop front: %d\n", dll_pop_front(list));
    printf("Pop back:  %d\n", dll_pop_back(list));
    printf("Forward:  ");
    dll_print_forward(list);

    dll_destroy(list);
    return 0;
}
