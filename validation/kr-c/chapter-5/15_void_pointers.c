/* K&R C Chapter 5: Void Pointers
 * Generic pointers and type casting
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Generic swap using void pointers */
void swap(void *a, void *b, size_t size) {
    void *temp = malloc(size);
    if (temp == NULL)
        return;

    memcpy(temp, a, size);
    memcpy(a, b, size);
    memcpy(b, temp, size);

    free(temp);
}

/* Generic print function */
void print_value(void *ptr, char type) {
    switch (type) {
        case 'i':
            printf("%d", *(int *)ptr);
            break;
        case 'f':
            printf("%.2f", *(float *)ptr);
            break;
        case 'c':
            printf("%c", *(char *)ptr);
            break;
        case 's':
            printf("%s", (char *)ptr);
            break;
    }
}

int main() {
    /* Swap integers */
    int x = 10, y = 20;
    printf("Before swap: x = %d, y = %d\n", x, y);
    swap(&x, &y, sizeof(int));
    printf("After swap:  x = %d, y = %d\n\n", x, y);

    /* Swap floats */
    float f1 = 3.14, f2 = 2.71;
    printf("Before swap: f1 = %.2f, f2 = %.2f\n", f1, f2);
    swap(&f1, &f2, sizeof(float));
    printf("After swap:  f1 = %.2f, f2 = %.2f\n\n", f1, f2);

    /* Swap characters */
    char c1 = 'A', c2 = 'Z';
    printf("Before swap: c1 = %c, c2 = %c\n", c1, c2);
    swap(&c1, &c2, sizeof(char));
    printf("After swap:  c1 = %c, c2 = %c\n\n", c1, c2);

    /* Generic printing */
    printf("Generic print function:\n");
    int num = 42;
    float pi = 3.14159;
    char letter = 'X';
    char *str = "Hello";

    printf("  int:    ");
    print_value(&num, 'i');
    printf("\n");

    printf("  float:  ");
    print_value(&pi, 'f');
    printf("\n");

    printf("  char:   ");
    print_value(&letter, 'c');
    printf("\n");

    printf("  string: ");
    print_value(str, 's');
    printf("\n");

    /* Void pointer array (generic container) */
    void *generic_array[4];
    generic_array[0] = &num;
    generic_array[1] = &pi;
    generic_array[2] = &letter;
    generic_array[3] = str;

    printf("\nGeneric array:\n");
    printf("  [0] (int):    ");
    print_value(generic_array[0], 'i');
    printf("\n");

    printf("  [1] (float):  ");
    print_value(generic_array[1], 'f');
    printf("\n");

    printf("  [2] (char):   ");
    print_value(generic_array[2], 'c');
    printf("\n");

    printf("  [3] (string): ");
    print_value(generic_array[3], 's');
    printf("\n");

    return 0;
}
