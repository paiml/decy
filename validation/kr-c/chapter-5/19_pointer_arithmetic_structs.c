/* K&R C Chapter 5: Pointer Arithmetic with Structs
 * Pointer arithmetic on structure arrays
 */

#include <stdio.h>
#include <string.h>

struct student {
    char name[50];
    int id;
    float gpa;
};

void print_students(struct student *arr, int n) {
    struct student *ptr;

    printf("Using pointer arithmetic:\n");
    for (ptr = arr; ptr < arr + n; ptr++) {
        printf("  ID %d: %s (GPA: %.2f)\n", ptr->id, ptr->name, ptr->gpa);
    }
}

int main() {
    struct student class[] = {
        {"Alice", 1001, 3.8},
        {"Bob", 1002, 3.5},
        {"Charlie", 1003, 3.9},
        {"Diana", 1004, 3.7},
        {"Eve", 1005, 3.6}
    };
    int n = sizeof(class) / sizeof(class[0]);

    /* Pointer arithmetic with structures */
    struct student *first = class;
    struct student *last = class + n - 1;

    printf("First student: %s\n", first->name);
    printf("Last student: %s\n", last->name);
    printf("Distance: %td students\n", last - first);

    /* Iterate with pointer arithmetic */
    print_students(class, n);

    /* Access middle element */
    struct student *middle = class + n / 2;
    printf("\nMiddle student: %s (GPA: %.2f)\n", middle->name, middle->gpa);

    /* Pointer comparison */
    struct student *ptr = class;
    int count = 0;
    while (ptr < class + n) {
        if (ptr->gpa >= 3.7)
            count++;
        ptr++;
    }
    printf("Students with GPA >= 3.7: %d\n", count);

    return 0;
}
