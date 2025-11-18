/* K&R C Chapter 4.5: Header Files
 * Page 82-83
 * Using standard headers and function declarations
 */

#include <stdio.h>
#include <string.h>
#include <math.h>
#include <ctype.h>

/* Function declarations (prototypes) */
void print_string_info(const char *s);
void print_math_functions(double x);
void print_char_info(char c);

int main() {
    const char *test_str = "Hello, World!";
    double test_num = 2.5;
    char test_char = 'A';

    printf("=== String Functions (string.h) ===\n");
    print_string_info(test_str);

    printf("\n=== Math Functions (math.h) ===\n");
    print_math_functions(test_num);

    printf("\n=== Character Functions (ctype.h) ===\n");
    print_char_info(test_char);
    print_char_info('a');
    print_char_info('5');
    print_char_info(' ');

    return 0;
}

void print_string_info(const char *s) {
    printf("String: \"%s\"\n", s);
    printf("Length: %zu\n", strlen(s));
    printf("First char: '%c'\n", s[0]);

    char copy[100];
    strcpy(copy, s);
    printf("Copy: \"%s\"\n", copy);

    if (strcmp(s, copy) == 0)
        printf("Strings are equal\n");
}

void print_math_functions(double x) {
    printf("x = %.2f\n", x);
    printf("sqrt(x) = %.4f\n", sqrt(x));
    printf("pow(x, 3) = %.4f\n", pow(x, 3.0));
    printf("sin(x) = %.4f\n", sin(x));
    printf("cos(x) = %.4f\n", cos(x));
    printf("exp(x) = %.4f\n", exp(x));
    printf("log(x) = %.4f\n", log(x));
    printf("ceil(x) = %.0f\n", ceil(x));
    printf("floor(x) = %.0f\n", floor(x));
}

void print_char_info(char c) {
    printf("\nChar: '%c' (ASCII %d)\n", c, c);
    printf("  isalpha: %d\n", isalpha(c));
    printf("  isdigit: %d\n", isdigit(c));
    printf("  isspace: %d\n", isspace(c));
    printf("  isupper: %d\n", isupper(c));
    printf("  islower: %d\n", islower(c));
    printf("  toupper: '%c'\n", toupper(c));
    printf("  tolower: '%c'\n", tolower(c));
}
