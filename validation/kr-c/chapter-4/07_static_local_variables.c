/* K&R C Chapter 4.3: External Variables
 * Page 80-83
 * Static local variables - preserving state across calls
 */

#include <stdio.h>

/* counter: function with static local variable */
int counter(void) {
    static int count = 0;  /* initialized only once */
    return ++count;
}

/* generate_id: generate unique IDs */
int generate_id(void) {
    static int next_id = 1000;
    return next_id++;
}

/* running_sum: keep running total */
int running_sum(int value) {
    static int total = 0;
    total += value;
    return total;
}

int main() {
    printf("Counter function (static local):\n");
    for (int i = 0; i < 5; i++)
        printf("Call %d: count = %d\n", i + 1, counter());

    printf("\nID generator:\n");
    printf("ID 1: %d\n", generate_id());
    printf("ID 2: %d\n", generate_id());
    printf("ID 3: %d\n", generate_id());

    printf("\nRunning sum:\n");
    printf("Add 10: total = %d\n", running_sum(10));
    printf("Add 20: total = %d\n", running_sum(20));
    printf("Add 30: total = %d\n", running_sum(30));
    printf("Add -15: total = %d\n", running_sum(-15));

    return 0;
}
