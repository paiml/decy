/* K&R C Chapter 4: Function Currying Simulation
 * Partial application of functions (limited in C)
 */

#include <stdio.h>

/* Regular function */
int add(int a, int b) {
    return a + b;
}

/* Curried version - struct holds partial state */
typedef struct {
    int x;
} AddPartial;

AddPartial add_partial(int x) {
    AddPartial p = {x};
    return p;
}

int add_apply(AddPartial p, int y) {
    return p.x + y;
}

/* Multiply with partial application */
typedef struct {
    int multiplier;
} MultiplyPartial;

MultiplyPartial multiply_by(int multiplier) {
    MultiplyPartial p = {multiplier};
    return p;
}

int apply_multiply(MultiplyPartial p, int value) {
    return p.multiplier * value;
}

/* More complex: power function */
typedef struct {
    int base;
} PowerPartial;

PowerPartial power_base(int base) {
    PowerPartial p = {base};
    return p;
}

int power_apply(PowerPartial p, int exponent) {
    int result = 1;
    for (int i = 0; i < exponent; i++)
        result *= p.base;
    return result;
}

/* Comparison with threshold */
typedef struct {
    int threshold;
} ComparePartial;

ComparePartial greater_than(int threshold) {
    ComparePartial p = {threshold};
    return p;
}

int check_greater(ComparePartial p, int value) {
    return value > p.threshold;
}

int main() {
    printf("=== Function Currying Simulation ===\n\n");

    /* Regular function */
    printf("Regular add(5, 3) = %d\n", add(5, 3));

    /* Partial application */
    printf("\nPartial application:\n");
    AddPartial add5 = add_partial(5);
    printf("  add5 = add_partial(5)\n");
    printf("  add5(3) = %d\n", add_apply(add5, 3));
    printf("  add5(10) = %d\n", add_apply(add5, 10));

    AddPartial add10 = add_partial(10);
    printf("  add10 = add_partial(10)\n");
    printf("  add10(3) = %d\n", add_apply(add10, 3));

    /* Multiply partial */
    printf("\nMultiplication:\n");
    MultiplyPartial double_it = multiply_by(2);
    MultiplyPartial triple_it = multiply_by(3);

    printf("  double(5) = %d\n", apply_multiply(double_it, 5));
    printf("  double(10) = %d\n", apply_multiply(double_it, 10));
    printf("  triple(5) = %d\n", apply_multiply(triple_it, 5));

    /* Power partial */
    printf("\nPower functions:\n");
    PowerPartial powers_of_2 = power_base(2);
    PowerPartial powers_of_3 = power_base(3);

    printf("  Powers of 2: ");
    for (int i = 0; i <= 5; i++)
        printf("%d ", power_apply(powers_of_2, i));
    printf("\n");

    printf("  Powers of 3: ");
    for (int i = 0; i <= 5; i++)
        printf("%d ", power_apply(powers_of_3, i));
    printf("\n");

    /* Comparison partial */
    printf("\nComparison predicates:\n");
    ComparePartial gt10 = greater_than(10);
    ComparePartial gt50 = greater_than(50);

    int values[] = {5, 15, 45, 55, 100};
    for (int i = 0; i < 5; i++) {
        printf("  %d: gt10=%d, gt50=%d\n",
               values[i],
               check_greater(gt10, values[i]),
               check_greater(gt50, values[i]));
    }

    return 0;
}
