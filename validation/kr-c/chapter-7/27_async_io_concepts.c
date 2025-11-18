/* K&R C Chapter 7: Asynchronous I/O Concepts
 * K&R Appendix B: Non-blocking and asynchronous I/O patterns
 * Tests async I/O simulation and patterns
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/select.h>
#include <sys/time.h>

/* Non-blocking read demo */
void nonblocking_read_demo(void) {
    printf("=== Non-Blocking Read Demo ===\n");

    /* Create test file */
    const char *filename = "nonblock_test.txt";
    FILE *fp = fopen(filename, "w");
    fprintf(fp, "Test data for non-blocking read\n");
    fclose(fp);

    /* Open with O_NONBLOCK */
    int fd = open(filename, O_RDONLY | O_NONBLOCK);
    if (fd == -1) {
        perror("open");
        return;
    }

    printf("Reading with non-blocking mode:\n");

    char buffer[100];
    ssize_t n = read(fd, buffer, sizeof(buffer) - 1);

    if (n > 0) {
        buffer[n] = '\0';
        printf("  Read: %s", buffer);
    } else if (n == -1 && errno == EAGAIN) {
        printf("  Would block (EAGAIN)\n");
    } else {
        printf("  Error or EOF\n");
    }

    close(fd);
    remove(filename);
    printf("\n");
}

/* Select-based I/O multiplexing */
void select_demo(void) {
    printf("=== Select I/O Multiplexing Demo ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - writer */
        close(pipefd[0]);

        for (int i = 1; i <= 3; i++) {
            sleep(1);
            char msg[50];
            snprintf(msg, sizeof(msg), "Message %d\n", i);
            write(pipefd[1], msg, strlen(msg));
            printf("  Child: Wrote message %d\n", i);
        }

        close(pipefd[1]);
        exit(0);
    } else {
        /* Parent - select-based reader */
        close(pipefd[1]);

        printf("  Parent: Using select() to monitor pipe...\n");

        for (int i = 0; i < 3; i++) {
            fd_set readfds;
            FD_ZERO(&readfds);
            FD_SET(pipefd[0], &readfds);

            struct timeval timeout;
            timeout.tv_sec = 5;
            timeout.tv_usec = 0;

            printf("  Parent: Waiting for data (timeout 5s)...\n");
            int ready = select(pipefd[0] + 1, &readfds, NULL, NULL, &timeout);

            if (ready == -1) {
                perror("select");
                break;
            } else if (ready == 0) {
                printf("  Parent: Timeout!\n");
                break;
            } else {
                if (FD_ISSET(pipefd[0], &readfds)) {
                    char buffer[100];
                    ssize_t n = read(pipefd[0], buffer, sizeof(buffer) - 1);
                    if (n > 0) {
                        buffer[n] = '\0';
                        printf("  Parent: Read '%s'", buffer);
                    }
                }
            }
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Multiple file monitoring with select */
void multiple_file_select(void) {
    printf("=== Multiple File Monitoring ===\n");

    int pipe1[2], pipe2[2];
    pipe(pipe1);
    pipe(pipe2);

    /* Fork two children */
    pid_t pid1 = fork();
    if (pid1 == 0) {
        close(pipe1[0]);
        close(pipe2[0]);
        close(pipe2[1]);

        sleep(1);
        write(pipe1[1], "From child 1\n", 13);

        close(pipe1[1]);
        exit(0);
    }

    pid_t pid2 = fork();
    if (pid2 == 0) {
        close(pipe1[0]);
        close(pipe1[1]);
        close(pipe2[0]);

        sleep(2);
        write(pipe2[1], "From child 2\n", 13);

        close(pipe2[1]);
        exit(0);
    }

    /* Parent - monitor both pipes */
    close(pipe1[1]);
    close(pipe2[1]);

    printf("  Parent: Monitoring two pipes...\n");

    int pipes_open = 2;
    while (pipes_open > 0) {
        fd_set readfds;
        FD_ZERO(&readfds);

        if (pipe1[0] != -1)
            FD_SET(pipe1[0], &readfds);
        if (pipe2[0] != -1)
            FD_SET(pipe2[0], &readfds);

        int maxfd = (pipe1[0] > pipe2[0] ? pipe1[0] : pipe2[0]);

        struct timeval timeout;
        timeout.tv_sec = 5;
        timeout.tv_usec = 0;

        int ready = select(maxfd + 1, &readfds, NULL, NULL, &timeout);

        if (ready == -1) {
            perror("select");
            break;
        } else if (ready == 0) {
            printf("  Parent: Timeout\n");
            break;
        }

        char buffer[100];

        if (pipe1[0] != -1 && FD_ISSET(pipe1[0], &readfds)) {
            ssize_t n = read(pipe1[0], buffer, sizeof(buffer) - 1);
            if (n > 0) {
                buffer[n] = '\0';
                printf("  Parent: Pipe 1: %s", buffer);
            } else {
                close(pipe1[0]);
                pipe1[0] = -1;
                pipes_open--;
            }
        }

        if (pipe2[0] != -1 && FD_ISSET(pipe2[0], &readfds)) {
            ssize_t n = read(pipe2[0], buffer, sizeof(buffer) - 1);
            if (n > 0) {
                buffer[n] = '\0';
                printf("  Parent: Pipe 2: %s", buffer);
            } else {
                close(pipe2[0]);
                pipe2[0] = -1;
                pipes_open--;
            }
        }
    }

    wait(NULL);
    wait(NULL);
    printf("\n");
}

/* Event loop pattern */
typedef struct {
    int fd;
    void (*callback)(int fd, void *data);
    void *data;
} EventHandler;

void read_handler(int fd, void *data) {
    char buffer[100];
    ssize_t n = read(fd, buffer, sizeof(buffer) - 1);
    if (n > 0) {
        buffer[n] = '\0';
        printf("  Event handler: Read '%s'", buffer);
    }
}

void event_loop_demo(void) {
    printf("=== Event Loop Pattern ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - generate events */
        close(pipefd[0]);

        for (int i = 1; i <= 3; i++) {
            sleep(1);
            char msg[50];
            snprintf(msg, sizeof(msg), "Event %d\n", i);
            write(pipefd[1], msg, strlen(msg));
        }

        close(pipefd[1]);
        exit(0);
    } else {
        /* Parent - event loop */
        close(pipefd[1]);

        EventHandler handler = {pipefd[0], read_handler, NULL};

        printf("  Starting event loop...\n");

        int running = 1;
        int events = 0;

        while (running && events < 3) {
            fd_set readfds;
            FD_ZERO(&readfds);
            FD_SET(handler.fd, &readfds);

            struct timeval timeout;
            timeout.tv_sec = 5;
            timeout.tv_usec = 0;

            int ready = select(handler.fd + 1, &readfds, NULL, NULL, &timeout);

            if (ready > 0 && FD_ISSET(handler.fd, &readfds)) {
                handler.callback(handler.fd, handler.data);
                events++;
            } else if (ready == 0) {
                printf("  Event loop: Timeout\n");
                running = 0;
            }
        }

        printf("  Event loop finished\n");

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Simulated async callback pattern */
typedef void (*AsyncCallback)(const char *result, void *user_data);

typedef struct {
    AsyncCallback callback;
    void *user_data;
    char result[100];
} AsyncRequest;

void async_callback(const char *result, void *user_data) {
    printf("  Async callback: Got result '%s'\n", result);
}

void async_read_file(const char *filename, AsyncCallback callback, void *user_data) {
    printf("  Initiating async read of %s...\n", filename);

    /* In real async I/O, this would be non-blocking */
    /* Here we simulate with immediate callback */

    FILE *fp = fopen(filename, "r");
    if (fp) {
        char buffer[100];
        if (fgets(buffer, sizeof(buffer), fp) != NULL) {
            callback(buffer, user_data);
        }
        fclose(fp);
    }
}

void async_callback_demo(void) {
    printf("=== Async Callback Pattern ===\n");

    /* Create test file */
    FILE *fp = fopen("async_test.txt", "w");
    fprintf(fp, "Async data\n");
    fclose(fp);

    /* Initiate async operation */
    async_read_file("async_test.txt", async_callback, NULL);

    remove("async_test.txt");
    printf("\n");
}

int main() {
    printf("=== Asynchronous I/O Concepts ===\n\n");

    nonblocking_read_demo();
    select_demo();
    multiple_file_select();
    event_loop_demo();
    async_callback_demo();

    printf("Asynchronous I/O patterns:\n");
    printf("  - Non-blocking I/O: Returns immediately (EAGAIN)\n");
    printf("  - select(): Monitor multiple file descriptors\n");
    printf("  - Event loop: Respond to I/O readiness\n");
    printf("  - Callbacks: Handle completion asynchronously\n");
    printf("  - Multiplexing: Handle multiple I/O streams efficiently\n");

    return 0;
}
