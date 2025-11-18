/* K&R C Chapter 4: Const Variables
 * Demonstrates const qualifier on global variables
 */

#include <stdio.h>

const int MAX_BUFFER = 1024;
const double PI = 3.14159;

int main() {
    printf("MAX_BUFFER = %d\n", MAX_BUFFER);
    printf("PI = %f\n", PI);
    return 0;
}
