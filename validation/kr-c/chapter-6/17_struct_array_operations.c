/* K&R C Chapter 6: Structure Array Operations
 * Sorting, searching, filtering arrays of structures
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int id;
    char name[30];
    float score;
} Student;

int compare_by_id(const void *a, const void *b) {
    const Student *sa = (const Student*)a;
    const Student *sb = (const Student*)b;
    return sa->id - sb->id;
}

int compare_by_name(const void *a, const void *b) {
    const Student *sa = (const Student*)a;
    const Student *sb = (const Student*)b;
    return strcmp(sa->name, sb->name);
}

int compare_by_score(const void *a, const void *b) {
    const Student *sa = (const Student*)a;
    const Student *sb = (const Student*)b;
    if (sa->score < sb->score) return 1;
    if (sa->score > sb->score) return -1;
    return 0;
}

void print_students(Student *arr, int n, const char *title) {
    printf("%s:\n", title);
    for (int i = 0; i < n; i++) {
        printf("  ID:%d %-20s Score:%.1f\n",
               arr[i].id, arr[i].name, arr[i].score);
    }
    printf("\n");
}

Student *find_by_id(Student *arr, int n, int id) {
    for (int i = 0; i < n; i++) {
        if (arr[i].id == id)
            return &arr[i];
    }
    return NULL;
}

int filter_by_score(Student *arr, int n, float min_score, Student *result) {
    int count = 0;
    for (int i = 0; i < n; i++) {
        if (arr[i].score >= min_score) {
            result[count++] = arr[i];
        }
    }
    return count;
}

int main() {
    Student students[] = {
        {1003, "Charlie", 85.5},
        {1001, "Alice", 92.0},
        {1005, "Eve", 78.5},
        {1002, "Bob", 88.0},
        {1004, "Diana", 95.5}
    };
    int n = sizeof(students) / sizeof(students[0]);

    print_students(students, n, "Original");

    /* Sort by ID */
    qsort(students, n, sizeof(Student), compare_by_id);
    print_students(students, n, "Sorted by ID");

    /* Sort by name */
    qsort(students, n, sizeof(Student), compare_by_name);
    print_students(students, n, "Sorted by Name");

    /* Sort by score (descending) */
    qsort(students, n, sizeof(Student), compare_by_score);
    print_students(students, n, "Sorted by Score (desc)");

    /* Find student */
    Student *found = find_by_id(students, n, 1002);
    if (found) {
        printf("Found student ID 1002: %s (Score: %.1f)\n\n",
               found->name, found->score);
    }

    /* Filter by score */
    Student high_scorers[10];
    int count = filter_by_score(students, n, 90.0, high_scorers);
    print_students(high_scorers, count, "Students with score >= 90");

    return 0;
}
