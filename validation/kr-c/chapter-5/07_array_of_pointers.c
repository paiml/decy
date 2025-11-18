/* K&R C Chapter 5.6: Pointer Arrays; Pointers to Pointers
 * Page 107-108
 * Array of pointers to strings
 */

#include <stdio.h>

int main() {
    char *months[] = {
        "Illegal month",
        "January", "February", "March",
        "April", "May", "June",
        "July", "August", "September",
        "October", "November", "December"
    };
    int i;

    for (i = 1; i <= 12; i++)
        printf("Month %d: %s\n", i, months[i]);

    return 0;
}
