/* K&R C Chapter 8: Time and Date Functions
 * K&R Appendix B: time, localtime, strftime
 * Tests time handling functions
 */

#include <stdio.h>
#include <time.h>

void demo_time(void) {
    printf("=== time() Demo ===\n");

    time_t now = time(NULL);
    printf("Unix timestamp: %ld seconds since epoch\n", now);
    printf("\n");
}

void demo_localtime(void) {
    printf("=== localtime() Demo ===\n");

    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);

    printf("Year:   %d\n", tm_info->tm_year + 1900);
    printf("Month:  %d\n", tm_info->tm_mon + 1);
    printf("Day:    %d\n", tm_info->tm_mday);
    printf("Hour:   %d\n", tm_info->tm_hour);
    printf("Minute: %d\n", tm_info->tm_min);
    printf("Second: %d\n", tm_info->tm_sec);
    printf("Weekday: %d (0=Sunday)\n", tm_info->tm_wday);
    printf("\n");
}

void demo_strftime(void) {
    printf("=== strftime() Demo ===\n");

    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char buffer[100];

    strftime(buffer, sizeof(buffer), "%Y-%m-%d %H:%M:%S", tm_info);
    printf("ISO format: %s\n", buffer);

    strftime(buffer, sizeof(buffer), "%A, %B %d, %Y", tm_info);
    printf("Long format: %s\n", buffer);

    strftime(buffer, sizeof(buffer), "%I:%M %p", tm_info);
    printf("12-hour time: %s\n", buffer);

    printf("\n");
}

void demo_difftime(void) {
    printf("=== difftime() Demo ===\n");

    time_t start = time(NULL);

    /* Simulate work */
    for (volatile int i = 0; i < 100000000; i++);

    time_t end = time(NULL);
    double elapsed = difftime(end, start);

    printf("Elapsed time: %.2f seconds\n", elapsed);
    printf("\n");
}

int main() {
    printf("=== Time and Date Functions ===\n\n");

    demo_time();
    demo_localtime();
    demo_strftime();
    demo_difftime();

    printf("Time functions:\n");
    printf("  - time(): Get current time\n");
    printf("  - localtime(): Convert to local time\n");
    printf("  - strftime(): Format time as string\n");
    printf("  - difftime(): Calculate time difference\n");

    return 0;
}
