/* K&R C Chapter 6.3: Arrays of Structures
 * Page 130-131
 * Array of structures for data records
 */

#include <stdio.h>
#include <string.h>

#define MAXKEYS 10

struct key {
    char *word;
    int count;
};

struct key keytab[MAXKEYS] = {
    {"auto", 0},
    {"break", 0},
    {"case", 0},
    {"char", 0},
    {"const", 0},
    {"continue", 0},
    {"default", 0},
    {"do", 0},
    {"double", 0},
    {"else", 0}
};

int main() {
    int i;

    /* Increment counts for demonstration */
    keytab[0].count = 5;
    keytab[1].count = 3;
    keytab[4].count = 7;

    printf("Keyword counts:\n");
    for (i = 0; i < MAXKEYS; i++) {
        if (keytab[i].count > 0)
            printf("%-10s %d\n", keytab[i].word, keytab[i].count);
    }

    return 0;
}
