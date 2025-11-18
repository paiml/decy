/* K&R C Chapter 7.4: Formatted Input - scanf
 * Page 157-159
 * Reading formatted input with scanf
 */

#include <stdio.h>

int main() {
    int day, year;
    char month[20];
    float temperature;

    printf("Enter date (e.g., January 15 2024): ");
    scanf("%s %d %d", month, &day, &year);

    printf("Enter temperature: ");
    scanf("%f", &temperature);

    printf("Date: %s %d, %d\n", month, day, year);
    printf("Temperature: %.1f degrees\n", temperature);

    return 0;
}
