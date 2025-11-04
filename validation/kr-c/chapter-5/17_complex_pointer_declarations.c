/* K&R C Chapter 5: Complicated Pointer Declarations
 * Advanced pointer declaration reading
 */

#include <stdio.h>

/* Function returning pointer to int */
int *func1(void) {
    static int x = 42;
    return &x;
}

/* Pointer to function returning int */
int func2(void) {
    return 100;
}

/* Array of pointers to functions returning int */
int add(void) { return 1; }
int sub(void) { return 2; }
int mul(void) { return 3; }

int main() {
    /* Simple pointer */
    int x = 10;
    int *p = &x;
    printf("int *p: p points to int, *p = %d\n", *p);

    /* Pointer to pointer */
    int **pp = &p;
    printf("int **pp: pointer to pointer, **pp = %d\n", **pp);

    /* Array of pointers */
    int a = 1, b = 2, c = 3;
    int *arr[3] = {&a, &b, &c};
    printf("int *arr[3]: array of 3 pointers, arr[1] = %d\n", *arr[1]);

    /* Pointer to array */
    int nums[5] = {10, 20, 30, 40, 50};
    int (*pa)[5] = &nums;
    printf("int (*pa)[5]: pointer to array of 5 ints, (*pa)[2] = %d\n", (*pa)[2]);

    /* Function returning pointer */
    int *result = func1();
    printf("int *func(): function returning pointer, *result = %d\n", *result);

    /* Pointer to function */
    int (*pf)(void) = func2;
    printf("int (*pf)(): pointer to function, pf() = %d\n", pf());

    /* Array of function pointers */
    int (*func_arr[3])(void) = {add, sub, mul};
    printf("int (*func_arr[3])(): array of function pointers\n");
    for (int i = 0; i < 3; i++) {
        printf("  func_arr[%d]() = %d\n", i, func_arr[i]());
    }

    /* Const variations */
    const int *cp1 = &x;  /* pointer to const int */
    int * const cp2 = &x;  /* const pointer to int */
    const int * const cp3 = &x;  /* const pointer to const int */

    printf("\nConst pointer variations:\n");
    printf("  const int *cp1: pointer to const int\n");
    printf("  int * const cp2: const pointer to int\n");
    printf("  const int * const cp3: const pointer to const int\n");

    /* Complex declaration */
    char *(*(*var)[10])(int *);
    printf("\nComplex: char *(*(*var)[10])(int *)\n");
    printf("  var is pointer to array of 10 pointers to functions\n");
    printf("  taking int* and returning char*\n");

    return 0;
}
