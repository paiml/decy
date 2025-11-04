/* K&R C Chapter 7: String Conversion Functions
 * K&R §7.2, Appendix B: atoi, atof, strtol, strtod
 * Tests string-to-number conversion functions
 */

#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <limits.h>

/* Demo atoi */
void demo_atoi(void) {
    printf("=== atoi() - String to Integer ===\n");

    const char *test_cases[] = {
        "123",
        "-456",
        "  789",
        "42abc",
        "abc42",
        "0",
        "2147483647",
        "-2147483648"
    };

    for (int i = 0; i < 8; i++) {
        int result = atoi(test_cases[i]);
        printf("  atoi(\"%s\") = %d\n", test_cases[i], result);
    }
    printf("\n");
}

/* Demo atof */
void demo_atof(void) {
    printf("=== atof() - String to Double ===\n");

    const char *test_cases[] = {
        "3.14159",
        "-2.71828",
        "  1.414",
        "123.456e2",
        "1e-3",
        "0.0",
        "123.45abc"
    };

    for (int i = 0; i < 7; i++) {
        double result = atof(test_cases[i]);
        printf("  atof(\"%s\") = %g\n", test_cases[i], result);
    }
    printf("\n");
}

/* Demo strtol with error checking */
void demo_strtol(void) {
    printf("=== strtol() - Robust String to Long ===\n");

    const char *test_cases[] = {
        "123",
        "-456",
        "0xFF",     /* Hexadecimal */
        "077",      /* Octal */
        "1010",     /* Binary (base 2) */
        "123abc",   /* Partial conversion */
        "abc123",   /* Invalid */
        "999999999999999999999"  /* Overflow */
    };

    int bases[] = {10, 10, 16, 8, 2, 10, 10, 10};

    for (int i = 0; i < 8; i++) {
        char *endptr;
        errno = 0;
        long result = strtol(test_cases[i], &endptr, bases[i]);

        printf("  strtol(\"%s\", base %d):\n", test_cases[i], bases[i]);

        if (errno == ERANGE) {
            printf("    ERROR: Overflow/underflow\n");
        } else if (endptr == test_cases[i]) {
            printf("    ERROR: No conversion performed\n");
        } else {
            printf("    Value: %ld\n", result);
            if (*endptr != '\0') {
                printf("    Stopped at: '%s'\n", endptr);
            }
        }
    }
    printf("\n");
}

/* Demo strtod with error checking */
void demo_strtod(void) {
    printf("=== strtod() - Robust String to Double ===\n");

    const char *test_cases[] = {
        "3.14159",
        "1.23e10",
        "  -456.789",
        "123.45abc",
        "abc",
        "INF",
        "NAN"
    };

    for (int i = 0; i < 7; i++) {
        char *endptr;
        errno = 0;
        double result = strtod(test_cases[i], &endptr);

        printf("  strtod(\"%s\"):\n", test_cases[i]);

        if (errno == ERANGE) {
            printf("    ERROR: Range error\n");
        } else if (endptr == test_cases[i]) {
            printf("    ERROR: No conversion\n");
        } else {
            printf("    Value: %g\n", result);
            if (*endptr != '\0') {
                printf("    Remaining: '%s'\n", endptr);
            }
        }
    }
    printf("\n");
}

/* Parse CSV line */
void parse_csv_line(const char *line) {
    printf("Parsing CSV: \"%s\"\n", line);
    char buffer[100];
    strcpy(buffer, line);

    char *token = strtok(buffer, ",");
    int field = 0;
    while (token != NULL) {
        /* Try to parse as number */
        char *endptr;
        long int_val = strtol(token, &endptr, 10);

        if (*endptr == '\0') {
            printf("  Field %d (int): %ld\n", field, int_val);
        } else {
            double float_val = strtod(token, &endptr);
            if (*endptr == '\0') {
                printf("  Field %d (float): %g\n", field, float_val);
            } else {
                printf("  Field %d (string): '%s'\n", field, token);
            }
        }

        token = strtok(NULL, ",");
        field++;
    }
    printf("\n");
}

/* Validate and convert user input */
int safe_read_int(const char *prompt, int *result) {
    char buffer[100];
    printf("%s", prompt);

    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return 0;
    }

    char *endptr;
    errno = 0;
    long val = strtol(buffer, &endptr, 10);

    /* Check for errors */
    if (errno == ERANGE || val > INT_MAX || val < INT_MIN) {
        printf("  Error: Number out of range\n");
        return 0;
    }
    if (endptr == buffer || (*endptr != '\n' && *endptr != '\0')) {
        printf("  Error: Invalid input\n");
        return 0;
    }

    *result = (int)val;
    return 1;
}

/* Parse hex color code */
void parse_hex_color(const char *hex) {
    if (hex[0] == '#')
        hex++;

    char *endptr;
    long color = strtol(hex, &endptr, 16);

    if (*endptr != '\0' || strlen(hex) != 6) {
        printf("Invalid color code: #%s\n", hex);
        return;
    }

    int r = (color >> 16) & 0xFF;
    int g = (color >> 8) & 0xFF;
    int b = color & 0xFF;

    printf("Color #%s: RGB(%d, %d, %d)\n", hex, r, g, b);
}

int main() {
    printf("=== String Conversion Functions ===\n\n");

    demo_atoi();
    demo_atof();
    demo_strtol();
    demo_strtod();

    /* CSV parsing */
    parse_csv_line("123,45.67,hello,3.14");
    parse_csv_line("10,20,30");

    /* Hex color parsing */
    printf("Hex color parsing:\n");
    parse_hex_color("#FF5733");
    parse_hex_color("#00AAFF");
    parse_hex_color("#123456");
    printf("\n");

    printf("Conversion function comparison:\n");
    printf("  atoi/atof:      Simple, no error checking\n");
    printf("  strtol/strtod:  Robust, with error detection\n");
    printf("  strtol/strtod:  Support multiple bases\n");
    printf("  strtol/strtod:  Can find where parsing stopped\n");
    printf("\nBest practice: Use strtol/strtod for production code\n");

    return 0;
}
