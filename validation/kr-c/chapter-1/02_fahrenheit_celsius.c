/* K&R C Chapter 1.2: Fahrenheit-Celsius Table
 * Page 8-9
 * Print Fahrenheit-Celsius table using while loop
 */

#include <stdio.h>

int main() {
    int fahr, celsius;
    int lower, upper, step;

    lower = 0;    /* lower limit of temperature table */
    upper = 300;  /* upper limit */
    step = 20;    /* step size */

    fahr = lower;
    while (fahr <= upper) {
        celsius = 5 * (fahr-32) / 9;
        printf("%d\t%d\n", fahr, celsius);
        fahr = fahr + step;
    }
    return 0;
}
