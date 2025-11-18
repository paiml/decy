/* K&R C Chapter 7.2, 7.4: String Formatting
 * Page 154, 159
 * sprintf and sscanf for string manipulation
 */

#include <stdio.h>

int main() {
    char buffer[100];
    int day, year;
    char month[20];
    float temp;

    /* Format into string */
    sprintf(buffer, "Date: %s %d, %d", "January", 15, 2024);
    printf("Formatted string: %s\n", buffer);

    /* Parse from string */
    sscanf("January 15 2024", "%s %d %d", month, &day, &year);
    printf("Parsed: month=%s day=%d year=%d\n", month, day, year);

    /* Number parsing */
    sscanf("Temperature: 72.5", "Temperature: %f", &temp);
    printf("Temperature: %.1f\n", temp);

    return 0;
}
