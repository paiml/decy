/* K&R C Chapter 3: Ternary Operator
 * K&R §3.5: Conditional expression (?:)
 * Tests ternary operator usage
 */

#include <stdio.h>

int max(int a, int b) {
    return (a > b) ? a : b;
}

int min(int a, int b) {
    return (a < b) ? a : b;
}

int abs_value(int x) {
    return (x < 0) ? -x : x;
}

const char *get_grade(int score) {
    return (score >= 90) ? "A" :
           (score >= 80) ? "B" :
           (score >= 70) ? "C" :
           (score >= 60) ? "D" : "F";
}

int main() {
    printf("=== Ternary Operator ===\n\n");

    printf("max(10, 20) = %d\n", max(10, 20));
    printf("min(10, 20) = %d\n", min(10, 20));
    printf("abs(-15) = %d\n", abs_value(-15));
    printf("\n");

    printf("Grading:\n");
    int scores[] = {95, 82, 73, 61, 45};
    for (int i = 0; i < 5; i++) {
        printf("  Score %d: Grade %s\n", scores[i], get_grade(scores[i]));
    }
    printf("\n");

    int x = 5;
    printf("x=%d is %s\n", x, (x % 2 == 0) ? "even" : "odd");

    printf("\nTernary operator:\n");
    printf("  Syntax: condition ? expr1 : expr2\n");
    printf("  Returns expr1 if condition true, else expr2\n");
    printf("  More concise than if-else for simple cases\n");

    return 0;
}
