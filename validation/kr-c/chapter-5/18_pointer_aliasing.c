/* K&R C Chapter 5: Pointer Aliasing and Restrict
 * Multiple pointers to same data
 */

#include <stdio.h>
#include <string.h>

/* Demonstrate pointer aliasing */
void modify_through_aliases(int *p1, int *p2) {
    *p1 = 100;
    printf("After *p1 = 100: *p1 = %d, *p2 = %d\n", *p1, *p2);

    *p2 = 200;
    printf("After *p2 = 200: *p1 = %d, *p2 = %d\n", *p1, *p2);
}

/* Array aliasing through pointers */
void array_aliasing(int *arr1, int *arr2, int size) {
    printf("Arrays before modification:\n");
    printf("  arr1: ");
    for (int i = 0; i < size; i++) printf("%d ", arr1[i]);
    printf("\n  arr2: ");
    for (int i = 0; i < size; i++) printf("%d ", arr2[i]);
    printf("\n");

    /* Modify through first pointer */
    arr1[0] = 999;

    printf("After arr1[0] = 999:\n");
    printf("  arr1: ");
    for (int i = 0; i < size; i++) printf("%d ", arr1[i]);
    printf("\n  arr2: ");
    for (int i = 0; i < size; i++) printf("%d ", arr2[i]);
    printf("\n");
}

/* Overlapping memory regions */
void overlapping_copy(char *dest, char *src, size_t n) {
    printf("Overlapping copy (naive, may have issues):\n");
    printf("  Before: dest=\"%s\", src=\"%s\"\n", dest, src);

    /* Naive copy - problematic if regions overlap */
    for (size_t i = 0; i < n; i++) {
        dest[i] = src[i];
    }

    printf("  After:  dest=\"%s\"\n", dest);
}

int main() {
    /* Simple aliasing */
    printf("=== Simple Pointer Aliasing ===\n");
    int value = 42;
    int *ptr1 = &value;
    int *ptr2 = &value;  /* Both point to same location */

    printf("Initial: value = %d, *ptr1 = %d, *ptr2 = %d\n",
           value, *ptr1, *ptr2);

    modify_through_aliases(ptr1, ptr2);

    /* Array aliasing */
    printf("\n=== Array Aliasing ===\n");
    int numbers[] = {1, 2, 3, 4, 5};
    int *p1 = numbers;
    int *p2 = numbers;  /* Both point to same array */

    array_aliasing(p1, p2, 5);

    /* Struct member aliasing */
    printf("\n=== Struct Member Aliasing ===\n");
    struct {
        int x;
        int y;
    } point;

    point.x = 10;
    point.y = 20;

    int *px = &point.x;
    int *py = &point.y;

    printf("point: {x=%d, y=%d}\n", point.x, point.y);
    printf("*px = %d, *py = %d\n", *px, *py);

    *px = 100;
    printf("After *px = 100: point.x = %d\n", point.x);

    /* Overlapping memory (memmove vs memcpy) */
    printf("\n=== Overlapping Memory Regions ===\n");
    char buffer[] = "Hello, World!";
    printf("Original: \"%s\"\n", buffer);

    /* Safe overlap with memmove */
    memmove(buffer + 2, buffer, 5);
    printf("After memmove(buffer+2, buffer, 5): \"%s\"\n", buffer);

    /* Demonstrate issue with naive copy on overlap */
    char buffer2[] = "ABCDEFGHIJ";
    overlapping_copy(buffer2 + 1, buffer2, 5);

    return 0;
}
