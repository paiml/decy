/* K&R C Chapter 8: Environment Variables
 * K&R §8.7: getenv, setenv, environ
 * Tests environment variable access
 */

#include <stdio.h>
#include <stdlib.h>

extern char **environ;

void demo_getenv(void) {
    printf("=== getenv() Demo ===\n");

    const char *home = getenv("HOME");
    if (home) {
        printf("HOME: %s\n", home);
    }

    const char *path = getenv("PATH");
    if (path) {
        printf("PATH: %s\n", path);
    }

    const char *user = getenv("USER");
    if (user) {
        printf("USER: %s\n", user);
    }

    printf("\n");
}

void demo_setenv(void) {
    printf("=== setenv() Demo ===\n");

    setenv("MY_VAR", "HelloWorld", 1);

    const char *value = getenv("MY_VAR");
    printf("MY_VAR: %s\n", value);

    setenv("MY_VAR", "NewValue", 1);
    value = getenv("MY_VAR");
    printf("MY_VAR (updated): %s\n", value);

    unsetenv("MY_VAR");
    value = getenv("MY_VAR");
    printf("MY_VAR (after unset): %s\n", value ? value : "(null)");

    printf("\n");
}

void demo_environ(void) {
    printf("=== environ Array Demo ===\n");
    printf("All environment variables:\n");

    int count = 0;
    for (char **env = environ; *env != NULL; env++) {
        if (count < 5) {  /* Show first 5 */
            printf("  %s\n", *env);
        }
        count++;
    }

    printf("  ... (%d total variables)\n", count);
    printf("\n");
}

int main() {
    printf("=== Environment Variables ===\n\n");

    demo_getenv();
    demo_setenv();
    demo_environ();

    printf("Environment functions:\n");
    printf("  - getenv(): Get variable value\n");
    printf("  - setenv(): Set variable\n");
    printf("  - unsetenv(): Remove variable\n");
    printf("  - environ: Array of all variables\n");

    return 0;
}
