// Dynamic string builder
// Tests: dynamic memory, realloc, string operations

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    char* data;
    size_t length;
    size_t capacity;
} StringBuilder;

StringBuilder* sb_create(void) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    if (sb != NULL) {
        sb->capacity = 16;
        sb->length = 0;
        sb->data = (char*)malloc(sb->capacity);
        sb->data[0] = '\0';
    }
    return sb;
}

void sb_append(StringBuilder* sb, const char* str) {
    size_t str_len = strlen(str);
    size_t new_len = sb->length + str_len;

    if (new_len + 1 > sb->capacity) {
        while (sb->capacity <= new_len) {
            sb->capacity *= 2;
        }
        sb->data = (char*)realloc(sb->data, sb->capacity);
    }

    strcpy(sb->data + sb->length, str);
    sb->length = new_len;
}

void sb_append_char(StringBuilder* sb, char c) {
    if (sb->length + 2 > sb->capacity) {
        sb->capacity *= 2;
        sb->data = (char*)realloc(sb->data, sb->capacity);
    }

    sb->data[sb->length] = c;
    sb->length++;
    sb->data[sb->length] = '\0';
}

char* sb_to_string(StringBuilder* sb) {
    char* result = (char*)malloc(sb->length + 1);
    strcpy(result, sb->data);
    return result;
}

void sb_free(StringBuilder* sb) {
    free(sb->data);
    free(sb);
}

int main(void) {
    StringBuilder* sb = sb_create();

    sb_append(sb, "Hello");
    sb_append_char(sb, ' ');
    sb_append(sb, "World");
    sb_append_char(sb, '!');

    printf("%s\n", sb->data);
    printf("Length: %zu, Capacity: %zu\n", sb->length, sb->capacity);

    char* result = sb_to_string(sb);
    printf("Result: %s\n", result);

    free(result);
    sb_free(sb);

    return 0;
}
