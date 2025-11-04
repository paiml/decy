/* K&R C Chapter 7: Character Classification
 * K&R §7.2, Appendix B: ctype.h functions
 * Tests character testing and conversion functions
 */

#include <stdio.h>
#include <ctype.h>
#include <string.h>

/* Classify single character */
void classify_char(char c) {
    printf("Character '%c' (ASCII %d):\n", c, (unsigned char)c);
    printf("  isalpha: %d\n", isalpha(c));
    printf("  isdigit: %d\n", isdigit(c));
    printf("  isalnum: %d\n", isalnum(c));
    printf("  isspace: %d\n", isspace(c));
    printf("  isupper: %d\n", isupper(c));
    printf("  islower: %d\n", islower(c));
    printf("  ispunct: %d\n", ispunct(c));
    printf("  isprint: %d\n", isprint(c));
    printf("  iscntrl: %d\n", iscntrl(c));
    printf("  toupper: '%c'\n", toupper(c));
    printf("  tolower: '%c'\n", tolower(c));
}

/* Count character categories in string */
void analyze_string(const char *str) {
    int alpha = 0, digit = 0, space = 0, punct = 0, other = 0;

    for (const char *p = str; *p; p++) {
        if (isalpha(*p))
            alpha++;
        else if (isdigit(*p))
            digit++;
        else if (isspace(*p))
            space++;
        else if (ispunct(*p))
            punct++;
        else
            other++;
    }

    printf("String: \"%s\"\n", str);
    printf("  Alphabetic: %d\n", alpha);
    printf("  Digits:     %d\n", digit);
    printf("  Whitespace: %d\n", space);
    printf("  Punctuation:%d\n", punct);
    printf("  Other:      %d\n", other);
    printf("  Total:      %lu\n", strlen(str));
}

/* Convert string to uppercase */
void str_to_upper(char *str) {
    for (char *p = str; *p; p++)
        *p = toupper(*p);
}

/* Convert string to lowercase */
void str_to_lower(char *str) {
    for (char *p = str; *p; p++)
        *p = tolower(*p);
}

/* Title case conversion */
void str_to_title(char *str) {
    int new_word = 1;
    for (char *p = str; *p; p++) {
        if (isspace(*p)) {
            new_word = 1;
        } else if (new_word) {
            *p = toupper(*p);
            new_word = 0;
        } else {
            *p = tolower(*p);
        }
    }
}

/* Remove non-alphanumeric characters */
void remove_non_alnum(char *str) {
    char *src = str, *dst = str;
    while (*src) {
        if (isalnum(*src))
            *dst++ = *src;
        src++;
    }
    *dst = '\0';
}

/* Validate password strength */
int validate_password(const char *password) {
    int has_upper = 0, has_lower = 0, has_digit = 0, has_special = 0;
    int length = strlen(password);

    if (length < 8)
        return 0;  /* Too short */

    for (const char *p = password; *p; p++) {
        if (isupper(*p))
            has_upper = 1;
        else if (islower(*p))
            has_lower = 1;
        else if (isdigit(*p))
            has_digit = 1;
        else if (ispunct(*p))
            has_special = 1;
    }

    return has_upper && has_lower && has_digit && has_special;
}

/* Extract words from text */
void extract_words(const char *text) {
    char buffer[100];
    int word_count = 0;

    printf("Words extracted from text:\n");
    const char *p = text;
    while (*p) {
        /* Skip non-alphanumeric */
        while (*p && !isalnum(*p))
            p++;

        if (*p) {
            /* Extract word */
            int i = 0;
            while (*p && isalnum(*p))
                buffer[i++] = *p++;
            buffer[i] = '\0';

            printf("  Word %d: '%s'\n", ++word_count, buffer);
        }
    }
    printf("Total words: %d\n", word_count);
}

int main() {
    printf("=== Character Classification ===\n\n");

    /* Classify individual characters */
    printf("Individual character classification:\n");
    classify_char('A');
    printf("\n");
    classify_char('5');
    printf("\n");
    classify_char(' ');
    printf("\n");

    /* Analyze strings */
    printf("String analysis:\n");
    analyze_string("Hello, World! 123");
    printf("\n");
    analyze_string("Test@Example.com");
    printf("\n");

    /* Case conversion */
    printf("Case conversion:\n");
    char str1[] = "Hello, World!";
    printf("  Original: %s\n", str1);
    str_to_upper(str1);
    printf("  Upper:    %s\n", str1);

    char str2[] = "Hello, World!";
    str_to_lower(str2);
    printf("  Lower:    %s\n", str2);

    char str3[] = "hello world from c programming";
    str_to_title(str3);
    printf("  Title:    %s\n", str3);
    printf("\n");

    /* Remove non-alphanumeric */
    printf("Remove non-alphanumeric:\n");
    char str4[] = "abc-123-xyz!@#";
    printf("  Original: %s\n", str4);
    remove_non_alnum(str4);
    printf("  Cleaned:  %s\n", str4);
    printf("\n");

    /* Password validation */
    printf("Password validation:\n");
    const char *passwords[] = {
        "weak",
        "Weak123",
        "Strong@123",
        "VeryStrong!2024"
    };
    for (int i = 0; i < 4; i++) {
        printf("  '%s': %s\n",
               passwords[i],
               validate_password(passwords[i]) ? "VALID" : "INVALID");
    }
    printf("\n");

    /* Extract words */
    printf("Word extraction:\n");
    extract_words("The quick brown fox jumps over the lazy dog!");
    printf("\n");

    /* Hexadecimal digit check */
    printf("Hexadecimal digit check:\n");
    const char *hex_chars = "0123456789abcdefABCDEFxyzXYZ";
    for (const char *p = hex_chars; *p; p++) {
        printf("  '%c': %s\n", *p, isxdigit(*p) ? "YES" : "NO");
    }

    printf("\nctype.h benefits:\n");
    printf("  - Portable character testing\n");
    printf("  - Locale-aware\n");
    printf("  - Fast table-driven implementation\n");
    printf("  - Essential for parsing and validation\n");

    return 0;
}
