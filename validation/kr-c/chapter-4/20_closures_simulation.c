/* K&R C Chapter 4: Closure Simulation
 * Simulating closures with structs
 */

#include <stdio.h>
#include <stdlib.h>

/* Counter closure */
typedef struct {
    int count;
    int (*increment)(struct counter_closure*);
    int (*get_value)(struct counter_closure*);
    void (*reset)(struct counter_closure*);
} CounterClosure;

int counter_increment(CounterClosure *self) {
    return ++self->count;
}

int counter_get_value(CounterClosure *self) {
    return self->count;
}

void counter_reset(CounterClosure *self) {
    self->count = 0;
}

CounterClosure *make_counter(int initial) {
    CounterClosure *c = malloc(sizeof(CounterClosure));
    c->count = initial;
    c->increment = counter_increment;
    c->get_value = counter_get_value;
    c->reset = counter_reset;
    return c;
}

/* Adder closure - captures value */
typedef struct {
    int captured_value;
    int (*add)(struct adder_closure*, int);
} AdderClosure;

int adder_add(AdderClosure *self, int x) {
    return self->captured_value + x;
}

AdderClosure *make_adder(int value) {
    AdderClosure *a = malloc(sizeof(AdderClosure));
    a->captured_value = value;
    a->add = adder_add;
    return a;
}

/* Multiplier closure */
typedef struct {
    int multiplier;
    int (*multiply)(struct multiplier_closure*, int);
} MultiplierClosure;

int multiplier_multiply(MultiplierClosure *self, int x) {
    return self->multiplier * x;
}

MultiplierClosure *make_multiplier(int multiplier) {
    MultiplierClosure *m = malloc(sizeof(MultiplierClosure));
    m->multiplier = multiplier;
    m->multiply = multiplier_multiply;
    return m;
}

/* Accumulator closure */
typedef struct {
    int total;
    void (*add)(struct accumulator_closure*, int);
    int (*get_total)(struct accumulator_closure*);
} AccumulatorClosure;

void accumulator_add(AccumulatorClosure *self, int value) {
    self->total += value;
}

int accumulator_get_total(AccumulatorClosure *self) {
    return self->total;
}

AccumulatorClosure *make_accumulator(int initial) {
    AccumulatorClosure *a = malloc(sizeof(AccumulatorClosure));
    a->total = initial;
    a->add = accumulator_add;
    a->get_total = accumulator_get_total;
    return a;
}

int main() {
    printf("=== Closure Simulation ===\n\n");

    /* Counter closure */
    printf("Counter closure:\n");
    CounterClosure *counter1 = make_counter(0);
    CounterClosure *counter2 = make_counter(10);

    printf("  counter1: %d\n", counter1->increment(counter1));
    printf("  counter1: %d\n", counter1->increment(counter1));
    printf("  counter2: %d\n", counter2->increment(counter2));
    printf("  counter1: %d\n", counter1->get_value(counter1));
    printf("  counter2: %d\n", counter2->get_value(counter2));

    counter1->reset(counter1);
    printf("  counter1 after reset: %d\n", counter1->get_value(counter1));

    /* Adder closures */
    printf("\nAdder closures:\n");
    AdderClosure *add5 = make_adder(5);
    AdderClosure *add10 = make_adder(10);

    printf("  add5(3) = %d\n", add5->add(add5, 3));
    printf("  add5(7) = %d\n", add5->add(add5, 7));
    printf("  add10(3) = %d\n", add10->add(add10, 3));
    printf("  add10(7) = %d\n", add10->add(add10, 7));

    /* Multiplier closures */
    printf("\nMultiplier closures:\n");
    MultiplierClosure *double_it = make_multiplier(2);
    MultiplierClosure *triple_it = make_multiplier(3);

    printf("  double(5) = %d\n", double_it->multiply(double_it, 5));
    printf("  triple(5) = %d\n", triple_it->multiply(triple_it, 5));

    /* Accumulator closure */
    printf("\nAccumulator closure:\n");
    AccumulatorClosure *acc = make_accumulator(0);

    printf("  Initial: %d\n", acc->get_total(acc));
    acc->add(acc, 10);
    printf("  After add(10): %d\n", acc->get_total(acc));
    acc->add(acc, 20);
    printf("  After add(20): %d\n", acc->get_total(acc));
    acc->add(acc, -5);
    printf("  After add(-5): %d\n", acc->get_total(acc));

    /* Cleanup */
    free(counter1);
    free(counter2);
    free(add5);
    free(add10);
    free(double_it);
    free(triple_it);
    free(acc);

    printf("\nClosure benefits:\n");
    printf("  - Encapsulate state with behavior\n");
    printf("  - Create factory functions\n");
    printf("  - Each instance has independent state\n");

    return 0;
}
