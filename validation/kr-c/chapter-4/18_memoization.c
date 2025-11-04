/* K&R C Chapter 4: Memoization Pattern
 * Caching function results for performance
 */

#include <stdio.h>
#include <string.h>

#define MAX_CACHE 100

/* Fibonacci with memoization */
static long fib_cache[MAX_CACHE];
static int fib_cache_initialized = 0;

void fib_cache_init(void) {
    for (int i = 0; i < MAX_CACHE; i++)
        fib_cache[i] = -1;
    fib_cache_initialized = 1;
}

long fib_memo(int n) {
    if (!fib_cache_initialized)
        fib_cache_init();

    if (n < 0 || n >= MAX_CACHE)
        return -1;

    /* Check cache */
    if (fib_cache[n] != -1)
        return fib_cache[n];

    /* Base cases */
    if (n <= 1) {
        fib_cache[n] = n;
        return n;
    }

    /* Compute and cache */
    fib_cache[n] = fib_memo(n - 1) + fib_memo(n - 2);
    return fib_cache[n];
}

/* Factorial with memoization */
static long fact_cache[MAX_CACHE];
static int fact_cache_initialized = 0;

void fact_cache_init(void) {
    for (int i = 0; i < MAX_CACHE; i++)
        fact_cache[i] = -1;
    fact_cache_initialized = 1;
}

long factorial_memo(int n) {
    if (!fact_cache_initialized)
        fact_cache_init();

    if (n < 0 || n >= MAX_CACHE)
        return -1;

    /* Check cache */
    if (fact_cache[n] != -1)
        return fact_cache[n];

    /* Base case */
    if (n <= 1) {
        fact_cache[n] = 1;
        return 1;
    }

    /* Compute and cache */
    fact_cache[n] = n * factorial_memo(n - 1);
    return fact_cache[n];
}

/* Call counter for demonstration */
static int call_count = 0;

long fib_slow(int n) {
    call_count++;
    if (n <= 1)
        return n;
    return fib_slow(n - 1) + fib_slow(n - 2);
}

int main() {
    printf("=== Memoization Pattern ===\n\n");

    /* Fibonacci with memoization */
    printf("Fibonacci with memoization:\n");
    for (int i = 0; i <= 20; i++) {
        printf("  fib(%d) = %ld\n", i, fib_memo(i));
    }

    printf("\nCalling fib_memo(20) again (cached): %ld\n", fib_memo(20));
    printf("Calling fib_memo(15) again (cached): %ld\n", fib_memo(15));

    /* Factorial with memoization */
    printf("\nFactorial with memoization:\n");
    for (int i = 0; i <= 15; i++) {
        printf("  %d! = %ld\n", i, factorial_memo(i));
    }

    /* Performance comparison */
    printf("\nPerformance comparison (fib(30)):\n");

    /* Memoized version */
    fib_cache_init();  /* Reset cache */
    printf("  Memoized: %ld\n", fib_memo(30));

    /* Slow version */
    call_count = 0;
    long result = fib_slow(30);
    printf("  Slow (no memo): %ld (%d function calls)\n", result, call_count);

    /* Memoized on second call (should be instant) */
    printf("  Memoized (cached): %ld\n", fib_memo(30));

    printf("\nMemoization benefits:\n");
    printf("  - Trades memory for speed\n");
    printf("  - Eliminates redundant calculations\n");
    printf("  - Dramatic speedup for recursive functions\n");

    return 0;
}
