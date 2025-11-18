/* K&R C Chapter 7: Random Access File I/O
 * K&R §7.5: fseek, ftell, rewind
 * Tests random access file positioning
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define RECORD_SIZE 100

typedef struct {
    int id;
    char name[50];
    int age;
    float score;
} Record;

/* Write record at specific position */
int write_record_at(FILE *fp, long position, Record *rec) {
    if (fseek(fp, position, SEEK_SET) != 0) {
        perror("fseek");
        return -1;
    }
    fwrite(rec, sizeof(Record), 1, fp);
    return 0;
}

/* Read record at specific position */
int read_record_at(FILE *fp, long position, Record *rec) {
    if (fseek(fp, position, SEEK_SET) != 0) {
        perror("fseek");
        return -1;
    }
    size_t read = fread(rec, sizeof(Record), 1, fp);
    if (read != 1) {
        fprintf(stderr, "Failed to read record\n");
        return -1;
    }
    return 0;
}

/* Get number of records in file */
long get_record_count(FILE *fp) {
    long current = ftell(fp);  /* Save current position */
    fseek(fp, 0, SEEK_END);
    long filesize = ftell(fp);
    fseek(fp, current, SEEK_SET);  /* Restore position */
    return filesize / sizeof(Record);
}

/* Reverse iteration through file */
void print_records_reverse(FILE *fp) {
    long count = get_record_count(fp);
    Record rec;

    printf("Records in reverse order:\n");
    for (long i = count - 1; i >= 0; i--) {
        read_record_at(fp, i * sizeof(Record), &rec);
        printf("  [%ld] %s (ID: %d, Age: %d, Score: %.1f)\n",
               i, rec.name, rec.id, rec.age, rec.score);
    }
}

/* Binary search in sorted file */
int binary_search_by_id(FILE *fp, int target_id, Record *result) {
    long left = 0;
    long right = get_record_count(fp) - 1;
    Record rec;

    while (left <= right) {
        long mid = left + (right - left) / 2;
        read_record_at(fp, mid * sizeof(Record), &rec);

        if (rec.id == target_id) {
            *result = rec;
            return mid;  /* Found at index mid */
        }
        if (rec.id < target_id)
            left = mid + 1;
        else
            right = mid - 1;
    }

    return -1;  /* Not found */
}

int main() {
    const char *filename = "records.dat";

    printf("=== Random Access File I/O ===\n\n");

    /* Create file with records */
    FILE *fp = fopen(filename, "w+b");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }

    /* Write records (sorted by ID for binary search) */
    Record records[] = {
        {101, "Alice", 25, 88.5},
        {103, "Bob", 30, 92.0},
        {105, "Carol", 22, 85.5},
        {107, "David", 28, 90.0},
        {109, "Eve", 26, 94.5}
    };
    int count = sizeof(records) / sizeof(records[0]);

    printf("Writing %d records...\n", count);
    for (int i = 0; i < count; i++) {
        write_record_at(fp, i * sizeof(Record), &records[i]);
    }

    /* Check file size */
    long record_count = get_record_count(fp);
    printf("File contains %ld records\n\n", record_count);

    /* Random access read */
    printf("Random access reads:\n");
    int indices[] = {2, 0, 4, 1};
    for (int i = 0; i < 4; i++) {
        Record rec;
        read_record_at(fp, indices[i] * sizeof(Record), &rec);
        printf("  Record %d: %s (ID: %d)\n", indices[i], rec.name, rec.id);
    }
    printf("\n");

    /* Reverse iteration */
    print_records_reverse(fp);
    printf("\n");

    /* Binary search */
    printf("Binary search for ID 105:\n");
    Record found;
    int index = binary_search_by_id(fp, 105, &found);
    if (index >= 0) {
        printf("  Found at index %d: %s (Age: %d, Score: %.1f)\n",
               index, found.name, found.age, found.score);
    } else {
        printf("  Not found\n");
    }

    printf("\nBinary search for ID 999 (should not exist):\n");
    index = binary_search_by_id(fp, 999, &found);
    if (index >= 0) {
        printf("  Found at index %d\n", index);
    } else {
        printf("  Not found (as expected)\n");
    }

    /* Update record in place */
    printf("\nUpdating record at index 2...\n");
    Record update = {105, "Carol Smith", 23, 87.0};
    write_record_at(fp, 2 * sizeof(Record), &update);

    /* Verify update */
    Record verify;
    read_record_at(fp, 2 * sizeof(Record), &verify);
    printf("  Updated record: %s (Age: %d, Score: %.1f)\n",
           verify.name, verify.age, verify.score);

    /* Test ftell and rewind */
    printf("\nPosition tests:\n");
    printf("  Current position: %ld\n", ftell(fp));
    rewind(fp);
    printf("  After rewind: %ld\n", ftell(fp));
    fseek(fp, 0, SEEK_END);
    printf("  At end: %ld bytes\n", ftell(fp));

    fclose(fp);

    printf("\nRandom access benefits:\n");
    printf("  - Direct record access without scanning\n");
    printf("  - Binary search on sorted files\n");
    printf("  - In-place updates\n");
    printf("  - Reverse iteration\n");

    return 0;
}
