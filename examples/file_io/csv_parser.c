// Simple CSV parser
// Tests: file I/O, string tokenization, dynamic arrays

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Record {
    char name[100];
    int age;
    char city[100];
} Record;

int parse_csv_line(const char* line, Record* record) {
    char* line_copy = (char*)malloc(strlen(line) + 1);
    strcpy(line_copy, line);

    char* token = strtok(line_copy, ",");
    if (token == NULL) {
        free(line_copy);
        return -1;
    }
    strncpy(record->name, token, sizeof(record->name) - 1);

    token = strtok(NULL, ",");
    if (token == NULL) {
        free(line_copy);
        return -1;
    }
    record->age = atoi(token);

    token = strtok(NULL, ",");
    if (token == NULL) {
        free(line_copy);
        return -1;
    }
    strncpy(record->city, token, sizeof(record->city) - 1);

    free(line_copy);
    return 0;
}

int read_csv(const char* filename, Record** records, int* count) {
    FILE* fp = fopen(filename, "r");
    if (fp == NULL) {
        return -1;
    }

    int capacity = 10;
    *records = (Record*)malloc(capacity * sizeof(Record));
    *count = 0;

    char line[256];
    while (fgets(line, sizeof(line), fp) != NULL) {
        if (*count >= capacity) {
            capacity *= 2;
            *records = (Record*)realloc(*records, capacity * sizeof(Record));
        }

        if (parse_csv_line(line, &(*records)[*count]) == 0) {
            (*count)++;
        }
    }

    fclose(fp);
    return 0;
}

int main(void) {
    Record* records;
    int count;

    // Note: This would need a test CSV file to actually run
    // For now, just demonstrate the API
    printf("CSV Parser example\n");
    printf("Would parse: name,age,city format\n");

    return 0;
}
