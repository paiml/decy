/* K&R C Chapter 1.4: Symbolic Constants
 * Page 14-15
 * Using #define for constants
 */

#include <stdio.h>

#define LOWER 0
#define UPPER 300
#define STEP 20

int main() {
    int fahr;

    printf("Fahrenheit-Celsius table (using symbolic constants)\n");
    printf("=================================================\n");

    for (fahr = LOWER; fahr <= UPPER; fahr = fahr + STEP)
        printf("%3d %6.1f\n", fahr, (5.0 / 9.0) * (fahr - 32));

    return 0;
}
