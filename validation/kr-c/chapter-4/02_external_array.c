/* K&R C Chapter 4: External Arrays
 * Global array declaration example
 */

#include <stdio.h>

int buf[10];  /* External array */

int main() {
    int i;
    for (i = 0; i < 10; i++) {
        buf[i] = i * 2;
    }

    printf("buf[5] = %d\n", buf[5]);
    return 0;
}
