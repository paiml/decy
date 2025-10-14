// String utility functions inspired by real C codebases
// Testing common patterns: string length, comparison, copying

int string_length(char* str) {
    int len;
    len = 0;
    while (*str != 0) {
        len = len + 1;
        str = str + 1;
    }
    return len;
}

int string_compare(char* s1, char* s2) {
    while (*s1 != 0 && *s2 != 0) {
        if (*s1 != *s2) {
            return *s1 - *s2;
        }
        s1 = s1 + 1;
        s2 = s2 + 1;
    }
    return *s1 - *s2;
}

void string_copy(char* dest, char* src) {
    while (*src != 0) {
        *dest = *src;
        dest = dest + 1;
        src = src + 1;
    }
    *dest = 0;
}
