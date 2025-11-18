// Simple grep-like utility
// Tests: string matching, line-by-line I/O, output parameters

#include <stdio.h>
#include <string.h>

int grep_file(const char* pattern, const char* filename, int* match_count) {
    FILE* fp = fopen(filename, "r");
    if (fp == NULL) {
        return -1;
    }

    char line[1024];
    *match_count = 0;

    while (fgets(line, sizeof(line), fp) != NULL) {
        if (strstr(line, pattern) != NULL) {
            printf("%s", line);
            (*match_count)++;
        }
    }

    fclose(fp);
    return 0;
}

int main(int argc, char** argv) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <pattern> <filename>\n", argv[0]);
        return 1;
    }

    int matches = 0;
    if (grep_file(argv[1], argv[2], &matches) != 0) {
        fprintf(stderr, "Error: Could not open file %s\n", argv[2]);
        return 1;
    }

    fprintf(stderr, "Found %d matching lines\n", matches);
    return 0;
}
