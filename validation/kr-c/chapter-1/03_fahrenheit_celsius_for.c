/* K&R C Chapter 1.3: Fahrenheit-Celsius Table (for loop)
 * Page 13
 * Print Fahrenheit-Celsius table using for loop
 */

#include <stdio.h>

int main() {
    int fahr;

    for (fahr = 0; fahr <= 300; fahr = fahr + 20)
        printf("%3d %6.1f\n", fahr, (5.0/9.0) * (fahr-32));

    return 0;
}
