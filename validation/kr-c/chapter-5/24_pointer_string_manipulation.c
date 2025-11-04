/* K&R C Chapter 5: Pointer-Based String Manipulation
 * Advanced string operations using pointers
 */

#include <stdio.h>
#include <ctype.h>

/* String reverse in-place */
void str_reverse(char *s) {
    char *start = s;
    char *end = s;

    /* Find end */
    while (*end)
        end++;
    end--;

    /* Swap from ends */
    while (start < end) {
        char temp = *start;
        *start++ = *end;
        *end-- = temp;
    }
}

/* String to uppercase */
void str_upper(char *s) {
    while (*s) {
        *s = toupper(*s);
        s++;
    }
}

/* String to lowercase */
void str_lower(char *s) {
    while (*s) {
        *s = tolower(*s);
        s++;
    }
}

/* Count words in string */
int count_words(const char *s) {
    int count = 0;
    int in_word = 0;

    while (*s) {
        if (isspace(*s)) {
            in_word = 0;
        } else if (!in_word) {
            in_word = 1;
            count++;
        }
        s++;
    }

    return count;
}

/* Find substring */
char *str_find(const char *haystack, const char *needle) {
    if (!*needle)
        return (char*)haystack;

    const char *h, *n;

    while (*haystack) {
        h = haystack;
        n = needle;

        while (*h && *n && *h == *n) {
            h++;
            n++;
        }

        if (!*n)
            return (char*)haystack;

        haystack++;
    }

    return NULL;
}

/* Remove leading/trailing whitespace */
char *str_trim(char *s) {
    char *start = s;
    char *end;

    /* Skip leading whitespace */
    while (*start && isspace(*start))
        start++;

    if (*start == '\0')
        return start;

    /* Find end */
    end = start;
    while (*end)
        end++;
    end--;

    /* Remove trailing whitespace */
    while (end > start && isspace(*end))
        end--;

    *(end + 1) = '\0';

    return start;
}

int main() {
    char str1[] = "Hello, World!";
    char str2[] = "  spaces around  ";
    char str3[] = "The quick brown fox";
    char text[] = "Find the needle in the haystack";

    printf("Original: \"%s\"\n", str1);
    str_reverse(str1);
    printf("Reversed: \"%s\"\n", str1);
    str_reverse(str1);  /* Reverse back */

    printf("\nOriginal: \"%s\"\n", str1);
    str_upper(str1);
    printf("Uppercase: \"%s\"\n", str1);
    str_lower(str1);
    printf("Lowercase: \"%s\"\n", str1);

    printf("\nOriginal: \"%s\"\n", str2);
    char *trimmed = str_trim(str2);
    printf("Trimmed: \"%s\"\n", trimmed);

    printf("\nCounting words in: \"%s\"\n", str3);
    printf("Word count: %d\n", count_words(str3));

    printf("\nSearching in: \"%s\"\n", text);
    char *found = str_find(text, "needle");
    if (found)
        printf("Found \"needle\" at: \"%s\"\n", found);
    else
        printf("Not found\n");

    found = str_find(text, "xyz");
    if (found)
        printf("Found \"xyz\"\n");
    else
        printf("\"xyz\" not found\n");

    return 0;
}
