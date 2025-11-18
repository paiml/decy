/* K&R C Chapter 5: Smart Pointers (Reference Counting)
 * K&R §5.10: Manual reference counting for memory management
 * Tests reference-counted pointers
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Reference-counted object */
typedef struct {
    void *data;
    int ref_count;
    void (*destructor)(void*);
} RefCounted;

/* Create reference-counted object */
RefCounted *rc_create(void *data, void (*destructor)(void*)) {
    RefCounted *rc = malloc(sizeof(RefCounted));
    rc->data = data;
    rc->ref_count = 1;
    rc->destructor = destructor;

    printf("  RC created (ref_count=1)\n");
    return rc;
}

/* Retain (increment reference count) */
RefCounted *rc_retain(RefCounted *rc) {
    if (rc == NULL) return NULL;

    rc->ref_count++;
    printf("  RC retained (ref_count=%d)\n", rc->ref_count);
    return rc;
}

/* Release (decrement reference count) */
void rc_release(RefCounted *rc) {
    if (rc == NULL) return;

    rc->ref_count--;
    printf("  RC released (ref_count=%d)\n", rc->ref_count);

    if (rc->ref_count == 0) {
        printf("  RC deallocating...\n");
        if (rc->destructor) {
            rc->destructor(rc->data);
        }
        free(rc);
    }
}

/* Example: String data */
typedef struct {
    char *str;
} String;

void string_destructor(void *data) {
    String *s = (String*)data;
    printf("    Destroying string: '%s'\n", s->str);
    free(s->str);
    free(s);
}

String *string_create(const char *str) {
    String *s = malloc(sizeof(String));
    s->str = malloc(strlen(str) + 1);
    strcpy(s->str, str);
    return s;
}

void demo_basic_refcount(void) {
    printf("=== Basic Reference Counting ===\n");

    String *data = string_create("Hello, World!");
    RefCounted *rc = rc_create(data, string_destructor);

    /* Multiple owners */
    RefCounted *owner1 = rc_retain(rc);
    RefCounted *owner2 = rc_retain(rc);

    /* Release owners */
    rc_release(owner1);
    rc_release(owner2);
    rc_release(rc);

    printf("\n");
}

/* Example: Shared linked list node */
typedef struct node {
    int data;
    RefCounted *next;
} Node;

void node_destructor(void *data) {
    Node *node = (Node*)data;
    printf("    Destroying node: %d\n", node->data);

    if (node->next) {
        rc_release(node->next);
    }

    free(node);
}

Node *node_create(int data) {
    Node *node = malloc(sizeof(Node));
    node->data = data;
    node->next = NULL;
    return node;
}

void demo_shared_list(void) {
    printf("=== Shared Linked List ===\n");

    /* Create nodes */
    Node *node1 = node_create(1);
    RefCounted *rc1 = rc_create(node1, node_destructor);

    Node *node2 = node_create(2);
    RefCounted *rc2 = rc_create(node2, node_destructor);

    Node *node3 = node_create(3);
    RefCounted *rc3 = rc_create(node3, node_destructor);

    /* Link nodes */
    node1->next = rc_retain(rc2);
    node2->next = rc_retain(rc3);

    /* Create two list heads sharing tail */
    Node *head1 = node_create(10);
    RefCounted *rc_head1 = rc_create(head1, node_destructor);
    head1->next = rc_retain(rc2);

    printf("List 1: 10 -> 2 -> 3\n");
    printf("List 2: 1 -> 2 -> 3 (shares 2->3 with List 1)\n\n");

    /* Release lists */
    printf("Releasing List 1:\n");
    rc_release(rc_head1);

    printf("\nReleasing List 2:\n");
    rc_release(rc1);

    printf("\n");
}

/* Example: Circular reference problem */
typedef struct circ_node {
    int data;
    RefCounted *next;
    RefCounted *prev;  /* Weak reference (not counted) */
} CircNode;

void circ_node_destructor(void *data) {
    CircNode *node = (CircNode*)data;
    printf("    Destroying circular node: %d\n", node->data);

    if (node->next) {
        rc_release(node->next);
    }
    /* Don't release prev - it's a weak reference */

    free(node);
}

void demo_circular_reference(void) {
    printf("=== Circular Reference (Weak Pointer) ===\n");

    CircNode *node1 = malloc(sizeof(CircNode));
    node1->data = 1;
    node1->next = NULL;
    node1->prev = NULL;

    CircNode *node2 = malloc(sizeof(CircNode));
    node2->data = 2;
    node2->next = NULL;
    node2->prev = NULL;

    RefCounted *rc1 = rc_create(node1, circ_node_destructor);
    RefCounted *rc2 = rc_create(node2, circ_node_destructor);

    /* Create cycle with weak back pointer */
    node1->next = rc_retain(rc2);  /* Strong reference */
    node2->prev = rc1;             /* Weak reference (no retain) */

    printf("Node 1 <-weak- Node 2\n");
    printf("Node 1 -strong-> Node 2\n\n");

    printf("Releasing nodes:\n");
    rc_release(rc1);
    rc_release(rc2);

    printf("\n");
}

/* Copy-on-write example */
typedef struct {
    char *buffer;
    size_t length;
} Buffer;

void buffer_destructor(void *data) {
    Buffer *buf = (Buffer*)data;
    printf("    Destroying buffer: '%.*s'\n", (int)buf->length, buf->buffer);
    free(buf->buffer);
    free(buf);
}

RefCounted *buffer_clone_on_write(RefCounted *rc) {
    if (rc->ref_count == 1) {
        /* Sole owner - modify in place */
        printf("  Sole owner - modify in place\n");
        return rc;
    }

    /* Multiple owners - clone */
    printf("  Multiple owners - cloning buffer\n");
    Buffer *original = (Buffer*)rc->data;

    Buffer *clone = malloc(sizeof(Buffer));
    clone->length = original->length;
    clone->buffer = malloc(clone->length);
    memcpy(clone->buffer, original->buffer, clone->length);

    rc_release(rc);
    return rc_create(clone, buffer_destructor);
}

void demo_copy_on_write(void) {
    printf("=== Copy-on-Write ===\n");

    Buffer *buf = malloc(sizeof(Buffer));
    buf->buffer = malloc(10);
    strcpy(buf->buffer, "Hello");
    buf->length = 6;

    RefCounted *rc1 = rc_create(buf, buffer_destructor);
    RefCounted *rc2 = rc_retain(rc1);

    printf("Two owners of buffer\n");

    /* Modify rc2 - should trigger copy */
    rc2 = buffer_clone_on_write(rc2);
    Buffer *buf2 = (Buffer*)rc2->data;
    strcpy(buf2->buffer, "World");

    printf("After modification:\n");
    printf("  rc1 buffer: '%s'\n", ((Buffer*)rc1->data)->buffer);
    printf("  rc2 buffer: '%s'\n", ((Buffer*)rc2->data)->buffer);

    rc_release(rc1);
    rc_release(rc2);

    printf("\n");
}

int main() {
    printf("=== Smart Pointers (Reference Counting) ===\n\n");

    demo_basic_refcount();
    demo_shared_list();
    demo_circular_reference();
    demo_copy_on_write();

    printf("Reference counting benefits:\n");
    printf("  - Automatic memory management\n");
    printf("  - Shared ownership\n");
    printf("  - Predictable deallocation\n");
    printf("  - Copy-on-write optimization\n");
    printf("\nCaveats:\n");
    printf("  - Circular references cause leaks\n");
    printf("  - Use weak pointers to break cycles\n");
    printf("  - Thread safety requires atomic ops\n");

    return 0;
}
