/* K&R C Chapter 7: Wide Character I/O
 * K&R Appendix B: wchar.h - Wide character support
 * Tests Unicode and wide character operations
 */

#include <stdio.h>
#include <stdlib.h>
#include <wchar.h>
#include <locale.h>

/* Write wide characters to file */
void write_wide_chars(const char *filename) {
    FILE *fp = fopen(filename, "w");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    printf("Writing wide characters to file:\n");

    /* Write ASCII */
    fwprintf(fp, L"ASCII: Hello, World!\n");

    /* Write Unicode characters */
    fwprintf(fp, L"Greek: Δ Λ Σ Ω\n");
    fwprintf(fp, L"Math: ∑ ∫ √ ∞\n");
    fwprintf(fp, L"Arrows: ← ↑ → ↓\n");
    fwprintf(fp, L"Chinese: 你好世界\n");
    fwprintf(fp, L"Emoji: 😀 🚀 ⭐\n");

    printf("  Written successfully\n");
    fclose(fp);
}

/* Read wide characters from file */
void read_wide_chars(const char *filename) {
    FILE *fp = fopen(filename, "r");
    if (fp == NULL) {
        perror("fopen");
        return;
    }

    printf("Reading wide characters from file:\n");

    wchar_t buffer[200];
    while (fgetws(buffer, 200, fp) != NULL) {
        wprintf(L"  %ls", buffer);
    }

    fclose(fp);
}

/* Wide character string operations */
void wide_string_operations(void) {
    printf("=== Wide String Operations ===\n");

    wchar_t str1[] = L"Hello";
    wchar_t str2[] = L"World";
    wchar_t result[50];

    /* Length */
    printf("  Length of '%ls': %zu\n", str1, wcslen(str1));

    /* Concatenation */
    wcscpy(result, str1);
    wcscat(result, L" ");
    wcscat(result, str2);
    wprintf(L"  Concatenated: %ls\n", result);

    /* Comparison */
    int cmp = wcscmp(str1, str2);
    printf("  Compare '%ls' vs '%ls': %d\n", str1, str2, cmp);

    /* Search */
    wchar_t *found = wcschr(result, L'o');
    if (found) {
        wprintf(L"  Found 'o' at position: %ld\n", found - result);
    }
}

/* Wide character classification */
void wide_char_classification(void) {
    printf("\n=== Wide Character Classification ===\n");

    wchar_t test_chars[] = {L'A', L'5', L' ', L'α', L'中', L'💚', L'\n'};
    int count = sizeof(test_chars) / sizeof(test_chars[0]);

    for (int i = 0; i < count - 1; i++) {
        wchar_t wc = test_chars[i];
        wprintf(L"  Char '%lc' (U+%04X):\n", wc, (unsigned int)wc);
        printf("    iswalpha: %d\n", iswalpha(wc));
        printf("    iswdigit: %d\n", iswdigit(wc));
        printf("    iswspace: %d\n", iswspace(wc));
        printf("    iswupper: %d\n", iswupper(wc));
        printf("    iswlower: %d\n", iswlower(wc));
    }
}

/* Wide character conversion */
void wide_char_conversion(void) {
    printf("\n=== Wide Character Conversion ===\n");

    /* Narrow to wide */
    char narrow[] = "Hello, World!";
    wchar_t wide[50];

    mbstowcs(wide, narrow, 50);
    wprintf(L"  Narrow to wide: %ls\n", wide);

    /* Wide to narrow */
    char back_to_narrow[50];
    wcstombs(back_to_narrow, wide, 50);
    printf("  Wide to narrow: %s\n", back_to_narrow);

    /* Single character conversion */
    wchar_t wc = L'A';
    char mb[MB_CUR_MAX];
    int len = wctomb(mb, wc);
    printf("  Wide char 'A' to multibyte: %d bytes\n", len);
}

/* Wide character formatting */
void wide_char_formatting(void) {
    printf("\n=== Wide Character Formatting ===\n");

    wchar_t buffer[100];

    /* Format integers */
    swprintf(buffer, 100, L"Integer: %d", 42);
    wprintf(L"  %ls\n", buffer);

    /* Format floats */
    swprintf(buffer, 100, L"Float: %.2f", 3.14159);
    wprintf(L"  %ls\n", buffer);

    /* Format strings */
    swprintf(buffer, 100, L"String: %ls", L"Hello");
    wprintf(L"  %ls\n", buffer);

    /* Mixed formatting */
    swprintf(buffer, 100, L"Name: %ls, Age: %d, Score: %.1f",
             L"Alice", 25, 95.5);
    wprintf(L"  %ls\n", buffer);
}

/* Wide character console I/O */
void wide_char_console_io(void) {
    printf("\n=== Wide Character Console I/O ===\n");

    /* Output to console */
    wprintf(L"  English: Hello, World!\n");
    wprintf(L"  Spanish: ¡Hola, Mundo!\n");
    wprintf(L"  French: Bonjour, le Monde!\n");
    wprintf(L"  German: Hallo, Welt!\n");
    wprintf(L"  Russian: Привет, мир!\n");
    wprintf(L"  Japanese: こんにちは世界!\n");
    wprintf(L"  Arabic: مرحبا بالعالم!\n");

    /* Note: Input with wscanf is tricky and platform-dependent */
    printf("  (Wide character input with wscanf is platform-specific)\n");
}

/* Wide character comparison and searching */
void wide_char_searching(void) {
    printf("\n=== Wide Character Searching ===\n");

    wchar_t text[] = L"The quick brown fox jumps over the lazy dog";
    wchar_t *found;

    /* Find character */
    found = wcschr(text, L'q');
    if (found) {
        wprintf(L"  Found 'q' at: %ls\n", found);
    }

    /* Find last occurrence */
    found = wcsrchr(text, L'o');
    if (found) {
        wprintf(L"  Last 'o' at: %ls\n", found);
    }

    /* Find substring */
    found = wcsstr(text, L"fox");
    if (found) {
        wprintf(L"  Found 'fox' at: %ls\n", found);
    }

    /* Search for any of multiple characters */
    found = wcspbrk(text, L"xyz");
    if (found) {
        wprintf(L"  First occurrence of x/y/z: %ls\n", found);
    }
}

int main() {
    /* Set locale for proper wide character support */
    setlocale(LC_ALL, "");

    printf("=== Wide Character I/O ===\n\n");

    /* File I/O */
    const char *filename = "wide_char_test.txt";
    printf("File I/O:\n");
    write_wide_chars(filename);
    read_wide_chars(filename);
    printf("\n");

    /* String operations */
    wide_string_operations();

    /* Classification */
    wide_char_classification();

    /* Conversion */
    wide_char_conversion();

    /* Formatting */
    wide_char_formatting();

    /* Console I/O */
    wide_char_console_io();

    /* Searching */
    wide_char_searching();

    /* Cleanup */
    remove(filename);

    printf("\nWide character support:\n");
    printf("  - wchar_t: Wide character type\n");
    printf("  - wprintf/wscanf: Wide character I/O\n");
    printf("  - wcs* functions: Wide string operations\n");
    printf("  - Locale-aware character handling\n");
    printf("  - Essential for internationalization (i18n)\n");

    return 0;
}
