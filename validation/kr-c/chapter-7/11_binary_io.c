/* K&R C Chapter 7: Binary I/O Operations
 * K&R §7.5: fread and fwrite
 * Tests binary file reading and writing
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int id;
    char name[50];
    float salary;
    int years;
} Employee;

/* Write binary data */
int write_employees(const char *filename, Employee *employees, int count) {
    FILE *fp = fopen(filename, "wb");
    if (fp == NULL) {
        perror("fopen");
        return -1;
    }

    size_t written = fwrite(employees, sizeof(Employee), count, fp);
    if (written != (size_t)count) {
        fprintf(stderr, "Warning: only wrote %zu of %d records\n", written, count);
    }

    fclose(fp);
    return 0;
}

/* Read binary data */
int read_employees(const char *filename, Employee **employees, int *count) {
    FILE *fp = fopen(filename, "rb");
    if (fp == NULL) {
        perror("fopen");
        return -1;
    }

    /* Get file size */
    fseek(fp, 0, SEEK_END);
    long filesize = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    *count = filesize / sizeof(Employee);
    *employees = malloc(sizeof(Employee) * (*count));

    size_t read_count = fread(*employees, sizeof(Employee), *count, fp);
    if (read_count != (size_t)(*count)) {
        fprintf(stderr, "Warning: only read %zu of %d records\n", read_count, *count);
    }

    fclose(fp);
    return 0;
}

/* Append employee to file */
int append_employee(const char *filename, Employee *emp) {
    FILE *fp = fopen(filename, "ab");
    if (fp == NULL) {
        perror("fopen");
        return -1;
    }

    fwrite(emp, sizeof(Employee), 1, fp);
    fclose(fp);
    return 0;
}

/* Update specific employee */
int update_employee(const char *filename, int index, Employee *emp) {
    FILE *fp = fopen(filename, "r+b");
    if (fp == NULL) {
        perror("fopen");
        return -1;
    }

    /* Seek to record position */
    fseek(fp, index * sizeof(Employee), SEEK_SET);
    fwrite(emp, sizeof(Employee), 1, fp);

    fclose(fp);
    return 0;
}

int main() {
    const char *filename = "employees.dat";

    printf("=== Binary I/O Operations ===\n\n");

    /* Create sample data */
    Employee employees[] = {
        {101, "Alice Johnson", 75000.0, 5},
        {102, "Bob Smith", 82000.0, 8},
        {103, "Carol Davis", 68000.0, 3},
        {104, "David Wilson", 91000.0, 12}
    };
    int count = sizeof(employees) / sizeof(employees[0]);

    /* Write binary data */
    printf("Writing %d employees to binary file...\n", count);
    if (write_employees(filename, employees, count) != 0) {
        fprintf(stderr, "Failed to write employees\n");
        return 1;
    }

    /* Read binary data */
    printf("Reading employees from binary file...\n");
    Employee *read_employees_array;
    int read_count;
    if (read_employees(filename, &read_employees_array, &read_count) != 0) {
        fprintf(stderr, "Failed to read employees\n");
        return 1;
    }

    printf("\nEmployees read from file:\n");
    for (int i = 0; i < read_count; i++) {
        printf("  [%d] %s - ID: %d, Salary: $%.2f, Years: %d\n",
               i,
               read_employees_array[i].name,
               read_employees_array[i].id,
               read_employees_array[i].salary,
               read_employees_array[i].years);
    }

    /* Append new employee */
    printf("\nAppending new employee...\n");
    Employee new_emp = {105, "Eve Martinez", 77000.0, 6};
    append_employee(filename, &new_emp);

    /* Update specific employee */
    printf("Updating employee at index 1...\n");
    Employee updated = {102, "Bob Smith", 85000.0, 9};
    update_employee(filename, 1, &updated);

    /* Read again to verify */
    free(read_employees_array);
    read_employees(filename, &read_employees_array, &read_count);

    printf("\nEmployees after append and update:\n");
    for (int i = 0; i < read_count; i++) {
        printf("  [%d] %s - ID: %d, Salary: $%.2f, Years: %d\n",
               i,
               read_employees_array[i].name,
               read_employees_array[i].id,
               read_employees_array[i].salary,
               read_employees_array[i].years);
    }

    free(read_employees_array);

    printf("\nBinary I/O advantages:\n");
    printf("  - Faster than text I/O\n");
    printf("  - Preserves exact data representation\n");
    printf("  - Random access via fseek\n");
    printf("  - Fixed-size records for easy indexing\n");

    return 0;
}
