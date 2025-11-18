/* K&R C Chapter 7: Signal Handling with I/O
 * K&R Appendix B: Signal handling during I/O operations
 * Tests signal interruption and recovery in I/O
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <unistd.h>
#include <errno.h>

volatile sig_atomic_t signal_received = 0;
volatile sig_atomic_t interrupted_io = 0;

/* Signal handler */
void signal_handler(int signum) {
    signal_received = signum;
    interrupted_io = 1;
}

/* SIGINT handler */
void sigint_handler(int signum) {
    write(STDOUT_FILENO, "\n  Caught SIGINT (Ctrl+C)\n", 27);
    signal_received = signum;
}

/* SIGALRM handler */
void sigalrm_handler(int signum) {
    write(STDOUT_FILENO, "\n  Timeout!\n", 13);
    signal_received = signum;
}

/* Interrupted read demo */
void interrupted_read_demo(void) {
    printf("=== Interrupted Read Demo ===\n");
    printf("Setting up signal handlers...\n");

    /* Install signal handler */
    struct sigaction sa;
    sa.sa_handler = signal_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;  /* No SA_RESTART - allow interruption */
    sigaction(SIGUSR1, &sa, NULL);

    printf("Creating pipe...\n");
    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - send signal after delay */
        close(pipefd[0]);
        close(pipefd[1]);

        sleep(1);
        printf("  Child: Sending SIGUSR1 to parent...\n");
        kill(getppid(), SIGUSR1);

        exit(0);
    } else {
        /* Parent - try to read (will be interrupted) */
        close(pipefd[1]);

        char buffer[100];
        printf("  Parent: Attempting to read (blocking)...\n");
        ssize_t n = read(pipefd[0], buffer, sizeof(buffer));

        if (n == -1 && errno == EINTR) {
            printf("  Parent: Read interrupted by signal (EINTR)\n");
        } else if (n >= 0) {
            printf("  Parent: Read completed (%zd bytes)\n", n);
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Timed read with alarm */
void timed_read_demo(void) {
    printf("=== Timed Read Demo ===\n");

    /* Set up SIGALRM handler */
    signal(SIGALRM, sigalrm_handler);

    /* Create test file */
    FILE *fp = fopen("timed_read_test.txt", "w");
    fprintf(fp, "Test data\n");
    fclose(fp);

    fp = fopen("timed_read_test.txt", "r");

    printf("Starting read with 2-second timeout...\n");

    /* Set alarm */
    signal_received = 0;
    alarm(2);

    char buffer[100];
    if (fgets(buffer, sizeof(buffer), fp) != NULL) {
        alarm(0);  /* Cancel alarm */
        printf("  Read completed: %s", buffer);
    } else {
        printf("  Read failed or interrupted\n");
    }

    fclose(fp);
    remove("timed_read_test.txt");
    printf("\n");
}

/* Restartable I/O with SA_RESTART */
void restartable_io_demo(void) {
    printf("=== Restartable I/O Demo ===\n");

    /* Install handler with SA_RESTART */
    struct sigaction sa;
    sa.sa_handler = signal_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_RESTART;  /* Automatically restart system calls */
    sigaction(SIGUSR1, &sa, NULL);

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - send signal and then data */
        close(pipefd[0]);

        sleep(1);
        printf("  Child: Sending SIGUSR1...\n");
        kill(getppid(), SIGUSR1);

        sleep(1);
        const char *msg = "Data after signal";
        write(pipefd[1], msg, strlen(msg));
        printf("  Child: Sent data\n");

        close(pipefd[1]);
        exit(0);
    } else {
        /* Parent - read (should restart after signal) */
        close(pipefd[1]);

        char buffer[100];
        printf("  Parent: Reading (with SA_RESTART)...\n");
        ssize_t n = read(pipefd[0], buffer, sizeof(buffer));

        if (n > 0) {
            buffer[n] = '\0';
            printf("  Parent: Read completed: '%s'\n", buffer);
        } else {
            printf("  Parent: Read failed\n");
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Signal-safe file operations */
void signal_safe_file_ops(void) {
    printf("=== Signal-Safe File Operations ===\n");

    signal(SIGINT, sigint_handler);

    printf("Opening file for writing...\n");
    int fd = open("signal_safe_test.txt", O_WRONLY | O_CREAT | O_TRUNC, 0644);

    if (fd == -1) {
        perror("open");
        return;
    }

    printf("Writing data (press Ctrl+C to interrupt)...\n");

    for (int i = 0; i < 5; i++) {
        char buffer[50];
        snprintf(buffer, sizeof(buffer), "Line %d\n", i + 1);

        ssize_t written = 0;
        size_t to_write = strlen(buffer);

        /* Retry on EINTR */
        while (written < (ssize_t)to_write) {
            ssize_t n = write(fd, buffer + written, to_write - written);

            if (n == -1) {
                if (errno == EINTR) {
                    printf("  Write interrupted, retrying...\n");
                    continue;
                } else {
                    perror("write");
                    break;
                }
            }

            written += n;
        }

        sleep(1);  /* Opportunity for interrupt */
    }

    printf("Closing file...\n");
    close(fd);

    /* Show file contents */
    printf("File contents:\n");
    FILE *fp = fopen("signal_safe_test.txt", "r");
    char buffer[100];
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        printf("  %s", buffer);
    }
    fclose(fp);

    remove("signal_safe_test.txt");
    printf("\n");
}

/* Non-blocking I/O with signals */
void nonblocking_io_with_signals(void) {
    printf("=== Non-Blocking I/O with Signals ===\n");

    int pipefd[2];
    pipe(pipefd);

    /* Set read end to non-blocking */
    int flags = fcntl(pipefd[0], F_GETFL, 0);
    fcntl(pipefd[0], F_SETFL, flags | O_NONBLOCK);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - slow writer */
        close(pipefd[0]);

        for (int i = 1; i <= 3; i++) {
            sleep(1);
            char buffer[50];
            snprintf(buffer, sizeof(buffer), "Message %d\n", i);
            write(pipefd[1], buffer, strlen(buffer));
            printf("  Child: Wrote message %d\n", i);
        }

        close(pipefd[1]);
        exit(0);
    } else {
        /* Parent - non-blocking reader */
        close(pipefd[1]);

        printf("  Parent: Reading with non-blocking I/O...\n");

        char buffer[100];
        int attempts = 0;
        int messages = 0;

        while (messages < 3 && attempts < 10) {
            ssize_t n = read(pipefd[0], buffer, sizeof(buffer) - 1);

            if (n > 0) {
                buffer[n] = '\0';
                printf("  Parent: Read '%s'", buffer);
                messages++;
            } else if (n == -1 && errno == EAGAIN) {
                printf("  Parent: Would block, trying again...\n");
                sleep(1);
            } else {
                break;
            }

            attempts++;
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

int main() {
    printf("=== Signal Handling with I/O ===\n\n");

    interrupted_read_demo();
    timed_read_demo();
    restartable_io_demo();
    signal_safe_file_ops();
    nonblocking_io_with_signals();

    printf("Signal handling in I/O:\n");
    printf("  - EINTR: System call interrupted by signal\n");
    printf("  - SA_RESTART: Automatically restart interrupted calls\n");
    printf("  - Signal-safe functions: Limited set during handlers\n");
    printf("  - Timed I/O: Use alarm() or select() with timeout\n");
    printf("  - Non-blocking I/O: Returns EAGAIN instead of blocking\n");

    return 0;
}
