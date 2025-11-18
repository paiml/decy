/* K&R C Chapter 5: Pointer-Based Linked List
 * Simple singly-linked list implementation
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct node {
    int data;
    struct node *next;
} Node;

typedef struct {
    Node *head;
    int size;
} List;

void list_init(List *list) {
    list->head = NULL;
    list->size = 0;
}

void list_push_front(List *list, int value) {
    Node *new_node = (Node*)malloc(sizeof(Node));
    new_node->data = value;
    new_node->next = list->head;
    list->head = new_node;
    list->size++;
}

void list_push_back(List *list, int value) {
    Node *new_node = (Node*)malloc(sizeof(Node));
    new_node->data = value;
    new_node->next = NULL;

    if (list->head == NULL) {
        list->head = new_node;
    } else {
        Node *current = list->head;
        while (current->next != NULL)
            current = current->next;
        current->next = new_node;
    }
    list->size++;
}

int list_pop_front(List *list) {
    if (list->head == NULL)
        return -1;

    Node *old_head = list->head;
    int value = old_head->data;
    list->head = old_head->next;
    free(old_head);
    list->size--;

    return value;
}

void list_print(List *list) {
    Node *current = list->head;
    printf("List [%d]: ", list->size);
    while (current != NULL) {
        printf("%d ", current->data);
        current = current->next;
    }
    printf("\n");
}

int list_find(List *list, int value) {
    Node *current = list->head;
    int index = 0;

    while (current != NULL) {
        if (current->data == value)
            return index;
        current = current->next;
        index++;
    }

    return -1;
}

void list_reverse(List *list) {
    Node *prev = NULL;
    Node *current = list->head;
    Node *next = NULL;

    while (current != NULL) {
        next = current->next;
        current->next = prev;
        prev = current;
        current = next;
    }

    list->head = prev;
}

void list_free(List *list) {
    Node *current = list->head;
    while (current != NULL) {
        Node *next = current->next;
        free(current);
        current = next;
    }
    list->head = NULL;
    list->size = 0;
}

int main() {
    List list;
    list_init(&list);

    printf("=== Linked List Demo ===\n\n");

    /* Push elements */
    printf("Pushing to front: 3, 2, 1\n");
    list_push_front(&list, 3);
    list_push_front(&list, 2);
    list_push_front(&list, 1);
    list_print(&list);

    printf("\nPushing to back: 4, 5, 6\n");
    list_push_back(&list, 4);
    list_push_back(&list, 5);
    list_push_back(&list, 6);
    list_print(&list);

    /* Find elements */
    printf("\nSearching for 5: ");
    int index = list_find(&list, 5);
    if (index >= 0)
        printf("found at index %d\n", index);
    else
        printf("not found\n");

    printf("Searching for 99: ");
    index = list_find(&list, 99);
    if (index >= 0)
        printf("found at index %d\n", index);
    else
        printf("not found\n");

    /* Reverse list */
    printf("\nReversing list...\n");
    list_reverse(&list);
    list_print(&list);

    /* Pop elements */
    printf("\nPopping from front:\n");
    for (int i = 0; i < 3; i++) {
        int value = list_pop_front(&list);
        printf("  Popped: %d\n", value);
        list_print(&list);
    }

    /* Cleanup */
    list_free(&list);
    printf("\nList freed\n");
    list_print(&list);

    return 0;
}
