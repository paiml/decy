/* K&R C Chapter 8: Signal Handling
 * K&R §8.7: signal() and signal handlers
 * Tests Unix signal mechanisms
 */

#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>

volatile sig_atomic_t signal_received = 0;

void sigint_handler(int signum) {
    printf("\n  Caught SIGINT (Ctrl+C)\n");
    signal_received = 1;
}

void sigalrm_handler(int signum) {
    printf("  SIGALRM: Timer expired\n");
}

void demo_sigint(void) {
    printf("=== SIGINT Handler Demo ===\n");
    printf("Press Ctrl+C (or wait 3 seconds)...\n");

    signal(SIGINT, sigint_handler);

    alarm(3);  /* Fallback */

    while (!signal_received && alarm(0) != 0) {
        sleep(1);
    }

    printf("Continuing...\n\n");
}

void demo_sigalrm(void) {
    printf("=== SIGALRM Handler Demo ===\n");

    signal(SIGALRM, sigalrm_handler);

    printf("Setting alarm for 2 seconds...\n");
    alarm(2);

    pause();  /* Wait for signal */

    printf("\n");
}

int main() {
    printf("=== Signal Handling ===\n\n");

    demo_sigalrm();

    printf("Signals:\n");
    printf("  - Asynchronous notifications\n");
    printf("  - SIGINT: Interrupt (Ctrl+C)\n");
    printf("  - SIGALRM: Alarm timer\n");
    printf("  - SIGTERM: Termination request\n");
    printf("  - signal(): Register handler\n");

    return 0;
}
