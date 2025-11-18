/* K&R C Chapter 7: String Formatting
 * K&R §7.2: sprintf, snprintf
 * Tests formatted string building
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Build formatted report */
void build_report(int id, const char *name, double amount) {
    char buffer[200];

    sprintf(buffer, "Report #%04d: %s - $%.2f", id, name, amount);
    printf("Report: %s\n", buffer);
}

/* Safe string formatting with snprintf */
void safe_format(void) {
    char buffer[20];  /* Intentionally small */
    const char *name = "Very Long Name That Won't Fit";

    /* Unsafe - may overflow */
    // sprintf(buffer, "Hello, %s!", name);  // DANGEROUS!

    /* Safe - truncates if needed */
    int written = snprintf(buffer, sizeof(buffer), "Hello, %s!", name);

    printf("snprintf result:\n");
    printf("  Buffer: '%s'\n", buffer);
    printf("  Bytes written (would be): %d\n", written);
    printf("  Buffer size: %lu\n", sizeof(buffer));

    if (written >= (int)sizeof(buffer)) {
        printf("  WARNING: Output was truncated\n");
    }
}

/* Build SQL query */
void build_sql_query(const char *table, const char *column, int value) {
    char query[200];

    snprintf(query, sizeof(query),
             "SELECT * FROM %s WHERE %s = %d;",
             table, column, value);

    printf("SQL Query: %s\n", query);
}

/* Format timestamp */
void format_timestamp(time_t t) {
    char buffer[100];
    struct tm *tm_info = localtime(&t);

    strftime(buffer, sizeof(buffer), "%Y-%m-%d %H:%M:%S", tm_info);
    printf("Formatted time: %s\n", buffer);

    /* Alternative formats */
    strftime(buffer, sizeof(buffer), "%B %d, %Y", tm_info);
    printf("Date only: %s\n", buffer);

    strftime(buffer, sizeof(buffer), "%I:%M %p", tm_info);
    printf("Time only: %s\n", buffer);
}

/* Build JSON string */
void build_json(int id, const char *name, int age, double salary) {
    char json[500];

    snprintf(json, sizeof(json),
             "{\n"
             "  \"id\": %d,\n"
             "  \"name\": \"%s\",\n"
             "  \"age\": %d,\n"
             "  \"salary\": %.2f\n"
             "}",
             id, name, age, salary);

    printf("JSON:\n%s\n", json);
}

/* Build CSV line */
void build_csv_line(int fields[], int count) {
    char buffer[200];
    int offset = 0;

    for (int i = 0; i < count; i++) {
        int written = snprintf(buffer + offset, sizeof(buffer) - offset,
                              "%d%s", fields[i], (i < count - 1) ? "," : "");
        if (written < 0 || offset + written >= (int)sizeof(buffer)) {
            printf("Error: Buffer too small\n");
            return;
        }
        offset += written;
    }

    printf("CSV: %s\n", buffer);
}

/* Format file size */
void format_file_size(long bytes) {
    char buffer[50];

    if (bytes < 1024) {
        snprintf(buffer, sizeof(buffer), "%ld bytes", bytes);
    } else if (bytes < 1024 * 1024) {
        snprintf(buffer, sizeof(buffer), "%.2f KB", bytes / 1024.0);
    } else if (bytes < 1024 * 1024 * 1024) {
        snprintf(buffer, sizeof(buffer), "%.2f MB", bytes / (1024.0 * 1024));
    } else {
        snprintf(buffer, sizeof(buffer), "%.2f GB", bytes / (1024.0 * 1024 * 1024));
    }

    printf("Size: %s\n", buffer);
}

/* Format progress bar */
void format_progress_bar(int percent) {
    char buffer[60];
    int bar_width = 50;
    int filled = (bar_width * percent) / 100;

    snprintf(buffer, sizeof(buffer), "[%.*s%.*s] %d%%",
             filled, "##################################################",
             bar_width - filled, "                                                  ",
             percent);

    printf("%s\n", buffer);
}

/* Build HTTP header */
void build_http_header(const char *method, const char *path, const char *host) {
    char header[500];

    snprintf(header, sizeof(header),
             "%s %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "User-Agent: CustomClient/1.0\r\n"
             "Accept: */*\r\n"
             "\r\n",
             method, path, host);

    printf("HTTP Header:\n%s", header);
}

/* Format table row */
void format_table_row(const char *col1, const char *col2, const char *col3) {
    char buffer[100];

    snprintf(buffer, sizeof(buffer), "| %-20s | %-15s | %-10s |",
             col1, col2, col3);

    printf("%s\n", buffer);
}

int main() {
    printf("=== String Formatting ===\n\n");

    /* Basic reports */
    printf("Formatted reports:\n");
    build_report(42, "Alice", 1234.56);
    build_report(123, "Bob", 9876.54);
    printf("\n");

    /* Safe formatting */
    printf("Safe formatting with snprintf:\n");
    safe_format();
    printf("\n");

    /* SQL queries */
    printf("SQL query building:\n");
    build_sql_query("users", "id", 42);
    build_sql_query("products", "price", 100);
    printf("\n");

    /* Timestamp formatting */
    printf("Timestamp formatting:\n");
    time_t now = time(NULL);
    format_timestamp(now);
    printf("\n");

    /* JSON building */
    printf("JSON building:\n");
    build_json(101, "Alice", 30, 75000.00);
    printf("\n");

    /* CSV building */
    printf("CSV line building:\n");
    int data[] = {10, 20, 30, 40, 50};
    build_csv_line(data, 5);
    printf("\n");

    /* File size formatting */
    printf("File size formatting:\n");
    format_file_size(500);
    format_file_size(5000);
    format_file_size(5000000);
    format_file_size(5000000000L);
    printf("\n");

    /* Progress bar */
    printf("Progress bar:\n");
    format_progress_bar(0);
    format_progress_bar(25);
    format_progress_bar(50);
    format_progress_bar(75);
    format_progress_bar(100);
    printf("\n");

    /* HTTP header */
    printf("HTTP header building:\n");
    build_http_header("GET", "/api/users", "example.com");
    printf("\n");

    /* Table formatting */
    printf("Table formatting:\n");
    format_table_row("Name", "Department", "Salary");
    format_table_row("--------------------", "---------------", "----------");
    format_table_row("Alice Johnson", "Engineering", "$75,000");
    format_table_row("Bob Smith", "Marketing", "$65,000");

    printf("\nFormatting best practices:\n");
    printf("  - Use snprintf() instead of sprintf()\n");
    printf("  - Always check return value\n");
    printf("  - Ensure buffer is large enough\n");
    printf("  - Use appropriate format specifiers\n");

    return 0;
}
