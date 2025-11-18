// Simple word count utility (like wc -w)
// Tests: argc/argv, file I/O, string processing

#include <stdio.h>
#include <ctype.h>

int count_words(const char* filename) {
    FILE* fp = fopen(filename, "r");
    if (fp == NULL) {
        return -1;
    }

    int word_count = 0;
    int in_word = 0;
    int c;

    while ((c = fgetc(fp)) != EOF) {
        if (isspace(c)) {
            in_word = 0;
        } else if (!in_word) {
            in_word = 1;
            word_count++;
        }
    }

    fclose(fp);
    return word_count;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <filename>\n", argv[0]);
        return 1;
    }

    int count = count_words(argv[1]);
    if (count < 0) {
        fprintf(stderr, "Error: Could not open file %s\n", argv[1]);
        return 1;
    }

    printf("%d\n", count);
    return 0;
}
